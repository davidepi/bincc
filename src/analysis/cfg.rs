use crate::analysis::Graph;
use crate::disasm::{Architecture, JumpType, Statement};
use fnv::FnvHashMap;
use parse_int::parse;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;

const SINK_ADDR: u64 = u64::MAX;

/// A Control Flow Graph.
///
/// Struct representing a Control Flow Graph (CFG).
/// This is a graph representation of all the possible execution paths in a function.
#[derive(Debug, Clone)]
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
    /// Numerical integer representing an unique identifier for this block.
    pub id: usize,
    /// Offset in the original code where this basic block begins.
    pub first: u64,
    /// Offset in the original code where this basic block ends.
    pub last: u64,
}

impl BasicBlock {
    /// Returns true if the current block is a sink block.
    ///
    /// Sink blocks are added by the [CFG::add_sink()] method.
    pub fn is_sink(&self) -> bool {
        self.first == self.last && self.first == SINK_ADDR
    }

    /// Creates a new sink block.
    fn new_sink() -> BasicBlock {
        BasicBlock {
            id: usize::MAX,
            first: SINK_ADDR,
            last: SINK_ADDR,
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
    /// Returns None if there is no next block, the current basic block does not belong to this CFG
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
    /// Returns None if the current basic block does not have conditional jumps, does not belong to
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
    pub fn to_dot(&self) -> String {
        let mut content = Vec::new();
        let sink_iter = self
            .edges
            .iter()
            .filter(|(node, _child)| node.is_sink())
            .last();
        let sink = if let Some(sink_node) = sink_iter {
            content.push("0[style=\"dotted\"];".to_string());
            sink_node.0.clone()
        } else {
            Rc::new(BasicBlock::default()) // just use any random node
        };
        for (node, child) in self.edges.iter() {
            if let Some(next) = &child[0] {
                let dashed = if next == &sink {
                    "[style=\"dotted\"];"
                } else {
                    ""
                };
                content.push(format!("{} -> {}{}", node.first, next.first, dashed));
            }
            if let Some(cond) = &child[1] {
                content.push(format!(
                    "{} -> {}[arrowhead=\"empty\"];",
                    node.first, cond.first
                ));
            }
        }
        format!("digraph {{\n{}\n}}\n", content.join("\n"))
    }

    /// Saves the current CFG into a Graphviz representation.
    ///
    /// Given a path to file, saves the current CFG as a Graphviz .dot file.
    /// This is equivalent of calling [CFG::to_dot()] and then saving the String content to file.
    pub fn to_file<S: AsRef<Path>>(&self, filename: S) -> Result<(), io::Error> {
        let mut file = File::create(filename)?;
        file.write_all(self.to_dot().as_bytes())
    }

    /// Adds a sink to the current CFG.
    ///
    /// In some cases, a CFG may have multiple nodes without children (like in the case of multiple
    /// return statements). This method merges those nodes by attaching them to a sink. The sink
    /// is recognizable by calling [BasicBlock::is_sink()].
    pub fn add_sink(&self) -> CFG {
        let exit_nodes = self
            .edges
            .iter()
            .filter(|(_, child)| child[0].is_none() && child[1].is_none())
            .count();
        let mut edges = self.edges.clone();
        if exit_nodes > 1 {
            let sink = Rc::new(BasicBlock::new_sink());
            for (_, child) in edges.iter_mut() {
                if child[0].is_none() && child[1].is_none() {
                    child[0] = Some(sink.clone());
                }
            }
            edges.insert(sink, [None, None]);
        }
        CFG {
            root: self.root.clone(),
            edges,
        }
    }

    /// Given a node, returns it's shared reference used internally by the CFG.
    ///
    /// This is useful to avoid some caveats with mutability and borrowing without having to clone
    /// the BasicBlock.
    ///
    /// Returns None if the input node does not belong to this graph.
    pub fn rc(&self, node: &BasicBlock) -> Option<Rc<BasicBlock>> {
        match self.edges.get_key_value(node) {
            None => None,
            Some((rc, _)) => Some(rc.clone()),
        }
    }
}

impl Graph for CFG {
    type Item = BasicBlock;

    fn root(&self) -> Option<&Self::Item> {
        if let Some(root) = &self.root {
            Some(&root)
        } else {
            None
        }
    }

    fn children(&self, node: &Self::Item) -> Option<Vec<&Self::Item>> {
        if let Some(children) = self.edges.get(node) {
            Some(
                children
                    .iter()
                    .flatten()
                    .map(|x| x.as_ref())
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        }
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
        let reachables = cfg.preorder().collect::<HashSet<_>>();
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
        .enumerate()
        .map(|(index, target)| BasicBlock {
            id: index,
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
    use std::rc::Rc;

    //digraph 0->1, 1->2, 2->3
    fn sequence() -> CFG {
        let nodes = (0..)
            .take(4)
            .map(|x| {
                Rc::new(BasicBlock {
                    id: x,
                    first: 0,
                    last: 0,
                })
            })
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
            .map(|x| {
                Rc::new(BasicBlock {
                    id: x,
                    first: 0,
                    last: 0,
                })
            })
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
        let children = cfg.children(&BasicBlock::new_sink());
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
        let children = cfg.children(cfg.root().unwrap());
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
        let children = cfg.children(cfg.root().unwrap());
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
        let cfg_with_sink = cfg.add_sink();
        assert_eq!(cfg.len(), cfg_with_sink.len());
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
