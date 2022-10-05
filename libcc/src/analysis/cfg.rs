use crate::analysis::Graph;
use crate::disasm::radare2::BareCFG;
use crate::disasm::{Architecture, JumpType, Statement, StatementType};
use fnv::FnvHashMap;
use lazy_static::lazy_static;
use parse_int::parse;
use regex::Regex;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io;
use std::io::{ErrorKind, Read, Write};
use std::path::Path;

/// Offset of an artificially created exit node.
pub const SINK_ADDR: u64 = u64::MAX;
/// Offset of an artificially created entry point.
pub const ENTRY_ADDR: u64 = 0;
/// Shape of the root in the exported/imported graphviz dot.
const EXTERN_DOT_ROOT: &str = "rect";
/// Shape of the sink/extended entry point in the exported/imported graphviz dot.
const EXTERN_DOT_SINK: &str = "point";
/// Color of the background in the saved CFG .dot file.
const EXTERN_DOT_BG_COLOUR: &str = "azure";
/// Color of the true edges in the saved CFG .dot file (conditional jumps).
const EXTERN_DOT_TRUE_COLOUR: &str = "forestgreen";
/// Color of the false edges in the saved CFG .dot file (conditional jumps).
const EXTERN_DOT_FALSE_COLOUR: &str = "crimson";
/// Color of the unconditional jumps edges in the saved CFG .dot file.
const EXTERN_DOT_JUMP_COLOUR: &str = "dodgerblue";

/// A Control Flow Graph.
///
/// Struct representing a Control Flow Graph (CFG).
/// This is a graph representation of all the possible execution paths in a function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CFG {
    pub(super) root: Option<BasicBlock>,
    pub(super) edges: HashMap<BasicBlock, Vec<BasicBlock>>,
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
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct BasicBlock {
    /// Offset, in the original code, of the **first** instruction belonging to this basic block.
    pub offset: u64,
    /// Length of the basic block in bytes.
    pub length: u64,
}

impl BasicBlock {
    /// Returns true if the current block is a sink block.
    ///
    /// Sink blocks are added by the [CFG::add_sink()] method.
    pub fn is_sink(&self) -> bool {
        self.length == 0 && self.offset == SINK_ADDR
    }

    /// Returns true if the current block is an artificially added entry point for a CFG.
    ///
    /// **NOTE:** The original entry point **WILL NOT** return true with this method; this method
    /// applies only to the node added with the [CFG::add_entry_point()] method.
    pub fn is_entry_point(&self) -> bool {
        self.length == 0 && self.offset == ENTRY_ADDR
    }

    /// Creates a new sink block.
    fn new_sink() -> BasicBlock {
        BasicBlock {
            offset: SINK_ADDR,
            length: 0,
        }
    }

    /// Creates a new artificial entry point.
    fn new_entry_point() -> BasicBlock {
        BasicBlock {
            offset: ENTRY_ADDR,
            length: 0,
        }
    }
}

impl Display for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.offset)
    }
}

impl Default for BasicBlock {
    fn default() -> Self {
        BasicBlock::new_sink()
    }
}

impl From<BareCFG> for CFG {
    fn from(bare: BareCFG) -> Self {
        let root_addr = bare.root.unwrap_or(0x0);
        let bbs = bare
            .blocks
            .iter()
            .cloned()
            .map(|(first, length)| {
                (
                    first,
                    BasicBlock {
                        offset: first,
                        length,
                    },
                )
            })
            .collect::<HashMap<_, _>>();
        let mut marked = HashSet::with_capacity(bbs.len());
        let mut edges = HashMap::new();
        let mut bare_edges_sorted = bare.edges;
        bare_edges_sorted.sort_unstable(); // first edge is always false, then there is true.
        bare_edges_sorted.dedup(); // remove dups as they may interfere with CFS
        for (src, dst) in bare_edges_sorted {
            let src_bb = bbs.get(&src);
            let dst_bb = bbs.get(&dst);
            if let (Some(src_bb), Some(dst_bb)) = (src_bb, dst_bb) {
                edges
                    .entry(*src_bb)
                    .and_modify(|e: &mut Vec<BasicBlock>| e.push(*dst_bb))
                    .or_insert_with(|| vec![*dst_bb]);
                marked.insert(src_bb);
            }
        }
        // insert terminating nodes
        bbs.iter()
            .filter(|(_, val)| !marked.contains(*val))
            .for_each(|(_, val)| {
                edges.insert(*val, Vec::with_capacity(0));
            });
        let mut root = bbs
            .iter()
            .map(|(_, bb)| bb)
            .find(|&bb| bb.offset == root_addr)
            .cloned();
        if root.is_none() && !bbs.is_empty() {
            // if the root written in the BareCFG does not exists (weird), pick the lowest offset
            root = bbs.iter().map(|(_, bb)| bb).min().cloned();
        }
        CFG { root, edges }
    }
}

impl CFG {
    /// Creates a new CFG from a list of statements.
    ///
    /// Given a list of statements, the function end offset and a source architectures, builds the
    /// CFG for that list.
    /// The list of statements is presented as slice.
    ///
    /// The function end offset is needed to correctly calculate the last basic block length. Other
    /// than that, any number can be passed.
    ///
    /// The newly returned CFG will not contain a sink and will contain only reachable nodes
    /// (thus eliminating indirect jumps).
    /// One should use [CFG::add_sink()] to refine the CFG.
    /// # Examples
    /// Basic usage:
    /// ```
    /// # use bcc::analysis::{Graph, CFG};
    /// # use bcc::disasm::{ArchX86, Statement, StatementType};
    /// let stmts = vec![
    ///     Statement::new(0x38, StatementType::CMP, "cmp dword [var_4h], 0"),
    ///     Statement::new(0x3C, StatementType::CJMP, "jle 0x45"),
    ///     Statement::new(0x3E, StatementType::MOV, "mov eax, 0"),
    ///     Statement::new(0x43, StatementType::JMP, "jmp 0x4a"),
    ///     Statement::new(0x45, StatementType::MOV, "mov eax, 1"),
    ///     Statement::new(0x4A, StatementType::RET, "ret"),
    /// ];
    /// let arch = ArchX86::new_amd64();
    /// let cfg = CFG::new(&stmts, 0x4B, &arch);
    ///
    /// assert_eq!(cfg.len(), 4);
    /// ```
    pub fn new(stmts: &[Statement], fn_end: u64, arch: &dyn Architecture) -> CFG {
        CFG::from(to_bare_cfg(stmts, fn_end, arch))
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
                children.first()
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
                if children.len() >= 2 {
                    Some(&children[1])
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
        for (node, children) in self.edges.iter() {
            let node_id = node.offset;
            let shape = if node.is_entry_point() || node.is_sink() {
                format!(",shape=\"{}\"", EXTERN_DOT_SINK)
            } else if Some(node) == self.root.as_ref() {
                format!(",shape=\"{}\"", EXTERN_DOT_ROOT)
            } else {
                String::new()
            };
            nodes_string.push(format!(
                "{}[comment=\"({},{})\"{}];",
                node.offset, node.offset, node.length, shape
            ));
            match children.len() {
                0 => {}
                // 1 falls into the _ case
                2 => {
                    let dst_false = &children[0].offset;
                    let dst_true = &children[1].offset;
                    edges_string.push(format!(
                        "{}->{}[color=\"{}\"];",
                        node_id, dst_false, EXTERN_DOT_FALSE_COLOUR
                    ));
                    edges_string.push(format!(
                        "{}->{}[color=\"{}\"];",
                        node_id, dst_true, EXTERN_DOT_TRUE_COLOUR
                    ));
                }
                _ => {
                    for child in children.iter() {
                        let dst = child.offset;
                        edges_string.push(format!(
                            "{}->{}[color=\"{}\"];",
                            node_id, dst, EXTERN_DOT_JUMP_COLOUR
                        ));
                    }
                }
            }
        }
        format!(
            "digraph{{\ngraph[bgcolor={},fontsize=8,splines=\"ortho\"];\n{}\n{}\n{}\n}}\n",
            EXTERN_DOT_BG_COLOUR,
            "node[fillcolor=gray,style=filled,shape=box];\nedge[arrowhead=normal];\n",
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
            let mut edges_ids = HashMap::new();
            let nodes_re_str = format!(
                r#"(\d+)\[comment="\((\d+),(\d+)\)"(?:,shape="({}|{})")?];"#,
                EXTERN_DOT_SINK, EXTERN_DOT_ROOT
            );
            let mut root = None;
            let node_re = Regex::new(&nodes_re_str).unwrap();
            lazy_static! {
                static ref DOT_EDGES_RE: Regex = Regex::new(r#"(\d+)->(\d+)(?:\[.*];)?"#).unwrap();
            }
            while let Some(line) = lines.pop() {
                if let Some(cap) = node_re.captures(line) {
                    let id = cap.get(1).unwrap().as_str().parse::<usize>()?;
                    let offset = cap.get(2).unwrap().as_str().parse::<u64>()?;
                    let length = cap.get(3).unwrap().as_str().parse::<u64>()?;
                    let node = BasicBlock { offset, length };
                    if let Some(shape) = cap.get(4) {
                        if shape.as_str() == EXTERN_DOT_ROOT {
                            root = Some(node);
                        }
                    }
                    nodes.insert(id, node);
                } else if let Some(cap) = DOT_EDGES_RE.captures(line) {
                    let from = cap.get(1).unwrap().as_str().parse::<usize>()?;
                    let to = cap.get(2).unwrap().as_str().parse::<usize>()?;
                    edges_ids
                        .entry(from)
                        .and_modify(|e: &mut Vec<usize>| e.push(to))
                        .or_insert_with(|| vec![to]);
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
            for (src, dst_vec) in edges_ids {
                let src_node = *nodes.get(&src).ok_or_else(parse_err)?;
                for dst in dst_vec {
                    let dst_node = *nodes.get(&dst).ok_or_else(parse_err)?;
                    edges
                        .entry(src_node)
                        .and_modify(|e: &mut Vec<BasicBlock>| e.push(dst_node))
                        .or_insert_with(|| vec![dst_node]);
                }
            }
            // add the exits
            let exits = nodes
                .into_iter()
                .filter(|(_, node)| !edges.contains_key(node))
                .map(|(_, node)| node)
                .collect::<Vec<_>>();
            for exit in exits {
                edges.insert(exit, Vec::with_capacity(0));
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
            .filter(|(_, child)| child.is_empty())
            .count();
        if exit_nodes > 1 {
            let sink = BasicBlock::new_sink();
            for (_, child) in self.edges.iter_mut() {
                if child.is_empty() {
                    child.push(sink);
                }
            }
            self.edges.insert(sink, Vec::with_capacity(0));
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
        if let Some(oep) = &self.root {
            let oep_has_preds = self
                .edges
                .iter()
                .flat_map(|(_, edge)| edge)
                .any(|x| x == oep);
            if oep_has_preds {
                let eep = BasicBlock::new_entry_point();
                self.edges.insert(eep, vec![self.root.take().unwrap()]);
                self.root = Some(eep);
            }
        }
        self
    }
}

impl Graph for CFG {
    type Item = BasicBlock;

    fn root(&self) -> Option<&Self::Item> {
        self.root.as_ref()
    }

    fn neighbours(&self, node: &Self::Item) -> &[Self::Item] {
        if let Some(n) = self.edges.get(node) {
            &n[..]
        } else {
            &[]
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
    let empty_stmt = Statement::new(0x0, StatementType::UNK, "");
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

// actual cfg building
fn to_bare_cfg(stmts: &[Statement], fn_end: u64, arch: &dyn Architecture) -> BareCFG {
    let tgmap = get_targets(stmts, arch);
    // This target is used for a strictly lower bound.
    let mut nodes = Vec::with_capacity(tgmap.targets.len());
    // the capacity here is not perfect but it's a good estimation
    let mut edges = Vec::with_capacity(
        tgmap.targets.len() + tgmap.srcs_cond.len() - tgmap.deadend_uncond.len(),
    );
    // create nodes
    let mut nodes_iter = tgmap.targets.iter().peekable();
    while let Some(current) = nodes_iter.next() {
        let next_target = *nodes_iter.peek().unwrap_or(&&fn_end);
        nodes.push((*current, *next_target - *current));
        if *next_target != fn_end {
            edges.push((*current, *next_target));
        }
    }
    // remove node->next_node edges where node contains a jump or a return
    let nodes_ordered = nodes
        .iter()
        .map(|&(first, _)| first)
        .collect::<BTreeSet<_>>();
    let jump_blocks = tgmap
        .srcs_cond
        .keys()
        .chain(tgmap.srcs_uncond.keys())
        .map(|src| *nodes_ordered.range(..=src).next_back().unwrap())
        .collect::<HashSet<_>>();
    let return_blocks = tgmap
        .deadend_cond
        .iter()
        .chain(tgmap.deadend_uncond.iter())
        .map(|src| *nodes_ordered.range(..=src).next_back().unwrap())
        .collect::<HashSet<_>>();
    edges = edges
        .into_iter()
        .filter(|(src, _)| !jump_blocks.contains(src))
        .filter(|(src, _)| !return_blocks.contains(src))
        .collect();
    // add jump edges
    for (off_src, off_dst) in tgmap.srcs_uncond {
        let src_bb = *nodes_ordered.range(..=off_src).next_back().unwrap();
        edges.push((src_bb, off_dst));
    }
    for (off_src, off_dst) in tgmap.srcs_cond {
        let src_bb = *nodes_ordered.range(..=off_src).next_back().unwrap();
        let next_dst = *nodes_ordered.range(off_src + 1..).next().unwrap_or(&fn_end);
        edges.push((src_bb, next_dst)); // first the next stmt
        edges.push((src_bb, off_dst)); // then the cond stmt
    }

    // find root
    let mut preds_no = nodes
        .iter()
        .map(|&(first, _)| (first, 0_u32))
        .collect::<FnvHashMap<_, _>>();
    edges.iter().for_each(|&(_, dst)| {
        preds_no.entry(dst).and_modify(|e| *e += 1);
    });
    let mut root = preds_no
        .iter()
        .filter(|(_, no)| **no == 0)
        .map(|(bb, _)| *bb)
        .min();
    if !preds_no.is_empty() && root.is_none() {
        // every preds has at least 1 entry, pick the lowest offset
        root = nodes.iter().min().map(|(first, _)| first).copied();
    }
    BareCFG {
        root,
        blocks: nodes,
        edges,
    }
}

#[cfg(test)]
mod tests {
    use crate::analysis::{BasicBlock, Graph, CFG};
    use crate::disasm::radare2::BareCFG;
    use crate::disasm::{ArchX86, Statement, StatementType};
    use maplit::hashmap;
    use std::collections::{HashMap, HashSet};
    use std::error::Error;
    use std::io::{Read, Seek, SeekFrom, Write};
    use tempfile::tempfile;

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
                .filter(|(node, _child)| reachables.contains(node))
                .collect::<HashMap<_, _>>();
            CFG {
                root: cfg.root,
                edges,
            }
        } else {
            cfg
        }
    }

    //digraph 0->1, 1->2, 2->3
    fn sequence() -> CFG {
        let nodes = (0..)
            .take(4)
            .map(|x| BasicBlock {
                offset: x,
                length: 1,
            })
            .collect::<Vec<_>>();
        let edges = hashmap![
            nodes[0] => vec![nodes[1]],
            nodes[1] => vec![nodes[2]],
            nodes[2] => vec![nodes[3]],
            nodes[3] => vec![],
        ];
        CFG {
            root: Some(nodes[0]),
            edges,
        }
    }

    //digraph 0->1, 2->3 (forced to skip the build_cfg otherwise reachable() will delete 2 and 3)
    fn two_sequences() -> CFG {
        let nodes = (0..)
            .take(4)
            .map(|x| BasicBlock {
                offset: x,
                length: 1,
            })
            .collect::<Vec<_>>();
        let edges = hashmap![
            nodes[0] => vec![nodes[1]],
            nodes[1] => vec![],
            nodes[2] => vec![nodes[3]],
            nodes[3] => vec![],
        ];
        CFG {
            root: Some(nodes[0]),
            edges,
        }
    }

    #[test]
    fn from_bare_cfg() {
        //expected
        let nodes = [
            BasicBlock {
                offset: 0x1000,
                length: 20,
            },
            BasicBlock {
                offset: 0x1014,
                length: 2,
            },
            BasicBlock {
                offset: 0x1016,
                length: 5,
            },
        ];
        let edges = hashmap![
            nodes[0] => vec![nodes[1], nodes[2]],
            nodes[1] => vec![nodes[2]],
            nodes[2] => vec![],
        ];
        let expected = CFG {
            root: Some(
                [
                    BasicBlock {
                        offset: 0x1000,
                        length: 20,
                    },
                    BasicBlock {
                        offset: 0x1014,
                        length: 2,
                    },
                    BasicBlock {
                        offset: 0x1016,
                        length: 5,
                    },
                ][0],
            ),
            edges,
        };
        //conversion
        let bare = BareCFG {
            root: Some(0x1000),
            blocks: vec![(0x1000, 20), (0x1014, 2), (0x1016, 5)],
            edges: vec![(0x1000, 0x1016), (0x1000, 0x1014), (0x1014, 0x1016)],
        };
        let cfg = CFG::from(bare);
        assert_eq!(cfg, expected);
    }

    #[test]
    fn root_empty() {
        let stmts = Vec::new();
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x0, &arch);
        assert!(cfg.root().is_none());
    }

    #[test]
    fn root() {
        let stmts = vec![
            Statement::new(0x61C, StatementType::MOV, "mov eax, 5"),
            Statement::new(0x624, StatementType::RET, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x625, &arch);
        assert!(cfg.root().is_some());
        assert_eq!(cfg.root().unwrap().offset, 0x61C);
    }

    #[test]
    fn get_children_nonexisting() {
        let stmts = Vec::new();
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x0, &arch);
        let children = cfg.neighbours(&BasicBlock::new_sink());
        assert!(children.is_empty())
    }

    #[test]
    fn get_children_existing_empty() {
        let stmts = vec![
            Statement::new(0x61C, StatementType::MOV, "mov eax, 5"),
            Statement::new(0x624, StatementType::RET, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x625, &arch);
        let children = cfg.neighbours(cfg.root().unwrap());
        assert!(children.is_empty());
    }

    #[test]
    fn get_children_existing() {
        let stmts = vec![
            Statement::new(0x610, StatementType::CMP, "test edi, edi"),
            Statement::new(0x612, StatementType::CJMP, "je 0x618"),
            Statement::new(0x614, StatementType::MOV, "mov eax, 6"),
            Statement::new(0x618, StatementType::RET, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x619, &arch);
        let children = cfg.neighbours(cfg.root().unwrap());
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn len() {
        let stmts = vec![
            Statement::new(0x610, StatementType::CMP, "test edi, edi"),
            Statement::new(0x612, StatementType::CJMP, "je 0x618"),
            Statement::new(0x614, StatementType::MOV, "mov eax, 6"),
            Statement::new(0x618, StatementType::RET, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x619, &arch);
        assert_eq!(cfg.len(), 3);
    }

    #[test]
    fn build_cfg_empty() {
        let stmts = Vec::new();
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x0, &arch);
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
            Statement::new(0x610, StatementType::CMP, "test edi, edi"), //0
            Statement::new(0x612, StatementType::CJMP, "je 0x620"),     //0
            Statement::new(0x614, StatementType::CMP, "test esi, esi"), //1
            Statement::new(0x616, StatementType::MOV, "mov eax, 5"),    //1
            Statement::new(0x61b, StatementType::CJMP, "je 0x620"),     //1
            Statement::new(0x61d, StatementType::RET, "ret"),           //2
            Statement::new(0x620, StatementType::MOV, "mov eax, 6"),    //3
            Statement::new(0x625, StatementType::RET, "ret"),           //3
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x626, &arch);
        assert!(!cfg.is_empty());
        assert_eq!(cfg.len(), 4);
        let node0 = cfg.root();
        let node1 = cfg.next(node0);
        let node2 = cfg.next(node1);
        let node3 = cfg.cond(node1);
        assert_eq!(node0.unwrap().offset, 0x610);
        assert_eq!(node1.unwrap().offset, 0x614);
        assert_eq!(node2.unwrap().offset, 0x61D);
        assert_eq!(node3.unwrap().offset, 0x620);
        assert!(cfg.next(node2).is_none());
        assert!(cfg.cond(node2).is_none());
        assert!(cfg.next(node3).is_none());
        assert!(cfg.cond(node3).is_none());
    }

    #[test]
    fn build_cfg_unconditional_jumps() {
        let stmts = vec![
            Statement::new(0x61E, StatementType::PUSH, "push rbp"), //0
            Statement::new(0x61F, StatementType::MOV, "mov rbp, rsp"), //0
            Statement::new(0x622, StatementType::MOV, "mov dword [var_4h], edi"), //0
            Statement::new(0x625, StatementType::MOV, "mov dword [var_8h], esi"), //0
            Statement::new(0x628, StatementType::CMP, "cmp dword [var_4h], 5"), //0
            Statement::new(0x62C, StatementType::CJMP, "jne 0x633"), //0
            Statement::new(0x62E, StatementType::MOV, "mov eax, dword [var_8h]"), //1
            Statement::new(0x631, StatementType::JMP, "jmp 0x638"), //1
            Statement::new(0x633, StatementType::MOV, "mov eax, 6"), //2
            Statement::new(0x638, StatementType::POP, "pop rbp"),   //3
            Statement::new(0x639, StatementType::RET, "ret"),       //3
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x640, &arch);
        assert_eq!(cfg.len(), 4);
        let node0 = cfg.root();
        let node1 = cfg.next(node0);
        let node2 = cfg.cond(node0);
        let node3 = cfg.next(node1);
        assert_eq!(node0.unwrap().offset, 0x61E);
        assert_eq!(node1.unwrap().offset, 0x62E);
        assert_eq!(node2.unwrap().offset, 0x633);
        assert_eq!(node3.unwrap().offset, 0x638);
        assert!(cfg.cond(node1).is_none());
        assert!(cfg.cond(node2).is_none());
        assert!(cfg.next(node3).is_none());
        assert!(cfg.cond(node3).is_none());
    }

    #[test]
    fn build_cfg_long_unconditional_jump() {
        // this is crafted so offsets are completely random
        let stmts = vec![
            Statement::new(0x610, StatementType::CMP, "test edi, edi"), //0
            Statement::new(0x611, StatementType::CMP, "je 0x613"),      //0
            Statement::new(0x612, StatementType::JMP, "jmp 0xFFFFFFFFFFFFFFFC"), //1
            Statement::new(0x613, StatementType::JMP, "jmp 0x600"),     //2
            Statement::new(0x614, StatementType::JMP, "jmp 0x615"),     //3
            Statement::new(0x615, StatementType::RET, "ret"),           //4
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x616, &arch);
        assert_eq!(cfg.len(), 5);
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
            Statement::new(0x610, StatementType::CMP, "test edi, edi"), //0
            Statement::new(0x614, StatementType::CMP, "je 0x628"),      //0
            Statement::new(0x618, StatementType::CMP, "test esi, esi"), //1
            Statement::new(0x61C, StatementType::MOV, "mov eax, 5"),    //1
            Statement::new(0x620, StatementType::CMP, "je 0x628"),      //1
            Statement::new(0x624, StatementType::RET, "ret"),           //2
            Statement::new(0x628, StatementType::MOV, "mov eax, 6"),    //3
            Statement::new(0x62C, StatementType::RET, "ret"),           //3
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x630, &arch);
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
        assert_eq!(node0.unwrap().offset, 0x610);
        assert_eq!(node0.unwrap().length, 8);
        assert_eq!(node1.unwrap().offset, 0x618);
        assert_eq!(node1.unwrap().length, 12);
        assert_eq!(node2.unwrap().offset, 0x624);
        assert_eq!(node2.unwrap().length, 4);
        assert_eq!(node3.unwrap().offset, 0x628);
        assert_eq!(node3.unwrap().length, 8);
    }

    #[test]
    fn build_cfg_offset_64bit() {
        let stmts = vec![
            Statement::new(0x3FD1A7EF534, StatementType::JMP, "jmp 0x3FD1A7EF538"),
            Statement::new(0x3FD1A7EF538, StatementType::ADD, "incl eax"),
            Statement::new(0x3FD1A7EF53C, StatementType::MOV, "mov ebx, [ebp+20]"),
            Statement::new(0x3FD1A7EF540, StatementType::CMP, "cmp eax, ebx"),
            Statement::new(0x3FD1A7EF544, StatementType::CJMP, "je 0x3FD1A7EF558"),
            Statement::new(0x3FD1A7EF548, StatementType::MOV, "mov ecx, [ebp+20]"),
            Statement::new(0x3FD1A7EF54C, StatementType::SUB, "decl ecx"),
            Statement::new(0x3FD1A7EF550, StatementType::MOV, "mov [ebp+20], ecx"),
            Statement::new(0x3FD1A7EF554, StatementType::JMP, "jmp 0x3FD1A7EF538"),
            Statement::new(0x3FD1A7EF558, StatementType::CMP, "test eax, eax"),
            Statement::new(0x3FD1A7EF55C, StatementType::MOV, "mov eax, 0"),
            Statement::new(0x3FD1A7EF560, StatementType::CJMP, "je 0x3FD1A7EF568"),
            Statement::new(0x3FD1A7EF564, StatementType::MOV, "mov eax, 1"),
            Statement::new(0x3FD1A7EF568, StatementType::RET, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x3FD1A7EF56C, &arch);
        assert_eq!(cfg.len(), 6);
    }

    #[test]
    fn save_and_retrieve_empty() -> Result<(), Box<dyn Error>> {
        let stmts = Vec::new();
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x0, &arch);
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
            Statement::new(0x61E, StatementType::PUSH, "push rbp"), //0
            Statement::new(0x622, StatementType::MOV, "mov dword [var_4h], edi"), //0
            Statement::new(0x628, StatementType::CMP, "cmp dword [var_4h], 5"), //0
            Statement::new(0x62C, StatementType::CJMP, "jne 0x633"), //0
            Statement::new(0x62E, StatementType::MOV, "mov eax, dword [var_8h]"), //1
            Statement::new(0x631, StatementType::JMP, "jmp 0x638"), //1
            Statement::new(0x633, StatementType::MOV, "mov eax, 6"), //2
            Statement::new(0x638, StatementType::POP, "pop rbp"),   //3
            Statement::new(0x639, StatementType::RET, "ret"),       //3
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x640, &arch);
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
            Statement::new(0x61E, StatementType::PUSH, "push rbp"), //0
            Statement::new(0x622, StatementType::MOV, "mov dword [var_4h], edi"), //0
            Statement::new(0x62C, StatementType::CJMP, "jne 0x638"), //0
            Statement::new(0x62E, StatementType::RET, "ret"),       //1
            Statement::new(0x638, StatementType::POP, "pop rbp"),   //2
            Statement::new(0x639, StatementType::RET, "ret"),       //2
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x640, &arch);
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
        let cfg = CFG::new(&stmts, 0x0, &arch);
        let cfg_with_sink = cfg.add_sink();
        assert!(cfg_with_sink.is_empty());
    }

    #[test]
    fn add_sink_necessary() {
        let stmts = vec![
            Statement::new(0x61C, StatementType::MOV, "mov eax, 5"),
            Statement::new(0x620, StatementType::CJMP, "je 0x628"),
            Statement::new(0x624, StatementType::RET, "ret"),
            Statement::new(0x628, StatementType::MOV, "mov eax, 6"),
            Statement::new(0x62C, StatementType::RET, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x630, &arch);
        assert_eq!(cfg.len(), 3);
        let cfg_with_sink = cfg.add_sink();
        assert_eq!(cfg_with_sink.len(), 4);
    }

    #[test]
    fn add_sink_unnecessary() {
        let stmts = vec![
            Statement::new(0x61C, StatementType::MOV, "mov eax, 5"),
            Statement::new(0x624, StatementType::RET, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x625, &arch);
        let cfg_with_sink = cfg.clone().add_sink();
        assert_eq!(cfg.len(), cfg_with_sink.len());
    }

    #[test]
    fn add_extra_entry_point() {
        let stmts = vec![
            Statement::new(0x61C, StatementType::MOV, "mov eax, 5"),
            Statement::new(0x620, StatementType::CJMP, "jne 0x61c"),
            Statement::new(0x624, StatementType::RET, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x625, &arch);
        assert_eq!(cfg.len(), 2);
        let cfg_with_eep = cfg.clone().add_entry_point();
        assert_eq!(cfg_with_eep.len(), 3);
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
            Statement::new(0x61C, StatementType::MOV, "mov eax, 5"),
            Statement::new(0x624, StatementType::RET, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x625, &arch);
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

    #[test]
    fn from_bare_cfg_root_midway() {
        //root does not have the min address
        let bcfg = BareCFG {
            root: Some(418580),
            blocks: vec![
                (418538, 10),
                (418548, 32),
                (418580, 16),
                (418596, 16),
                (418712, 2),
            ],
            edges: vec![
                (418538, 418712),
                (418538, 418548),
                (418580, 418538),
                (418580, 418596),
                (418596, 418538),
            ],
        };
        let cfg = CFG::from(bcfg);
        assert_eq!(cfg.bfs().count(), 5);
    }

    #[test]
    fn new_root_midway() {
        let stmts = vec![
            Statement::new(0x538, StatementType::CJMP, "jne 0x712"),
            Statement::new(0x548, StatementType::JMP, "jmp 0x712"),
            Statement::new(0x580, StatementType::CJMP, "jne 0x538"),
            Statement::new(0x596, StatementType::JMP, "jmp 0x538"),
            Statement::new(0x712, StatementType::RET, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x713, &arch);
        assert_eq!(cfg.len(), 5);
    }

    #[test]
    fn from_bare_cfg_everybody_has_preds() {
        // This test is now useless, previously I estimated the entry point for bareCFG.
        // leaving it here because it does not hurts
        let bcfg = BareCFG {
            root: Some(0),
            blocks: vec![(0, 1), (2, 1)],
            edges: vec![(0, 2), (2, 0)],
        };
        let cfg = CFG::from(bcfg);
        assert!(cfg.root().is_some());
        assert_eq!(cfg.root().unwrap().offset, 0);
    }

    #[test]
    fn new_everybody_has_preds() {
        let stmts = vec![
            Statement::new(0x61C, StatementType::JMP, "jmp 0x620"),
            Statement::new(0x620, StatementType::JMP, "jmp 0x61c"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, 0x62C, &arch);
        assert!(cfg.root().is_some());
        assert_eq!(cfg.root().unwrap().offset, 0x61C);
    }
}
