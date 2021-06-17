use crate::analysis::Graph;
use crate::disasm::{Architecture, JumpType, Statement};
use fnv::FnvHashMap;
use parse_int::parse;
use regex::Regex;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::{ErrorKind, Read, Write};
use std::path::Path;
use std::rc::Rc;

/// Offset of an artificially created exit node.
pub const SINK_ADDR: u64 = u64::MAX;
/// Offset of an artificially created entry point.
pub const ENTRY_ADDR: u64 = 0;

/// A Control Flow Graph.
///
/// Struct representing a Control Flow Graph (CFG).
/// This is a graph representation of all the possible execution paths in a function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CFG {
    pub(super) root: Option<Rc<BasicBlock>>,
    pub(super) edges: HashMap<Rc<BasicBlock>, [Option<Rc<BasicBlock>>; 2]>,
}

/// Minimum portion of code without any jump.
///
/// Represents a list of statements without any jump, except for the last one.
/// This does not guarantee, however, that the last statement will be a jump.
/// For example, the [CFG::new()] method generates basic blocks in such a way that each jump inside
/// the CFG lands exactly in the first instruction of each basic block (instead of, for example,
/// in the middle of it). This creates some blocks without any jumps inside them but also not
/// terminating with a jump.
///
/// This class does not contains the actual statements, rather than their offsets in the original
/// code.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BasicBlock {
    /// Offset, in the original code, of the **first** instruction belonging to this basic block.
    pub first: u64,
    /// Offset, in the original code, of the **last** instruction belonging to this basic block.
    pub last: u64,
}

impl BasicBlock {
    /// Returns true if the current block is a sink block.
    ///
    /// Sink blocks are added by the [CFG::add_sink()] method.
    pub fn is_sink(&self) -> bool {
        self.first == self.last && self.first == SINK_ADDR
    }

    /// Returns true if the current block is an artificially added entry point for a CFG.
    ///
    /// **NOTE:** The original entry point **WILL NOT** return true with this method; this method
    /// applies only to the node added with the [CFG::add_entry_point()] method.
    pub fn is_entry_point(&self) -> bool {
        self.first == self.last && self.first == ENTRY_ADDR
    }

    /// Creates a new sink block.
    fn new_sink() -> BasicBlock {
        BasicBlock {
            first: SINK_ADDR,
            last: SINK_ADDR,
        }
    }

    /// Creates a new artificial entry point.
    fn new_entry_point() -> BasicBlock {
        BasicBlock {
            first: ENTRY_ADDR,
            last: ENTRY_ADDR,
        }
    }
}

impl Default for BasicBlock {
    fn default() -> Self {
        BasicBlock::new_sink()
    }
}

impl Display for CFG {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let has_sink = self
            .edges
            .iter()
            .filter(|(node, _child)| node.is_sink())
            .map(|(node, _child)| node)
            .last();
        let edges_no = self
            .edges
            .iter()
            .flat_map(|(_node, edge)| edge)
            .filter_map(Option::Some)
            .count();
        if let Some(_sink) = has_sink {
            //TODO: fix this +0 (filter after filter_map) removing everything equal to sink
            write!(f, "CFG({}+1, {}+0)", self.len(), edges_no)
        } else {
            write!(f, "CFG({}, {})", self.len(), edges_no)
        }
    }
}

impl CFG {
    /// Creates a new CFG from a list of statements.
    ///
    /// Given a list of statements and a source architectures, builds the CFG for that list.
    /// The list of statements is presented as slice.
    ///
    /// The newly returned CFG will not contain a sink and will contain only reachable nodes
    /// (thus eliminating indirect jumps).
    /// One should use [CFG::add_sink()] to refine the CFG.
    /// # Examples
    /// Basic usage:
    /// ```
    /// use bcc::analysis::{Graph, CFG};
    /// use bcc::disasm::{ArchX86, Statement};
    ///
    /// let stmts = vec![
    ///     Statement::new(0x38, "cmp dword [var_4h], 0"),
    ///     Statement::new(0x3C, "jle 0x45"),
    ///     Statement::new(0x3E, "mov eax, 0"),
    ///     Statement::new(0x43, "jmp 0x4a"),
    ///     Statement::new(0x45, "mov eax, 1"),
    ///     Statement::new(0x4A, "ret"),
    /// ];
    /// let arch = ArchX86::new_amd64();
    /// let cfg = CFG::new(&stmts, &arch);
    ///
    /// assert_eq!(cfg.len(), 4);
    /// ```
    pub fn new(stmts: &[Statement], arch: &dyn Architecture) -> CFG {
        build_cfg(stmts, arch)
    }

    /// Returns the next basic block.
    ///
    /// Given an optional basic block, returns its follower.
    /// This means the target of an unconditional jump or the next block in case the current block
    /// ends with a conditional jump.
    ///
    /// Returns [Option::None] if there is no next block, the current basic block does not belong to this CFG
    /// or the original BasicBlock is None.
    pub fn next(&self, block: Option<&BasicBlock>) -> Option<&BasicBlock> {
        if let Some(bb) = block {
            let maybe_children = self.edges.get(bb);
            if let Some(children) = maybe_children {
                if let Some(next) = &children[0] {
                    Some(next)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Returns the conditional basic block.
    ///
    /// Given an optional basic block, returns the conditional jump target.
    ///
    /// Returns [Option::None] if the current basic block does not have conditional jumps, does not belong to
    /// this CFG or the original BasicBlock is None.
    pub fn cond(&self, block: Option<&BasicBlock>) -> Option<&BasicBlock> {
        if let Some(bb) = block {
            let maybe_children = self.edges.get(bb);
            if let Some(children) = maybe_children {
                if let Some(cond) = &children[1] {
                    Some(cond)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Converts the current CFG into a Graphviz dot representation.
    ///
    /// The generated file contains also each Basic Blocks starting and ending offset.
    /// This information is recorded as comment for each node in the form
    /// `(start offset, end offset)`.
    ///
    /// This method assumes that every node is reachable from the root. If this is not true, all
    /// unreachable nodes will be considered as a single node with ID [usize::MAX].
    pub fn to_dot(&self) -> String {
        let mut edges_string = Vec::new();
        let mut nodes_string = Vec::new();
        let nodes_ids = self.node_id_map();
        for (node, child) in self.edges.iter() {
            let node_id = nodes_ids.get(node).unwrap_or(&usize::MAX);
            let shape = if node.is_entry_point() || node.is_sink() {
                r#",shape="point""#
            } else {
                ""
            };
            nodes_string.push(format!(
                "{}[comment=\"({},{})\"{}];",
                node_id, node.first, node.last, shape
            ));
            if let Some(next) = &child[0] {
                let next_id = *nodes_ids.get(next).unwrap_or(&usize::MAX);
                edges_string.push(format!("{}->{}", node_id, next_id));
            }
            if let Some(cond) = &child[1] {
                let cond_id = *nodes_ids.get(cond).unwrap_or(&usize::MAX);
                edges_string.push(format!("{}->{}[arrowhead=\"empty\"];", node_id, cond_id));
            }
        }
        format!(
            "digraph{{\n{}\n{}\n}}\n",
            nodes_string.join("\n"),
            edges_string.join("\n")
        )
    }

    /// Constructs a CFG from an external dot file.
    ///
    /// The input string must come from a dot file generated with the [CFG::to_dot] or
    /// [CFG::to_file] methods. This method expects some additional metadata that otherwise is not
    /// present in a dot file.
    ///
    /// This method returns [std::io::Error] in case of malformed input or [std::num::ParseIntError]
    /// in case the input file contains non-parsable numbers.
    pub fn from_dot(str: &str) -> Result<CFG, Box<dyn Error>> {
        // this parser is super dumb, but even a smart one will never work with **any** .dot file
        // because I need to store some metadata about nodes
        let mut lines = str.lines().collect::<Vec<_>>();
        lines.reverse();
        if let Some(_first @ "digraph{") = lines.pop() {
            let mut nodes = HashMap::new();
            let mut edges_next = HashMap::new();
            let mut edges_cond = HashMap::new();
            let node_re =
                Regex::new(r#"(\d+)\[comment="\((\d+),(\d+)\)"(?:,shape="point")?];"#).unwrap();
            let edge_re = Regex::new(r#"(\d+)->(\d+)(\[.*];)?"#).unwrap();
            while let Some(line) = lines.pop() {
                if let Some(cap) = node_re.captures(line) {
                    let id = cap.get(1).unwrap().as_str().parse::<usize>()?;
                    let first = cap.get(2).unwrap().as_str().parse::<u64>()?;
                    let last = cap.get(3).unwrap().as_str().parse::<u64>()?;
                    let node = BasicBlock { first, last };
                    nodes.insert(id, Rc::new(node));
                } else if let Some(cap) = edge_re.captures(line) {
                    let from = cap.get(1).unwrap().as_str().parse::<usize>()?;
                    let to = cap.get(2).unwrap().as_str().parse::<usize>()?;
                    if cap.get(3).is_none() {
                        edges_next.insert(from, to);
                    } else {
                        edges_cond.insert(from, to);
                    }
                }
            }
            let mut edges = HashMap::new();
            // Invalid files may have inconsistent data. This error is used to avoid panicking.
            let parse_err = || {
                Box::new(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "inconsistent data",
                ))
            };
            for (src, dst) in edges_next {
                let src_node = nodes.get(&src).ok_or_else(parse_err)?.clone();
                let dst_node = nodes.get(&dst).ok_or_else(parse_err)?.clone();
                edges.insert(src_node, [Some(dst_node), None]);
            }
            for (src, dst) in edges_cond {
                let src_node = nodes.get(&src).ok_or_else(parse_err)?.clone();
                let dst_node = nodes.get(&dst).ok_or_else(parse_err)?.clone();
                let existing = edges.get_mut(&src_node).ok_or_else(parse_err)?;
                existing[1] = Some(dst_node);
            }
            let root = if !nodes.is_empty() {
                Some(nodes.get(&0).unwrap().clone())
            } else {
                None
            };
            // add the exits
            let exits = nodes
                .into_iter()
                .filter(|(_, node)| !edges.contains_key(node))
                .map(|(_, node)| node)
                .collect::<Vec<_>>();
            for exit in exits {
                edges.insert(exit, [None, None]);
            }
            Ok(CFG { root, edges })
        } else {
            Err(Box::new(std::io::Error::new(
                ErrorKind::InvalidInput,
                "unexpected input filetype",
            )))
        }
    }

    /// Saves the current CFG into a Graphviz representation.
    ///
    /// Given a path to file, saves the current CFG as a Graphviz .dot file.
    /// This is equivalent of calling [CFG::to_dot()] and then saving the String content to file.
    pub fn to_file<S: AsRef<Path>>(&self, filename: S) -> Result<(), io::Error> {
        let mut file = File::create(filename)?;
        file.write_all(self.to_dot().as_bytes())
    }

    /// Retrieves a CFG file from a Graphviz representation.
    ///
    /// Given a path to file, retrieves a CFG previously created with [CFG::to_file] method (or with
    /// [CFG::to_dot] later saved to a file).
    ///
    /// This method returns [std::io::Error] in case of malformed input or [std::num::ParseIntError]
    /// in case the input file contains non-parsable numbers.
    pub fn from_file<S: AsRef<Path>>(filename: S) -> Result<CFG, Box<dyn Error>> {
        let mut file = File::open(filename)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        CFG::from_dot(&content)
    }

    /// Adds a sink to the current CFG.
    ///
    /// In some cases, a CFG may have multiple nodes without children (like in the case of multiple
    /// return statements). This method merges those nodes by attaching them to a sink. The sink
    /// is recognizable by calling [BasicBlock::is_sink()].
    #[must_use]
    pub fn add_sink(mut self) -> CFG {
        let exit_nodes = self
            .edges
            .iter()
            .filter(|(_, child)| child[0].is_none() && child[1].is_none())
            .count();
        if exit_nodes > 1 {
            let sink = Rc::new(BasicBlock::new_sink());
            for (_, child) in self.edges.iter_mut() {
                if child[0].is_none() && child[1].is_none() {
                    child[0] = Some(sink.clone());
                }
            }
            self.edges.insert(sink, [None, None]);
        }
        self
    }

    ///Adds an additional entry point to the current CFG.
    ///
    /// Some transformation requires CFG nodes to have an exact number of entry edges and will fail
    /// for the root node. This method add an additional, bogus root node that allows CFG
    /// transformations to complete successfully. The new artificial entry node is recognizable by
    /// calling [BasicBlock::is_entry_point()] and is added **iff** the original entry point has
    /// one or more predecessors.
    #[must_use]
    pub fn add_entry_point(mut self) -> CFG {
        let oep_has_preds = self
            .edges
            .iter()
            .flat_map(|(_, edge)| edge)
            .any(|x| x == &self.root);
        if oep_has_preds {
            let eep = Rc::new(BasicBlock::new_entry_point());
            self.edges.insert(eep.clone(), [self.root.take(), None]);
            self.root = Some(eep);
        }
        self
    }

    /// Given a node, returns it's shared reference used internally by the CFG.
    ///
    /// This is useful to avoid some caveats with mutability and borrowing without having to clone
    /// the BasicBlock.
    ///
    /// Returns [Option::None] if the input node does not belong to this graph.
    pub fn rc(&self, node: &BasicBlock) -> Option<Rc<BasicBlock>> {
        self.edges.get_key_value(node).map(|(rc, _)| rc.clone())
    }

    /// Assigns an unique ID to each node in the CFG.
    ///
    /// Unless the CFG changes, the id assigned by this method will always be the same, and based on
    /// a preorder visit of the CFG.
    pub fn node_id_map(&self) -> HashMap<Rc<BasicBlock>, usize> {
        self.dfs_preorder()
            .enumerate()
            .map(|(index, node)| (self.rc(node).unwrap(), index))
            .collect::<HashMap<_, _>>()
    }
}

impl Graph for CFG {
    type Item = BasicBlock;

    fn root(&self) -> Option<&Self::Item> {
        self.root.as_ref().map(|root| root.as_ref())
    }

    fn neighbours(&self, node: &Self::Item) -> Option<Vec<&Self::Item>> {
        self.edges.get(node).map(|children| {
            children
                .iter()
                .flatten()
                .map(|x| x.as_ref())
                .collect::<Vec<_>>()
        })
    }

    fn len(&self) -> usize {
        self.edges.len()
    }
}

// struct containing multiple maps related to jumps sources/dests
struct TargetMap {
    // list of offsets that ends up being targets for a jump somewhere
    targets: BTreeSet<u64>,
    // map for conditional jumps: <source offset, dest offset>
    srcs_cond: FnvHashMap<u64, u64>,
    // map for unconditional jumps: <source offset, dest offset>
    srcs_uncond: FnvHashMap<u64, u64>,
    // set for conditional returns containing the return offset
    deadend_cond: BTreeSet<u64>,
    // set for unconditional returns containing the return offset
    deadend_uncond: BTreeSet<u64>,
}

// given a list of Statements and an Architecture creates the TargetMap struct
fn get_targets(stmts: &[Statement], arch: &dyn Architecture) -> TargetMap {
    let mut targets = BTreeSet::default();
    let mut srcs_cond = FnvHashMap::default();
    let mut srcs_uncond = FnvHashMap::default();
    let mut deadend_cond = BTreeSet::default();
    let mut deadend_uncond = BTreeSet::default();
    let empty_stmt = Statement::new(0x0, "");
    let func_lower_bound = stmts.first().unwrap_or(&empty_stmt).get_offset();
    let func_upper_bound = stmts.last().unwrap_or(&empty_stmt).get_offset();
    let mut previous_was_jump = true;
    for stmt in stmts {
        if previous_was_jump {
            previous_was_jump = false;
            targets.insert(stmt.get_offset());
        }
        let mnemonic = stmt.get_mnemonic();
        let jump_type = arch.jump(mnemonic);
        match jump_type {
            JumpType::JumpUnconditional => {
                let maybe_target = parse::<u64>(stmt.get_args());
                if let Ok(target) = maybe_target {
                    // direct jump
                    if target >= func_lower_bound && target <= func_upper_bound {
                        // inside the current function
                        srcs_uncond.insert(stmt.get_offset(), target);
                        targets.insert(target);
                    } else {
                        // unconditional jump outside the function, so it's like a return
                        deadend_uncond.insert(stmt.get_offset());
                    }
                } else {
                    // unconditional jump to an unknown target. this is a problem.
                    deadend_uncond.insert(stmt.get_offset());
                }
                previous_was_jump = true;
            }
            JumpType::JumpConditional => {
                let maybe_target = parse::<u64>(stmt.get_args());
                if let Ok(target) = maybe_target {
                    // direct jump
                    if target >= func_lower_bound && target <= func_upper_bound {
                        // inside the current function
                        srcs_cond.insert(stmt.get_offset(), target);
                    }
                    targets.insert(target);
                }
                previous_was_jump = true;
            }
            JumpType::RetUnconditional => {
                deadend_uncond.insert(stmt.get_offset());
                previous_was_jump = true;
            }
            JumpType::RetConditional => {
                deadend_cond.insert(stmt.get_offset());
                previous_was_jump = true;
            }
            JumpType::NoJump => {}
        }
    }
    TargetMap {
        targets,
        srcs_cond,
        srcs_uncond,
        deadend_cond,
        deadend_uncond,
    }
}

/// Removes unreachable nodes.
///
/// Removes nodes that are not reachable from the CFG root by any path. These nodes are usually
/// created when there are indirect jumps in the original statement list.
fn reachable(cfg: CFG) -> CFG {
    if !cfg.is_empty() {
        let reachables = cfg.dfs_preorder().collect::<HashSet<_>>();
        // need to clone edges map that uses Rc instead of the reachables set
        let edges = cfg
            .edges
            .clone()
            .into_iter()
            .filter(|(node, _child)| reachables.contains(node.as_ref()))
            .collect::<HashMap<_, _>>();
        CFG {
            root: cfg.root,
            edges,
        }
    } else {
        cfg
    }
}

// given an offset (any offset) returns the corresponding basic block id containing it.
// requires:
// - the actual offset
// - a map <basic block starting offset, basic block id>
// - list of jump targets
// this method is used ad-hoc inside the build_cfg function and needs to be rewritten to be used in
// any other case
fn resolve_bb_id(offset: u64, id_map: &FnvHashMap<u64, usize>, targets: &BTreeSet<u64>) -> usize {
    if let Some(ret) = id_map.get(&offset) {
        // corner case: the current offset is also the block start
        *ret
    } else {
        // these MUST exist, otherwise the previous if should happen
        let block_start_offset = targets.range(..offset).last().unwrap();
        *id_map.get(block_start_offset).unwrap()
    }
}

// actual cfg building
fn build_cfg(stmts: &[Statement], arch: &dyn Architecture) -> CFG {
    let all_offsets = stmts
        .iter()
        .map(|x| x.get_offset())
        .collect::<BTreeSet<_>>();
    let tgmap = get_targets(stmts, arch);
    let empty_stmt = Statement::new(0x0, "");
    // This target is used for a strictly lower bound.
    // The +1 is useful so I can use the last statement in the function
    let function_over = stmts.last().unwrap_or(&empty_stmt).get_offset() + 1;
    // create all nodes (without ending statement)
    let mut nodes_tmp = tgmap
        .targets
        .iter()
        .map(|target| BasicBlock {
            first: *target,
            last: 0,
        })
        .collect::<Vec<_>>();
    // fill ending statement offset
    let mut nodes_iter = tgmap.targets.iter().enumerate().peekable();
    while let Some((index, _)) = nodes_iter.next() {
        let next_target = *nodes_iter.peek().unwrap_or(&(0, &function_over)).1;
        let last_stmt = *all_offsets.range(..next_target).last().unwrap_or(&0);
        nodes_tmp[index].last = last_stmt;
    }
    let nodes = nodes_tmp.into_iter().map(Rc::new).collect::<Vec<_>>();
    let mut edges = if !tgmap.targets.is_empty() {
        let mut edges = HashMap::new();
        let mut iter = nodes.iter().peekable();
        while let Some(node) = iter.next() {
            match iter.peek() {
                Some(next) => edges.insert(node.clone(), [Some((*next).clone()), None]),
                None => edges.insert(node.clone(), [None, None]),
            };
        }
        edges
    } else {
        HashMap::new()
    };
    // map every offset to the block id
    let offset_id_map = tgmap
        .targets
        .iter()
        .enumerate()
        .map(|(index, target)| (*target, index))
        .collect::<FnvHashMap<_, _>>();
    for (off_src, off_dst) in tgmap.srcs_uncond {
        let src_id = resolve_bb_id(off_src, &offset_id_map, &tgmap.targets);
        let dst_id = resolve_bb_id(off_dst, &offset_id_map, &tgmap.targets);
        edges.get_mut(&nodes[src_id]).unwrap()[0] = Some(nodes[dst_id].clone());
    }
    for (off_src, off_dst) in tgmap.srcs_cond {
        let src_id = resolve_bb_id(off_src, &offset_id_map, &tgmap.targets);
        let dst_id = resolve_bb_id(off_dst, &offset_id_map, &tgmap.targets);
        edges.get_mut(&nodes[src_id]).unwrap()[1] = Some(nodes[dst_id].clone());
    }
    for ret in tgmap.deadend_uncond {
        let src_id = resolve_bb_id(ret, &offset_id_map, &tgmap.targets);
        edges.get_mut(&nodes[src_id]).unwrap()[0] = None;
    }
    for ret in tgmap.deadend_cond {
        let src_id = resolve_bb_id(ret, &offset_id_map, &tgmap.targets);
        edges.get_mut(&nodes[src_id]).unwrap()[1] = None;
    }
    reachable(CFG {
        root: nodes.first().cloned(),
        edges,
    })
}

#[cfg(test)]
mod tests {
    use crate::analysis::cfg::reachable;
    use crate::analysis::{BasicBlock, Graph, CFG};
    use crate::disasm::{ArchX86, Statement};
    use maplit::hashmap;
    use std::collections::HashMap;
    use std::error::Error;
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::rc::Rc;
    use tempfile::tempfile;

    //digraph 0->1, 1->2, 2->3
    fn sequence() -> CFG {
        let nodes = (0..)
            .take(4)
            .map(|x| Rc::new(BasicBlock { first: x, last: 0 }))
            .collect::<Vec<_>>();
        let edges = hashmap![
            nodes[0].clone() => [Some(nodes[1].clone()), None],
            nodes[1].clone() => [Some(nodes[2].clone()), None],
            nodes[2].clone() => [Some(nodes[3].clone()), None],
            nodes[3].clone() => [None, None],
        ];
        CFG {
            root: Some(nodes[0].clone()),
            edges,
        }
    }

    //digraph 0->1, 2->3 (forced to skip the build_cfg otherwise reachable() will delete 2 and 3)
    fn two_sequences() -> CFG {
        let nodes = (0..)
            .take(4)
            .map(|x| Rc::new(BasicBlock { first: x, last: 0 }))
            .collect::<Vec<_>>();
        let edges = hashmap![
            nodes[0].clone() => [Some(nodes[1].clone()), None],
            nodes[1].clone() => [None, None],
            nodes[2].clone() => [Some(nodes[3].clone()), None],
            nodes[3].clone() => [None, None],
        ];
        CFG {
            root: Some(nodes[0].clone()),
            edges,
        }
    }

    #[test]
    fn root_empty() {
        let stmts = Vec::new();
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        assert!(cfg.root().is_none());
    }

    #[test]
    fn root() {
        let stmts = vec![
            Statement::new(0x61C, "mov eax, 5"),
            Statement::new(0x624, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        assert!(cfg.root().is_some());
        assert_eq!(cfg.root().unwrap().first, 0x61C);
    }

    #[test]
    fn get_children_nonexisting() {
        let stmts = Vec::new();
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        let children = cfg.neighbours(&BasicBlock::new_sink());
        assert!(children.is_none())
    }

    #[test]
    fn get_children_existing_empty() {
        let stmts = vec![
            Statement::new(0x61C, "mov eax, 5"),
            Statement::new(0x624, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        let children = cfg.neighbours(cfg.root().unwrap());
        assert!(children.is_some());
        assert!(children.unwrap().is_empty());
    }

    #[test]
    fn get_children_existing() {
        let stmts = vec![
            Statement::new(0x610, "test edi, edi"),
            Statement::new(0x612, "je 0x618"),
            Statement::new(0x614, "mov eax, 6"),
            Statement::new(0x618, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        let children = cfg.neighbours(cfg.root().unwrap());
        assert!(children.is_some());
        assert_eq!(children.unwrap().len(), 2);
    }

    #[test]
    fn len() {
        let stmts = vec![
            Statement::new(0x610, "test edi, edi"),
            Statement::new(0x612, "je 0x618"),
            Statement::new(0x614, "mov eax, 6"),
            Statement::new(0x618, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        assert_eq!(cfg.len(), 3);
    }

    #[test]
    fn build_cfg_empty() {
        let stmts = Vec::new();
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        assert!(cfg.is_empty());
        assert_eq!(cfg.len(), 0);
    }

    #[test]
    fn next() {
        let cfg = sequence();
        let root = cfg.root();
        assert!(cfg.next(root).is_some())
    }

    #[test]
    fn cond() {
        let cfg = sequence();
        let root = cfg.root();
        assert!(cfg.cond(root).is_none())
    }

    #[test]
    fn build_cfg_conditional_jumps() {
        let stmts = vec![
            Statement::new(0x610, "test edi, edi"), //0
            Statement::new(0x612, "je 0x620"),      //0
            Statement::new(0x614, "test esi, esi"), //1
            Statement::new(0x616, "mov eax, 5"),    //1
            Statement::new(0x61b, "je 0x620"),      //1
            Statement::new(0x61d, "ret"),           //2
            Statement::new(0x620, "mov eax, 6"),    //3
            Statement::new(0x625, "ret"),           //3
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        assert!(!cfg.is_empty());
        assert_eq!(cfg.len(), 4);
        let node0 = cfg.root();
        let node1 = cfg.next(node0);
        let node2 = cfg.next(node1);
        let node3 = cfg.cond(node1);
        assert_eq!(node0.unwrap().first, 0x610);
        assert_eq!(node1.unwrap().first, 0x614);
        assert_eq!(node2.unwrap().first, 0x61D);
        assert_eq!(node3.unwrap().first, 0x620);
        assert!(cfg.next(node2).is_none());
        assert!(cfg.cond(node2).is_none());
        assert!(cfg.next(node3).is_none());
        assert!(cfg.cond(node3).is_none());
    }

    #[test]
    fn build_cfg_unconditional_jumps() {
        let stmts = vec![
            Statement::new(0x61E, "push rbp"),                //0
            Statement::new(0x61F, "mov rbp, rsp"),            //0
            Statement::new(0x622, "mov dword [var_4h], edi"), //0
            Statement::new(0x625, "mov dword [var_8h], esi"), //0
            Statement::new(0x628, "cmp dword [var_4h], 5"),   //0
            Statement::new(0x62C, "jne 0x633"),               //0
            Statement::new(0x62E, "mov eax, dword [var_8h]"), //1
            Statement::new(0x631, "jmp 0x638"),               //1
            Statement::new(0x633, "mov eax, 6"),              //2
            Statement::new(0x638, "pop rbp"),                 //3
            Statement::new(0x639, "ret"),                     //3
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        assert_eq!(cfg.len(), 4);
        let node0 = cfg.root();
        let node1 = cfg.next(node0);
        let node2 = cfg.cond(node0);
        let node3 = cfg.next(node1);
        assert_eq!(node0.unwrap().first, 0x61E);
        assert_eq!(node1.unwrap().first, 0x62E);
        assert_eq!(node2.unwrap().first, 0x633);
        assert_eq!(node3.unwrap().first, 0x638);
        assert!(cfg.cond(node1).is_none());
        assert!(cfg.cond(node2).is_none());
        assert!(cfg.next(node3).is_none());
        assert!(cfg.cond(node3).is_none());
    }

    #[test]
    fn build_cfg_long_unconditional_jump() {
        // this is crafted so offsets are completely random
        let stmts = vec![
            Statement::new(0x610, "test edi, edi"),          //0
            Statement::new(0x611, "je 0x613"),               //0
            Statement::new(0x612, "jmp 0xFFFFFFFFFFFFFFFC"), //1
            Statement::new(0x613, "jmp 0x600"),              //2
            Statement::new(0x614, "jmp 0x615"),              //3
            Statement::new(0x615, "ret"),                    //4
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        assert_eq!(cfg.len(), 3);
        let node0 = cfg.root();
        let node1 = cfg.next(node0);
        let node2 = cfg.cond(node0);
        assert!(cfg.next(node1).is_none());
        assert!(cfg.cond(node1).is_none());
        assert!(cfg.next(node2).is_none());
        assert!(cfg.cond(node2).is_none());
    }

    #[test]
    fn build_cfg_bb_offset() {
        let stmts = vec![
            Statement::new(0x610, "test edi, edi"), //0
            Statement::new(0x614, "je 0x628"),      //0
            Statement::new(0x618, "test esi, esi"), //1
            Statement::new(0x61C, "mov eax, 5"),    //1
            Statement::new(0x620, "je 0x628"),      //2
            Statement::new(0x624, "ret"),           //2
            Statement::new(0x628, "mov eax, 6"),    //3
            Statement::new(0x62C, "ret"),           //3
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        assert_eq!(cfg.len(), 4);
        let node0 = cfg.root();
        let node1 = cfg.next(node0);
        let node2 = cfg.next(node1);
        let node3 = cfg.cond(node0);
        assert_eq!(cfg.cond(node1), node3);
        assert!(cfg.next(node2).is_none());
        assert!(cfg.cond(node2).is_none());
        assert!(cfg.next(node3).is_none());
        assert!(cfg.cond(node3).is_none());
        assert_eq!(node0.unwrap().first, 0x610);
        assert_eq!(node0.unwrap().last, 0x614);
        assert_eq!(node1.unwrap().first, 0x618);
        assert_eq!(node1.unwrap().last, 0x620);
        assert_eq!(node2.unwrap().first, 0x624);
        assert_eq!(node2.unwrap().last, 0x624);
        assert_eq!(node3.unwrap().first, 0x628);
        assert_eq!(node3.unwrap().last, 0x62C);
    }

    #[test]
    fn build_cfg_offset_64bit() {
        let stmts = vec![
            Statement::new(0x3FD1A7EF534, "jmp 0x3FD1A7EF538"),
            Statement::new(0x3FD1A7EF538, "incl eax"),
            Statement::new(0x3FD1A7EF53C, "mov ebx, [ebp+20]"),
            Statement::new(0x3FD1A7EF540, "cmp eax, ebx"),
            Statement::new(0x3FD1A7EF544, "je 0x3FD1A7EF558"),
            Statement::new(0x3FD1A7EF548, "mov ecx, [ebp+20]"),
            Statement::new(0x3FD1A7EF54C, "decl ecx"),
            Statement::new(0x3FD1A7EF550, "mov [ebp+20], ecx"),
            Statement::new(0x3FD1A7EF554, "jmp 0x3FD1A7EF538"),
            Statement::new(0x3FD1A7EF558, "test eax, eax"),
            Statement::new(0x3FD1A7EF55C, "mov eax, 0"),
            Statement::new(0x3FD1A7EF560, "je 0x3FD1A7EF568"),
            Statement::new(0x3FD1A7EF564, "mov eax, 1"),
            Statement::new(0x3FD1A7EF568, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        assert_eq!(cfg.len(), 6);
    }

    #[test]
    fn save_and_retrieve_empty() -> Result<(), Box<dyn Error>> {
        let stmts = Vec::new();
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        let mut file = tempfile()?;
        file.write_all(cfg.to_dot().as_bytes())?;
        file.seek(SeekFrom::Start(0))?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let cfg_read = CFG::from_dot(&content)?;
        assert_eq!(cfg_read, cfg);
        Ok(())
    }

    #[test]
    fn save_and_retrieve() -> Result<(), Box<dyn Error>> {
        let stmts = vec![
            Statement::new(0x61E, "push rbp"),                //0
            Statement::new(0x622, "mov dword [var_4h], edi"), //0
            Statement::new(0x628, "cmp dword [var_4h], 5"),   //0
            Statement::new(0x62C, "jne 0x633"),               //0
            Statement::new(0x62E, "mov eax, dword [var_8h]"), //1
            Statement::new(0x631, "jmp 0x638"),               //1
            Statement::new(0x633, "mov eax, 6"),              //2
            Statement::new(0x638, "pop rbp"),                 //3
            Statement::new(0x639, "ret"),                     //3
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        let mut file = tempfile()?;
        file.write_all(cfg.to_dot().as_bytes())?;
        file.seek(SeekFrom::Start(0))?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let cfg_read = CFG::from_dot(&content)?;
        assert_eq!(cfg_read, cfg);
        Ok(())
    }

    #[test]
    fn save_and_retrieve_with_entry_point() -> Result<(), Box<dyn Error>> {
        let stmts = vec![
            Statement::new(0x61E, "push rbp"),                //0
            Statement::new(0x622, "mov dword [var_4h], edi"), //0
            Statement::new(0x62C, "jne 0x638"),               //0
            Statement::new(0x62E, "ret"),                     //1
            Statement::new(0x638, "pop rbp"),                 //2
            Statement::new(0x639, "ret"),                     //2
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        let cfg_sink_eep = cfg.clone().add_entry_point().add_sink();
        assert_ne!(cfg, cfg_sink_eep);
        let mut file = tempfile()?;
        file.write_all(cfg_sink_eep.to_dot().as_bytes())?;
        file.seek(SeekFrom::Start(0))?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let cfg_read = CFG::from_dot(&content)?;
        assert_eq!(cfg_read, cfg_sink_eep);
        Ok(())
    }

    #[test]
    fn add_sink_empty() {
        let stmts = Vec::new();
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        let cfg_with_sink = cfg.add_sink();
        assert!(cfg_with_sink.is_empty());
    }

    #[test]
    fn add_sink_necessary() {
        let stmts = vec![
            Statement::new(0x61C, "mov eax, 5"),
            Statement::new(0x620, "je 0x628"),
            Statement::new(0x624, "ret"),
            Statement::new(0x628, "mov eax, 6"),
            Statement::new(0x62C, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        assert_eq!(cfg.len(), 3);
        let cfg_with_sink = cfg.add_sink();
        assert_eq!(cfg_with_sink.len(), 4);
    }

    #[test]
    fn add_sink_unnecessary() {
        let stmts = vec![
            Statement::new(0x61C, "mov eax, 5"),
            Statement::new(0x624, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        let cfg_with_sink = cfg.clone().add_sink();
        assert_eq!(cfg.len(), cfg_with_sink.len());
    }

    #[test]
    fn add_extra_entry_point() {
        let stmts = vec![
            Statement::new(0x61C, "mov eax, 5"),
            Statement::new(0x620, "jmp 0x61c"),
            Statement::new(0x624, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        assert_eq!(cfg.len(), 1);
        let cfg_with_eep = cfg.clone().add_entry_point();
        assert_eq!(cfg_with_eep.len(), 2);
        assert!(cfg_with_eep.root().unwrap().is_entry_point());
        assert_eq!(
            cfg_with_eep.next(cfg_with_eep.root()).unwrap(),
            cfg.root().unwrap()
        );
    }

    #[test]
    fn add_extra_entry_point_empty() {
        let cfg = CFG {
            root: None,
            edges: HashMap::new(),
        };
        let cfg_with_eep = cfg.add_entry_point();
        assert!(cfg_with_eep.is_empty());
    }

    #[test]
    fn add_extra_entry_point_unnecessary() {
        let stmts = vec![
            Statement::new(0x61C, "mov eax, 5"),
            Statement::new(0x624, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        assert_eq!(cfg.len(), 1);
        let cfg_with_eep = cfg.clone().add_entry_point();
        assert_eq!(cfg_with_eep, cfg);
    }

    #[test]
    fn reachable_empty() {
        let cfg = CFG {
            root: None,
            edges: HashMap::new(),
        };
        let cfg_only_reachables = reachable(cfg);
        assert!(cfg_only_reachables.is_empty());
    }

    #[test]
    fn reachable_all() {
        let cfg = sequence();
        let cfg_only_reachables = reachable(cfg.clone());
        assert_eq!(cfg_only_reachables.len(), cfg.len());
    }

    #[test]
    fn reachable_some() {
        let cfg = two_sequences();
        let cfg_only_reachables = reachable(cfg);
        assert_eq!(cfg_only_reachables.len(), 2);
    }

    #[test]
    fn reference_unreachable() {
        // add unreachable nodes, then reference them when asking for next
        // assert no panic
        let cfg = two_sequences();
        let cfg_only_reachables = reachable(cfg);
        let node0 = cfg_only_reachables.root();
        let node1 = cfg_only_reachables.next(node0);
        let node2 = cfg_only_reachables.next(node1);
        assert!(node2.is_none());
    }
}
