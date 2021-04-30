use crate::analysis::{BasicBlock, Graph, StructureBlock, CFG};
use fnv::FnvHashSet;
use std::array::IntoIter;
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::hash::Hasher;
use std::rc::Rc;

pub struct CFS<H: Hasher> {
    cfg: CFG,
    structure: Option<Box<StructureBlock<H>>>,
}

impl<H: Hasher> CFS<H> {
    pub fn new(cfg: &CFG) -> CFS<H> {
        CFS {
            cfg: cfg.clone(),
            structure: build_cfs(cfg),
        }
    }
}

fn build_cfs<H: Hasher>(cfg: &CFG) -> Option<Box<StructureBlock<H>>> {
    let scss = cfg.scc();
    let preds = cfg.predecessors();
    let _nonat_cfg = remove_natural_loops(&scss, &preds, cfg.clone());
    todo!()
}

// calculates the depth of the spanning tree at each node.
fn calculate_depth(cfg: &CFG) -> HashMap<Rc<BasicBlock>, usize> {
    let mut depth_map = HashMap::new();
    for node in cfg.postorder() {
        let children = cfg.children(node).unwrap();
        let mut depth = 0;
        for child in children {
            if let Some(child_depth) = depth_map.get(child) {
                depth = max(depth, child_depth + 1);
            }
        }
        depth_map.insert(cfg.rc(node).unwrap(), depth);
    }
    depth_map
}

// calculates the exit nodes and target (of the exit) for a node in a particular loop
fn exits_and_targets(
    node: &BasicBlock,
    sccs: &HashMap<&BasicBlock, usize>,
    cfg: &CFG,
) -> (Vec<Rc<BasicBlock>>, HashSet<Rc<BasicBlock>>) {
    let mut visit = vec![node];
    let mut visited = IntoIter::new([node]).collect::<HashSet<_>>();
    let mut exits = Vec::new();
    let mut targets = HashSet::new();
    // checks the exits from the loop
    while let Some(node) = visit.pop() {
        let node_scc_id = *sccs.get(node).unwrap();
        for child in cfg.children(node).unwrap() {
            let child_scc_id = *sccs.get(child).unwrap();
            if child_scc_id != node_scc_id {
                let node_rc = cfg.rc(node).unwrap();
                let child_rc = cfg.rc(child).unwrap();
                exits.push(node_rc);
                targets.insert(child_rc);
            } else if !visited.contains(child) {
                // continue the visit only if the scc is the same and the node is not visited
                // |-> stay in the loop
                visit.push(child);
            }
            visited.insert(child);
        }
    }
    (exits, targets)
}

// remove all edges from a CFG that points to a list of targets
fn remove_edges<'a>(
    node: &'a BasicBlock,
    targets: &HashSet<Rc<BasicBlock>>,
    sccs: &HashMap<&'a BasicBlock, usize>,
    mut cfg: CFG,
) -> CFG {
    let mut changes = Vec::new();
    let mut visit = vec![node];
    let mut visited = IntoIter::new([node]).collect::<HashSet<_>>();
    while let Some(node) = visit.pop() {
        let node_rc = cfg.rc(node).unwrap();
        let node_scc_id = *sccs.get(node).unwrap();
        if let Some(cond) = cfg.cond(Some(node)) {
            if targets.contains(cond) {
                // remove edge (deferred)
                let current = [cfg.edges.get(node).unwrap()[0].clone(), None];
                changes.push((node_rc.clone(), current));
            } else {
                // don't remove edge
                let cond_scc_id = *sccs.get(cond).unwrap();
                if !visited.contains(cond) && cond_scc_id == node_scc_id {
                    visit.push(cond);
                }
                visited.insert(cond);
            }
        }
        // impossible that I need to edit both edge and cond: this would not be a loop
        else if let Some(next) = cfg.next(Some(node)) {
            if targets.contains(next) {
                // remove edge and swap next and cond
                let current = [cfg.edges.get(node).unwrap()[1].clone(), None];
                changes.push((node_rc.clone(), current));
            } else {
                // don't remove edge
                let next_scc_id = *sccs.get(next).unwrap();
                if !visited.contains(next) && next_scc_id == node_scc_id {
                    visit.push(next);
                }
                visited.insert(next);
            }
        }
    }
    for (node, edge) in changes {
        *cfg.edges.get_mut(&node).unwrap() = edge;
    }
    cfg
}

fn denaturate_loop(
    node: &BasicBlock,
    sccs: &HashMap<&BasicBlock, usize>,
    preds: &HashMap<&BasicBlock, HashSet<&BasicBlock>>,
    depth_map: &HashMap<Rc<BasicBlock>, usize>,
    mut cfg: CFG,
) -> CFG {
    let (exits, mut targets) = exits_and_targets(node, sccs, &cfg);
    if exits.len() > 1 {
        // harder case, more than 2 output targets, keep the target with the highest depth
        if targets.len() >= 2 {
            let correct = targets
                .iter()
                .reduce(|a, b| {
                    if depth_map.get(a) > depth_map.get(b) {
                        a
                    } else {
                        b
                    }
                })
                .unwrap()
                .clone();
            targets.remove(&correct);
            cfg = remove_edges(node, &targets, sccs, cfg);
        }
    }
    let (mut exits, targets) = exits_and_targets(node, sccs, &cfg);
    exits.sort_by(|a, b| {
        preds
            .get(&**a)
            .unwrap()
            .len()
            .cmp(&preds.get(&**b).unwrap().len())
    });
    if targets.len() == 1 {
        let correct_exit = IntoIter::new([exits.last().cloned().unwrap()]).collect::<HashSet<_>>();
        cfg = remove_edges(node, &correct_exit, sccs, cfg);
    }
    cfg
}

fn remove_natural_loops(
    sccs: &HashMap<&BasicBlock, usize>,
    preds: &HashMap<&BasicBlock, HashSet<&BasicBlock>>,
    mut cfg: CFG,
) -> CFG {
    let mut loops_done = FnvHashSet::default();
    let depth_map = calculate_depth(&cfg);
    let nodes = cfg.edges.keys().cloned().collect::<Vec<_>>();
    for node in nodes {
        let scc_id = sccs.get(&*node).unwrap();
        if !loops_done.contains(scc_id) {
            cfg = denaturate_loop(&node, sccs, preds, &depth_map, cfg);
            loops_done.insert(scc_id);
        }
    }
    cfg
}

#[cfg(test)]
mod tests {
    use crate::analysis::{cfs, BasicBlock, Graph, CFG};
    use maplit::hashmap;
    use std::collections::{HashMap, HashSet};
    use std::rc::Rc;

    fn empty() -> CFG {
        CFG {
            root: None,
            edges: HashMap::default(),
        }
    }

    fn sample() -> CFG {
        let nodes = (0..7)
            .map(|x| {
                Rc::new(BasicBlock {
                    id: 0,
                    first: x,
                    last: 0,
                })
            })
            .collect::<Vec<_>>();
        let adj = hashmap! {
            nodes[0].clone() => [Some(nodes[1].clone()), Some(nodes[2].clone())],
            nodes[1].clone() => [Some(nodes[6].clone()), None],
            nodes[2].clone() => [Some(nodes[3].clone()), None],
            nodes[3].clone() => [Some(nodes[5].clone()), None],
            nodes[4].clone() => [Some(nodes[2].clone()), None],
            nodes[5].clone() => [Some(nodes[6].clone()), Some(nodes[4].clone())],
            nodes[6].clone() => [None, None]
        };
        CFG {
            root: Some(nodes[0].clone()),
            edges: adj,
        }
    }

    #[test]
    fn calculate_depth_empty() {
        let cfg = empty();
        let depth = cfs::calculate_depth(&cfg);
        assert!(depth.is_empty());
    }

    #[test]
    fn calculate_depth() {
        let cfg = sample();
        let depth = cfs::calculate_depth(&cfg);
        let mut visit = cfg.postorder().collect::<Vec<_>>();
        visit.sort_by(|a, b| a.first.cmp(&b.first));
        let expected = vec![4_usize, 1, 3, 2, 0, 1, 0];
        let actual = visit
            .into_iter()
            .map(|x| *depth.get(x).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }
}
