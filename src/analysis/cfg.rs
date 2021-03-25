use crate::disasm::{Architecture, JumpType, Statement};
use fnv::FnvHashMap;
use parse_int::parse;
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::Write;
use std::ops::Index;
use std::path::Path;

pub struct CFG {
    nodes: Vec<BasicBlock>,
    edges: Vec<[Option<usize>; 2]>,
    root: usize,
}

pub struct BasicBlock {
    pub id: usize,
}

impl Display for CFG {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CFG({}, {})", self.nodes.len(), self.edges.len())
    }
}

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
    pub fn new(stmts: &[Statement], arch: &dyn Architecture) -> CFG {
        build_cfg(stmts, arch)
    }

    pub fn preorder(&self) -> CFGIter {
        let mut buffer = vec![self.root];
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
    }

    pub fn next(&self, id: usize) -> Option<usize> {
        self.edges[id][0]
    }

    pub fn cond(&self, id: usize) -> Option<usize> {
        self.edges[id][1]
    }

    pub fn root(&self) -> usize {
        self.root
    }

    pub fn to_dot(&self) -> String {
        let mut content = Vec::new();
        for edge in self.edges.iter().enumerate() {
            if let Some(next) = edge.1[0] {
                content.push(format!("{} -> {}", edge.0, next));
            }
            if let Some(cond) = edge.1[1] {
                content.push(format!("{} -> {}[arrowhead=\"empty\"];", edge.0, cond));
            }
        }
        format!("digraph {{\n{}\n}}\n", content.join("\n"))
    }

    pub fn to_file<S: AsRef<Path>>(&self, filename: S) -> Result<(), io::Error> {
        let mut file = File::create(filename)?;
        file.write_all(self.to_dot().as_bytes())
    }
}

impl Index<usize> for CFG {
    type Output = BasicBlock;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

struct TargetMap {
    targets: BTreeSet<u64>,
    srcs_cond: FnvHashMap<u64, u64>,
    srcs_uncond: FnvHashMap<u64, u64>,
    deadend_cond: BTreeSet<u64>,
    deadend_uncond: BTreeSet<u64>,
}

fn get_targets(stmts: &[Statement], arch: &dyn Architecture) -> TargetMap {
    let mut targets = BTreeSet::default();
    let mut srcs_cond = FnvHashMap::default();
    let mut srcs_uncond = FnvHashMap::default();
    let mut deadend_cond = BTreeSet::default();
    let mut deadend_uncond = BTreeSet::default();
    let func_lower_bound = stmts.first().unwrap().get_offset();
    let func_upper_bound = stmts.last().unwrap().get_offset();
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

fn build_cfg(stmts: &[Statement], arch: &dyn Architecture) -> CFG {
    let tgmap = get_targets(stmts, arch);
    let nodes = tgmap
        .targets
        .iter()
        .enumerate()
        .map(|x| BasicBlock { id: x.0 })
        .collect();
    let mut edges = (1..)
        .take(tgmap.targets.len() - 1)
        .map(|next| [Some(next), None])
        .collect::<Vec<_>>();
    edges.push([None, None]);
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
    CFG {
        nodes,
        edges,
        root: 0,
    }
}

#[cfg(test)]
mod tests {
    use crate::analysis::cfg::build_cfg;
    use crate::analysis::{BasicBlock, CFG};
    use crate::disasm::{ArchX86, Statement};

    #[test]
    fn preorder() {
        let nodes = (0..).take(7).map(|x| BasicBlock { id: x }).collect();
        let edges = vec![
            [Some(1), Some(2)],
            [Some(6), None],
            [Some(3), Some(4)],
            [Some(5), None],
            [Some(5), None],
            [Some(6), None],
            [None, None],
        ];
        let cfg = CFG {
            nodes,
            edges,
            root: 0,
        };
        let expected = vec![0, 1, 6, 2, 3, 5, 4];
        for (index, val) in cfg.preorder().enumerate() {
            assert_eq!(val.id, expected[index]);
        }
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
        let cfg = build_cfg(&stmts, &arch);
        assert_eq!(cfg.root(), 0);
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
        let cfg = build_cfg(&stmts, &arch);
        assert_eq!(cfg.root(), 0);
        assert_eq!(cfg.next(0), Some(1));
        assert_eq!(cfg.cond(0), Some(2));
        assert_eq!(cfg.next(1), Some(3));
        assert!(cfg.cond(1).is_none());
        assert_eq!(cfg.next(2), Some(3));
        assert!(cfg.cond(2).is_none());
        assert!(cfg.next(3).is_none());
        assert!(cfg.cond(3).is_none());
    }
}
