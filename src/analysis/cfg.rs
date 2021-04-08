use crate::disasm::{Architecture, JumpType, Statement};
use fnv::{FnvHashMap, FnvHashSet};
use parse_int::parse;
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::Write;
use std::ops::Index;
use std::path::Path;

/// A Control Flow Graph.
///
/// Struct representing a Control Flow Graph (CFG).
/// This is a graph representation of all the possible execution paths in a function.
#[derive(Debug, Clone)]
pub struct CFG {
    nodes: Vec<BasicBlock>,
    edges: Vec<[Option<usize>; 2]>,
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
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Numerical integer representing an unique identifier for this block.
    pub id: usize,
    /// Offset in the original code where this basic block begins.
    pub first: u64,
    /// Offset in the original code where this basic block ends.
    pub last: u64,
}

impl Display for CFG {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let has_sink = self
            .nodes
            .iter()
            .filter(|x| x.first == 0x0 && x.last == 0x0)
            .count()
            == 1;
        if has_sink {
            write!(f, "CFG({}+1, {})", self.nodes.len() - 1, self.edges.len())
        } else {
            write!(f, "CFG({}, {})", self.nodes.len(), self.edges.len())
        }
    }
}

/// An iterator for contents of a CFG.
pub struct CFGIter<'a> {
    stack: Vec<&'a BasicBlock>,
}

impl<'a> Iterator for CFGIter<'a> {
    type Item = &'a BasicBlock;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop()
    }
}

impl CFG {
    /// Creates a new CFG from a list of statements.
    ///
    /// Given a list of statements and a source architectures, builds the CFG for that list.
    /// The list of statements is presented as slice.
    ///
    /// The newly returned CFG will not contain a sink and may present unreachable nodes (mainly due
    /// to indirect jumps). One should use [CFG::add_sink()] or [CFG::reachable()] to refine the
    /// CFG.
    /// # Examples
    /// Basic usage:
    /// ```
    /// use bcc::analysis::CFG;
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

    /// Visits the CFG nodes in pre-order.
    ///
    /// Returns an iterator visiting every node reachable from the CFG root using a depth-first
    /// pre-order.
    pub fn preorder(&self) -> CFGIter {
        if !self.is_empty() {
            let mut buffer = vec![0];
            let mut retval = Vec::with_capacity(self.nodes.len());
            let mut marked = vec![false; self.nodes.len()];
            while let Some(current_id) = buffer.pop() {
                let current = &self.nodes[current_id];
                retval.push(current);
                marked[current_id] = true;
                let children = self.edges[current_id];
                for maybe_child in children.iter().rev() {
                    if let Some(child_id) = maybe_child {
                        if !marked[*child_id] {
                            marked[*child_id] = true;
                            buffer.push(*child_id);
                        }
                    }
                }
            }
            retval.reverse();
            CFGIter { stack: retval }
        } else {
            CFGIter { stack: Vec::new() }
        }
    }

    /// Visits the CFG nodes in post-order.
    ///
    /// Returns an iterator visiting every node reachable from the CFG root using a depth-first
    /// post-order.
    pub fn postorder(&self) -> CFGIter {
        if !self.is_empty() {
            let mut buffer = vec![0];
            let mut retval = Vec::with_capacity(self.nodes.len());
            let mut marked = vec![false; self.nodes.len()];
            while let Some(current_id) = buffer.last() {
                let mut to_push = Vec::new();
                marked[*current_id] = true;
                let children = self.edges[*current_id];
                for maybe_child in children.iter().rev() {
                    if let Some(child_id) = maybe_child {
                        if !marked[*child_id] {
                            marked[*child_id] = true;
                            to_push.push(*child_id);
                        }
                    }
                }
                if to_push.is_empty() {
                    let current = &self.nodes[buffer.pop().unwrap()];
                    retval.push(current);
                } else {
                    buffer.append(&mut to_push);
                }
            }
            retval.reverse();
            CFGIter { stack: retval }
        } else {
            CFGIter { stack: Vec::new() }
        }
    }

    /// Returns the next basic block.
    ///
    /// Given a basic block id, returns the basic block id of its follower.
    /// This means the target of an unconditional jump or the next block in case the current block
    /// ends with a conditional jump.
    ///
    /// Returns None if there is no next block.
    pub fn next(&self, id: usize) -> Option<usize> {
        self.edges[id][0]
    }

    /// Returns the conditional basic block.
    ///
    /// Given a basic block id, returns the basic block id of the conditional jump target.
    /// If the current block does not end with a conditional jump, None is returned.
    pub fn cond(&self, id: usize) -> Option<usize> {
        self.edges[id][1]
    }

    /// Returns the number of blocks in this CFG.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns true if the current CFG is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Converts the current CFG into a Graphviz dot representation.
    pub fn to_dot(&self) -> String {
        let mut content = Vec::new();
        let sink_iter = self
            .nodes
            .iter()
            .filter(|x| x.first == 0x0 && x.last == 0x0)
            .last();
        let sink = if let Some(sink_node) = sink_iter {
            let id = sink_node.id;
            content.push(format!("{}[style=\"dotted\"];", id));
            id
        } else {
            usize::MAX
        };
        for edge in self.edges.iter().enumerate() {
            if let Some(next) = edge.1[0] {
                let dashed = if next == sink {
                    "[style=\"dotted\"];"
                } else {
                    ""
                };
                content.push(format!("{} -> {}{}", edge.0, next, dashed));
            }
            if let Some(cond) = edge.1[1] {
                content.push(format!("{} -> {}[arrowhead=\"empty\"];", edge.0, cond));
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
    /// is recognizable by having both starting and ending offsets equal to 0.
    pub fn add_sink(&self) -> CFG {
        let exit_nodes = self
            .edges
            .iter()
            .enumerate()
            .filter(|x| x.1[0].is_none() && x.1[1].is_none())
            .map(|x| x.0)
            .collect::<BTreeSet<_>>();
        let mut nodes = self.nodes.clone();
        let mut edges = self.edges.clone();
        if exit_nodes.len() > 1 {
            let sink = BasicBlock {
                id: nodes.len(),
                first: 0x0,
                last: 0x0,
            };
            nodes.push(sink);
            edges = edges
                .into_iter()
                .map(|x| {
                    if x[0].is_none() && x[1].is_none() {
                        [Some(nodes.len() - 1), None]
                    } else {
                        x
                    }
                })
                .collect();
        }
        CFG { nodes, edges }
    }

    /// Removes unreachable nodes.
    ///
    /// Removes nodes that are not reachable from the CFG root by any path. These nodes are usually
    /// created when there are indirect jumps in the original statement list.
    pub fn reachable(&self) -> CFG {
        if self.nodes.len() > 1 {
            let mut new_nodes = vec![self.nodes[0].clone()];
            let mut new_edges = vec![self.edges[0]];
            let reachables = self
                .preorder()
                .enumerate()
                .map(|x| x.0)
                .collect::<FnvHashSet<_>>();
            let mut skipped = vec![0; self.nodes.len()];
            for index in 1..self.nodes.len() {
                skipped[index] = skipped[index - 1];
                if reachables.contains(&index) {
                    new_nodes.push(self.nodes[index].clone());
                    new_edges.push(self.edges[index]);
                } else {
                    skipped[index] += 1;
                }
            }
            for edge in new_edges.iter_mut() {
                if let Some(target) = edge[0] {
                    edge[0] = Some(target - skipped[target])
                }
                if let Some(target) = edge[1] {
                    edge[1] = Some(target - skipped[target])
                }
            }
            CFG {
                nodes: new_nodes,
                edges: new_edges,
            }
        } else {
            self.clone()
        }
    }
}

impl Index<usize> for CFG {
    type Output = BasicBlock;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
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
    let mut nodes = tgmap
        .targets
        .iter()
        .enumerate()
        .map(|x| BasicBlock {
            id: x.0,
            first: *x.1,
            last: 0,
        })
        .collect::<Vec<_>>();
    // fill ending statement
    let mut nodes_iter = tgmap.targets.iter().enumerate().peekable();
    while let Some(target) = nodes_iter.next() {
        let next_target = *nodes_iter.peek().unwrap_or(&(0, &function_over)).1;
        let last_stmt = *all_offsets.range(..next_target).last().unwrap_or(&0);
        nodes[target.0].last = last_stmt;
    }
    let mut edges = if !tgmap.targets.is_empty() {
        let mut edges = (1..)
            .take(tgmap.targets.len() - 1)
            .map(|next| [Some(next), None])
            .collect::<Vec<_>>();
        edges.push([None, None]);
        edges
    } else {
        Vec::new()
    };
    // map every offset to the block id
    let id_map = tgmap
        .targets
        .iter()
        .enumerate()
        .map(|x| (*x.1, x.0))
        .collect::<FnvHashMap<_, _>>();
    for jmp in tgmap.srcs_uncond {
        let src_id = resolve_bb_id(jmp.0, &id_map, &tgmap.targets);
        let dst_id = resolve_bb_id(jmp.1, &id_map, &tgmap.targets);
        edges[src_id][0] = Some(dst_id);
    }
    for jmp in tgmap.srcs_cond {
        let src_id = resolve_bb_id(jmp.0, &id_map, &tgmap.targets);
        let dst_id = resolve_bb_id(jmp.1, &id_map, &tgmap.targets);
        edges[src_id][1] = Some(dst_id);
    }
    for ret in tgmap.deadend_uncond {
        let src_id = resolve_bb_id(ret, &id_map, &tgmap.targets);
        edges[src_id][0] = None;
    }
    for ret in tgmap.deadend_cond {
        let src_id = resolve_bb_id(ret, &id_map, &tgmap.targets);
        edges[src_id][1] = None;
    }
    CFG { nodes, edges }
}

#[cfg(test)]
mod tests {
    use crate::analysis::{BasicBlock, CFG};
    use crate::disasm::{ArchX86, Statement};

    #[test]
    fn preorder_empty() {
        let cfg = CFG {
            nodes: Vec::new(),
            edges: Vec::new(),
        };
        let order = cfg.preorder();
        assert_eq!(order.count(), 0)
    }

    #[test]
    fn preorder() {
        let nodes = (0..)
            .take(7)
            .map(|x| BasicBlock {
                id: x,
                first: 0,
                last: 0,
            })
            .collect();
        let edges = vec![
            [Some(1), Some(2)],
            [Some(6), None],
            [Some(3), Some(4)],
            [Some(5), None],
            [Some(5), None],
            [Some(6), None],
            [None, None],
        ];
        let cfg = CFG { nodes, edges };
        let expected = vec![0, 1, 6, 2, 3, 5, 4];
        for (index, val) in cfg.preorder().enumerate() {
            assert_eq!(val.id, expected[index]);
        }
    }

    #[test]
    fn postorder_empty() {
        let cfg = CFG {
            nodes: Vec::new(),
            edges: Vec::new(),
        };
        let order = cfg.postorder();
        assert_eq!(order.count(), 0)
    }

    #[test]
    fn postorder() {
        let nodes = (0..)
            .take(7)
            .map(|x| BasicBlock {
                id: x,
                first: 0,
                last: 0,
            })
            .collect();
        let edges = vec![
            [Some(1), Some(2)],
            [Some(6), None],
            [Some(3), Some(4)],
            [Some(5), None],
            [Some(5), None],
            [Some(6), None],
            [None, None],
        ];
        let cfg = CFG { nodes, edges };
        let expected = vec![6, 1, 5, 3, 4, 2, 0];
        for (index, val) in cfg.postorder().enumerate() {
            assert_eq!(val.id, expected[index]);
        }
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
    fn build_cfg_conditional_jumps() {
        let stmts = vec![
            Statement::new(0x610, "test edi, edi"),
            Statement::new(0x612, "je 0x620"),
            Statement::new(0x614, "test esi, esi"),
            Statement::new(0x616, "mov eax, 5"),
            Statement::new(0x61b, "je 0x620"),
            Statement::new(0x61d, "ret"),
            Statement::new(0x620, "mov eax, 6"),
            Statement::new(0x625, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        assert!(!cfg.is_empty());
        assert_eq!(cfg.len(), 4);
        assert_eq!(cfg.next(0), Some(1));
        assert_eq!(cfg.cond(0), Some(3));
        assert_eq!(cfg.next(1), Some(2));
        assert_eq!(cfg.cond(1), Some(3));
        assert!(cfg.next(2).is_none());
        assert!(cfg.cond(2).is_none());
        assert!(cfg.next(3).is_none());
        assert!(cfg.cond(3).is_none());
    }

    #[test]
    fn build_cfg_unconditional_jumps() {
        let stmts = vec![
            Statement::new(0x61E, "push rbp"),
            Statement::new(0x61F, "mov rbp, rsp"),
            Statement::new(0x622, "mov dword [var_4h], edi"),
            Statement::new(0x625, "mov dword [var_8h], esi"),
            Statement::new(0x628, "cmp dword [var_4h], 5"),
            Statement::new(0x62C, "jne 0x633"),
            Statement::new(0x62E, "mov eax, dword [var_8h]"),
            Statement::new(0x631, "jmp 0x638"),
            Statement::new(0x633, "mov eax, 6"),
            Statement::new(0x638, "pop rbp"),
            Statement::new(0x639, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        assert_eq!(cfg.len(), 4);
        assert_eq!(cfg.next(0), Some(1));
        assert_eq!(cfg.cond(0), Some(2));
        assert_eq!(cfg.next(1), Some(3));
        assert!(cfg.cond(1).is_none());
        assert_eq!(cfg.next(2), Some(3));
        assert!(cfg.cond(2).is_none());
        assert!(cfg.next(3).is_none());
        assert!(cfg.cond(3).is_none());
    }

    #[test]
    fn build_cfg_long_unconditional_jump() {
        // this is crafted so offsets are completely random
        let stmts = vec![
            Statement::new(0x610, "test edi, edi"),
            Statement::new(0x611, "je 0x613"),
            Statement::new(0x612, "jmp 0xFFFFFFFFFFFFFFFC"),
            Statement::new(0x613, "jmp 0x600"),
            Statement::new(0x614, "jmp 0x615"),
            Statement::new(0x615, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        assert_eq!(cfg.len(), 5);
        assert_eq!(cfg.next(0), Some(1));
        assert_eq!(cfg.cond(0), Some(2));
        assert!(cfg.next(1).is_none());
        assert!(cfg.cond(1).is_none());
        assert!(cfg.next(2).is_none());
        assert!(cfg.cond(2).is_none());
    }

    #[test]
    fn build_cfg_bb_offset() {
        let stmts = vec![
            Statement::new(0x610, "test edi, edi"),
            Statement::new(0x614, "je 0x628"),
            Statement::new(0x618, "test esi, esi"),
            Statement::new(0x61C, "mov eax, 5"),
            Statement::new(0x620, "je 0x628"),
            Statement::new(0x624, "ret"),
            Statement::new(0x628, "mov eax, 6"),
            Statement::new(0x62C, "ret"),
        ];
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        assert_eq!(cfg.len(), 4);
        assert_eq!(cfg.next(0), Some(1));
        assert_eq!(cfg.cond(0), Some(3));
        assert_eq!(cfg.next(1), Some(2));
        assert_eq!(cfg.cond(1), Some(3));
        assert!(cfg.next(2).is_none());
        assert!(cfg.cond(2).is_none());
        assert!(cfg.next(3).is_none());
        assert!(cfg.cond(3).is_none());
        assert_eq!(cfg[0].first, 0x610);
        assert_eq!(cfg[0].last, 0x614);
        assert_eq!(cfg[1].first, 0x618);
        assert_eq!(cfg[1].last, 0x620);
        assert_eq!(cfg[2].first, 0x624);
        assert_eq!(cfg[2].last, 0x624);
        assert_eq!(cfg[3].first, 0x628);
        assert_eq!(cfg[3].last, 0x62C);
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
        let stmts = Vec::new();
        let arch = ArchX86::new_amd64();
        let cfg = CFG::new(&stmts, &arch);
        let cfg_only_reachables = cfg.reachable();
        assert!(cfg_only_reachables.is_empty());
    }

    #[test]
    fn reachable_all() {
        let nodes = (0..)
            .take(4)
            .map(|x| BasicBlock {
                id: x,
                first: 0,
                last: 0,
            })
            .collect();
        let edges = vec![
            [Some(1), None],
            [Some(2), None],
            [Some(3), None],
            [None, None],
        ];
        let cfg = CFG { nodes, edges };
        let cfg_only_reachables = cfg.reachable();
        assert_eq!(cfg_only_reachables.len(), cfg.len());
    }

    #[test]
    fn reachable_some() {
        let nodes = (0..)
            .take(4)
            .map(|x| BasicBlock {
                id: x,
                first: 0,
                last: 0,
            })
            .collect();
        let edges = vec![[Some(1), None], [None, None], [Some(3), None], [None, None]];
        let cfg = CFG { nodes, edges };
        let cfg_only_reachables = cfg.reachable();
        assert_eq!(cfg_only_reachables.len(), 2);
    }
}
