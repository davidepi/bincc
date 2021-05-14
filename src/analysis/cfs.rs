use crate::analysis::blocks::StructureBlock;
use crate::analysis::{BasicBlock, BlockType, DirectedGraph, Graph, NestedBlock, CFG};
use fnv::FnvHashSet;
use std::array::IntoIter;
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct CFS {
    cfg: CFG,
    tree: DirectedGraph<StructureBlock>,
}

impl CFS {
    pub fn new(cfg: &CFG) -> CFS {
        CFS {
            cfg: cfg.clone(),
            tree: build_cfs(cfg),
        }
    }

    pub fn get_tree(&self) -> Option<StructureBlock> {
        if self.tree.len() == 1 {
            Some(self.tree.root.clone().unwrap())
        } else {
            None
        }
    }

    pub fn get_cfg(&self) -> &CFG {
        &self.cfg
    }
}

fn reduce_self_loop(
    node: &StructureBlock,
    graph: &DirectedGraph<StructureBlock>,
    _: &HashMap<&StructureBlock, HashSet<&StructureBlock>>,
) -> Option<(StructureBlock, Option<StructureBlock>)> {
    match node {
        StructureBlock::Basic(_) => {
            let children = graph.children(node).unwrap();
            if children.len() == 2 && children.contains(&node) {
                let next = children.into_iter().filter(|x| x != &node).last().unwrap();
                Some((
                    StructureBlock::from(Rc::new(NestedBlock {
                        block_type: BlockType::SelfLooping,
                        content: vec![node.clone()],
                        depth: node.get_depth() + 1,
                    })),
                    Some(next.clone()),
                ))
            } else {
                None
            }
        }
        StructureBlock::Nested(_) => None,
    }
}

fn reduce_sequence(
    node: &StructureBlock,
    graph: &DirectedGraph<StructureBlock>,
    preds: &HashMap<&StructureBlock, HashSet<&StructureBlock>>,
) -> Option<(StructureBlock, Option<StructureBlock>)> {
    // conditions for a sequence:
    // - current node has only one successor node
    // - successor has only one predecessor (the current node)
    // - successor has one or none successors
    //   ^--- this is necessary to avoid a double exit sequence
    let mut children = graph.children(node).unwrap();
    if children.len() == 1 {
        let next = children.pop().unwrap();
        let mut nextnexts = graph.children(next).unwrap();
        if preds.get(next).map_or(0, |x| x.len()) == 1 && nextnexts.len() <= 1 {
            Some((
                StructureBlock::from(Rc::new(construct_and_flatten_sequence(node, next))),
                nextnexts.pop().cloned(),
            ))
        } else {
            None
        }
    } else {
        None
    }
}

fn construct_and_flatten_sequence(node: &StructureBlock, next: &StructureBlock) -> NestedBlock {
    let flatten = |node: &StructureBlock| match node {
        StructureBlock::Basic(_) => {
            vec![node.clone()]
        }
        StructureBlock::Nested(nb) => {
            if nb.block_type == BlockType::Sequence {
                nb.content.clone()
            } else {
                vec![node.clone()]
            }
        }
    };
    let content = flatten(node)
        .into_iter()
        .chain(flatten(next))
        .collect::<Vec<_>>();
    let depth = content.iter().fold(0, |acc, val| val.get_depth().max(acc));
    NestedBlock {
        block_type: BlockType::Sequence,
        content,
        depth: depth + 1,
    }
}

fn remap_nodes(
    old: StructureBlock,
    new: StructureBlock,
    next: Option<StructureBlock>,
    graph: DirectedGraph<StructureBlock>,
) -> DirectedGraph<StructureBlock> {
    if !graph.is_empty() {
        let mut new_adjacency = HashMap::new();
        for (node, children) in graph.adjacency.into_iter() {
            if node != old {
                let children_replaced = children
                    .into_iter()
                    .map(|child| if child != old { child } else { new.clone() })
                    .collect();
                new_adjacency.insert(node.clone(), children_replaced);
            } else {
                let replacement = match &next {
                    None => vec![],
                    Some(next_unwrapped) => vec![next_unwrapped.clone()],
                };
                new_adjacency.insert(new.clone(), replacement);
            }
        }
        let new_root = if graph.root.as_ref().unwrap() != &old {
            graph.root
        } else {
            Some(new)
        };
        let mut new_graph = DirectedGraph {
            root: new_root,
            adjacency: new_adjacency,
        };
        //remove unreachable nodes from map (they are now wrapped inside other nodes)
        let reachables = new_graph.preorder().cloned().collect::<HashSet<_>>();
        new_graph.adjacency = new_graph
            .adjacency
            .into_iter()
            .filter(|(node, _)| reachables.contains(node))
            .collect();
        new_graph
    } else {
        graph
    }
}

fn build_cfs(cfg: &CFG) -> DirectedGraph<StructureBlock> {
    let nonat_cfg = remove_natural_loops(&cfg.scc(), &cfg.predecessors(), cfg.clone());
    let mut graph = deep_copy(&nonat_cfg);
    loop {
        let mut modified = false;
        let preds = graph.predecessors();
        for node in graph.postorder() {
            let reductions = [reduce_self_loop, reduce_sequence];
            let mut reduced = None;
            for reduction in &reductions {
                reduced = (reduction)(node, &graph, &preds);
                if reduced.is_some() {
                    break;
                }
            }
            if let Some((new, next)) = reduced {
                graph = remap_nodes(node.clone(), new, next, graph);
                modified = true;
                break;
            }
        }
        if !modified {
            break;
        }
    }
    graph
}

fn deep_copy(cfg: &CFG) -> DirectedGraph<StructureBlock> {
    let mut graph = DirectedGraph::default();
    if !cfg.is_empty() {
        let root = cfg.root.as_ref().unwrap().clone();
        graph.root = Some(StructureBlock::from(root.clone()));
        let mut stack = vec![root];
        let mut visited = HashSet::with_capacity(cfg.len());
        while let Some(node) = stack.pop() {
            if !visited.contains(&node) {
                visited.insert(node.clone());
                let children = cfg
                    .edges
                    .get(&node)
                    .iter()
                    .flat_map(|x| x.iter())
                    .flatten()
                    .cloned()
                    .map(StructureBlock::from)
                    .collect();
                stack.extend(
                    cfg.edges
                        .get(&node)
                        .iter()
                        .flat_map(|x| x.iter())
                        .flatten()
                        .cloned(),
                );
                graph.adjacency.insert(StructureBlock::from(node), children);
            }
        }
    }
    graph
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
    use crate::analysis::{cfs, BasicBlock, Graph, CFG, CFS};
    use maplit::hashmap;
    use std::collections::HashMap;
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

    #[test]
    fn constructor_empty() {
        let cfg = empty();
        let cfs = CFS::new(&cfg);
        assert!(cfs.get_tree().is_none());
    }

    fn create_nodes(n: usize) -> Vec<Rc<BasicBlock>> {
        (0..)
            .take(n)
            .map(|x| {
                Rc::new(BasicBlock {
                    id: x,
                    first: 0,
                    last: 0,
                })
            })
            .collect()
    }

    #[test]
    fn reduce_sequence() {
        // 0 -> 1 -> 2 -> 3 -> 4
        let nodes = create_nodes(5);
        let edges = hashmap! {
            nodes[0].clone() => [Some(nodes[1].clone()),None],
            nodes[1].clone() => [Some(nodes[2].clone()),None],
            nodes[2].clone() => [Some(nodes[3].clone()),None],
            nodes[3].clone() => [Some(nodes[4].clone()),None],
            nodes[4].clone() => [None,None],
        };
        let cfg = CFG {
            root: Some(nodes[0].clone()),
            edges,
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 5);
        assert_eq!(sequence.get_depth(), 1);
    }

    // #[test]
    // TODO: finish implementing
    // fn reduce_self_loop() {
    //     // 0 -> 1 -> 2 with 1 -> 1 conditional loop and 1 -> 2 unconditional
    //     let nodes = create_nodes(3);
    //     let edges = hashmap! {
    //         nodes[0].clone() => [Some(nodes[1].clone()),None],
    //         nodes[1].clone() => [Some(nodes[2].clone()), Some(nodes[1].clone())],
    //         nodes[2].clone() => [None, None]
    //     };
    //     let cfg = CFG {
    //         root: Some(nodes[0].clone()),
    //         edges
    //     };
    //
    // }
}
