use crate::analysis::blocks::StructureBlock;
use crate::analysis::{BasicBlock, BlockType, DirectedGraph, Graph, NestedBlock, CFG};
use fnv::FnvHashSet;
use std::array::IntoIter;
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::mem::swap;
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

fn ascend_if_chain<'a>(
    mut rev_chain: Vec<&'a StructureBlock>,
    cont: &'a StructureBlock,
    graph: &DirectedGraph<StructureBlock>,
    preds: &HashMap<&'a StructureBlock, HashSet<&'a StructureBlock>>,
    mut depth: u32,
) -> (Vec<&'a StructureBlock>, u32) {
    let mut visited = rev_chain.iter().cloned().collect::<HashSet<_>>();
    let mut cur_head = *rev_chain.last().unwrap();
    while preds.get(cur_head).unwrap().len() == 1 {
        cur_head = preds.get(cur_head).unwrap().iter().last().unwrap();
        if !visited.contains(cur_head) {
            visited.insert(cur_head);
            let head_children = graph.children(cur_head).unwrap();
            if head_children.len() == 2 {
                // one of the edges must point to the cont block.
                // the other one obviously points to the current head
                if head_children[0] == cont || head_children[1] == cont {
                    rev_chain.push(cur_head);
                    depth = depth.max(cur_head.get_depth());
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
    (rev_chain, depth)
}

fn reduce_ifthen(
    node: &StructureBlock,
    graph: &DirectedGraph<StructureBlock>,
    preds: &HashMap<&StructureBlock, HashSet<&StructureBlock>>,
) -> Option<(StructureBlock, Option<StructureBlock>)> {
    let mut children = graph.children(node).unwrap();
    if children.len() == 2 {
        let head = node;
        let mut cont = children.pop().unwrap();
        let mut cont_children = graph.children(cont).unwrap();
        let mut then = children.pop().unwrap();
        let mut then_children = graph.children(then).unwrap();
        let mut then_preds = preds.get(then).unwrap();
        let mut cont_preds = preds.get(cont).unwrap();
        if cont_children.len() == 1 && cont_children[0] == then && cont_preds.len() == 1 {
            swap(&mut cont, &mut then);
            swap(&mut cont_children, &mut then_children);
            swap(&mut cont_preds, &mut then_preds);
        }
        if then_children.len() == 1 && then_children[0] == cont && then_preds.len() == 1 {
            // we detected the innermost if-then block. Now we try to ascend the various preds
            // to see if these is a chain of if-then. In order to hold, every edge not pointing
            // to the current one should point to the exit.
            let (child_rev, depth) = ascend_if_chain(
                vec![then, head],
                cont,
                graph,
                preds,
                std::cmp::max(then.get_depth(), head.get_depth()),
            );
            //now creates the block itself
            let block = Rc::new(NestedBlock {
                block_type: BlockType::IfThen,
                content: child_rev.into_iter().cloned().rev().collect(),
                depth: depth + 1,
            });
            Some((StructureBlock::from(block), Some(cont.clone())))
        } else {
            None
        }
    } else {
        None
    }
}

fn reduce_ifelse(
    node: &StructureBlock,
    graph: &DirectedGraph<StructureBlock>,
    preds: &HashMap<&StructureBlock, HashSet<&StructureBlock>>,
) -> Option<(StructureBlock, Option<StructureBlock>)> {
    let node_children = graph.children(&node).unwrap();
    if node_children.len() == 2 {
        let mut thenb = node_children[0];
        let mut thenb_preds = preds.get(&thenb).unwrap();
        let mut elseb = node_children[1];
        let mut elseb_preds = preds.get(&elseb).unwrap();
        // check for swapped if-else blocks
        if thenb_preds.len() > 1 {
            if elseb_preds.len() == 1 {
                swap(&mut thenb, &mut elseb);
                // technically I don't use them anymore, but I can already see big bugs if I will
                // ever modify this function without this line.
                swap(&mut thenb_preds, &mut elseb_preds);
            } else {
                return None;
            }
        }
        // checks that child of both then and else should go to the same node
        let thenb_children = graph.children(&thenb).unwrap();
        let elseb_children = graph.children(&elseb).unwrap();
        if thenb_children.len() == 1
            && elseb_children.len() == 1
            && thenb_children[0] == elseb_children[0]
        {
            // we detected the innermost if-else block. Now we try to ascend the various preds
            // to see if these is a chain of if-else. In order to hold, every edge not pointing
            // to the current one should point to the else block.
            let (child_rev, depth) = ascend_if_chain(
                vec![elseb, thenb, node],
                elseb,
                graph,
                preds,
                (elseb.get_depth().max(thenb.get_depth())).max(node.get_depth()),
            );
            let block = Rc::new(NestedBlock {
                block_type: BlockType::IfThenElse,
                content: child_rev.into_iter().cloned().rev().collect(),
                depth: depth + 1,
            });
            Some((StructureBlock::from(block), Some(elseb_children[0].clone())))
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
    new: StructureBlock,
    next: Option<StructureBlock>,
    graph: DirectedGraph<StructureBlock>,
) -> DirectedGraph<StructureBlock> {
    if !graph.is_empty() {
        let mut new_adjacency = HashMap::new();
        let remove_list = new.children().iter().collect::<HashSet<_>>();
        for (node, children) in graph.adjacency.into_iter() {
            if !remove_list.contains(&node) {
                let children_replaced = children
                    .into_iter()
                    .map(|child| {
                        if !remove_list.contains(&child) {
                            child
                        } else {
                            new.clone()
                        }
                    })
                    .collect();
                new_adjacency.insert(node.clone(), children_replaced);
            }
        }
        let replacement = match &next {
            None => vec![],
            Some(next_unwrapped) => vec![next_unwrapped.clone()],
        };
        new_adjacency.insert(new.clone(), replacement);

        let new_root = if !remove_list.contains(graph.root.as_ref().unwrap()) {
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
            let reductions = [
                reduce_self_loop,
                reduce_sequence,
                reduce_ifthen,
                reduce_ifelse,
            ];
            let mut reduced = None;
            for reduction in &reductions {
                reduced = (reduction)(node, &graph, &preds);
                if reduced.is_some() {
                    break;
                }
            }
            if let Some((new, next)) = reduced {
                graph = remap_nodes(new, next, graph);
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

fn is_loop<'a>(sccs: &HashMap<&'a BasicBlock, usize>) -> HashMap<&'a BasicBlock, bool> {
    let mut retval = HashMap::new();
    let mut counting = vec![0_usize; sccs.len()];
    for (_, scc_id) in sccs.iter() {
        counting[*scc_id] += 1;
    }
    for (node, scc_id) in sccs.iter() {
        if counting[*scc_id] <= 1 {
            retval.insert(*node, false);
        } else {
            retval.insert(*node, true);
        }
    }
    retval
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
    let is_loop = *is_loop(sccs).get(node).unwrap();
    if exits.len() > 1 && is_loop {
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
        let (mut exits, targets) = exits_and_targets(node, sccs, &cfg);
        exits.sort_by(|a, b| {
            preds
                .get(&**a)
                .unwrap()
                .len()
                .cmp(&preds.get(&**b).unwrap().len())
        });
        if targets.len() == 1 {
            let correct_exit =
                IntoIter::new([exits.last().cloned().unwrap()]).collect::<HashSet<_>>();
            cfg = remove_edges(node, &correct_exit, sccs, cfg);
        }
    }
    //TODO: what about 1 exit and 2 targets? can be solved by the other rules?
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
    use crate::analysis::{cfs, BasicBlock, BlockType, Graph, CFG, CFS};
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
                    first: x as u64,
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
        assert_eq!(sequence.get_type(), BlockType::Sequence);
    }

    #[test]
    fn reduce_self_loop() {
        // 0 -> 1 -> 2 with 1 -> 1 conditional loop and 1 -> 2 unconditional
        let nodes = create_nodes(3);
        let edges = hashmap! {
            nodes[0].clone() => [Some(nodes[1].clone()),None],
            nodes[1].clone() => [Some(nodes[2].clone()), Some(nodes[1].clone())],
            nodes[2].clone() => [None, None]
        };
        let cfg = CFG {
            root: Some(nodes[0].clone()),
            edges,
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 3);
        assert_eq!(sequence.get_depth(), 2);
        assert_eq!(sequence.get_type(), BlockType::Sequence);
        let children = sequence.children();
        assert_eq!(children[0].get_type(), BlockType::Basic);
        assert_eq!(children[1].get_type(), BlockType::SelfLooping);
        assert_eq!(children[2].get_type(), BlockType::Basic);
    }

    #[test]
    fn reduce_if_then_next() {
        // 0 -> 1 -> 2 -> 3 -> 4 uncond, 1->3 cond
        let nodes = create_nodes(5);
        let edges = hashmap! {
            nodes[0].clone() => [Some(nodes[1].clone()),None],
            nodes[1].clone() => [Some(nodes[2].clone()),Some(nodes[3].clone())],
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
        assert_eq!(sequence.len(), 4);
        assert_eq!(sequence.get_depth(), 2);
        assert_eq!(sequence.get_type(), BlockType::Sequence);
        let children = sequence.children();
        assert_eq!(children[0].get_type(), BlockType::Basic);
        assert_eq!(children[1].get_type(), BlockType::IfThen);
        assert_eq!(children[2].get_type(), BlockType::Basic);
        assert_eq!(children[3].get_type(), BlockType::Basic);
        assert_eq!(children[1].len(), 2);
    }

    #[test]
    fn reduce_if_then_cond() {
        // 0 -> 1 -> 3 -> 4 uncond, 2->3 uncond, 1->2 cond
        let nodes = create_nodes(5);
        let edges = hashmap! {
            nodes[0].clone() => [Some(nodes[1].clone()),None],
            nodes[1].clone() => [Some(nodes[3].clone()),Some(nodes[2].clone())],
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
        assert_eq!(sequence.len(), 4);
        assert_eq!(sequence.get_depth(), 2);
        assert_eq!(sequence.get_type(), BlockType::Sequence);
        let children = sequence.children();
        assert_eq!(children[0].get_type(), BlockType::Basic);
        assert_eq!(children[1].get_type(), BlockType::IfThen);
        assert_eq!(children[2].get_type(), BlockType::Basic);
        assert_eq!(children[3].get_type(), BlockType::Basic);
        assert_eq!(children[1].len(), 2);
    }

    #[test]
    fn short_circuit_if_then() {
        // 0 -> 1 -> 2 -> 3 uncond, 0 -> 3 cond, 1->3 cond
        // 2 is reached iff 0 and 1 holds
        let nodes = create_nodes(4);
        let edges = hashmap! {
            nodes[0].clone() => [Some(nodes[1].clone()),Some(nodes[3].clone())],
            nodes[1].clone() => [Some(nodes[2].clone()),Some(nodes[3].clone())],
            nodes[2].clone() => [Some(nodes[3].clone()),None],
            nodes[3].clone() => [None,None],
        };
        let cfg = CFG {
            root: Some(nodes[0].clone()),
            edges,
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 2);
        assert_eq!(sequence.get_type(), BlockType::Sequence);
        let children = sequence.children();
        assert_eq!(children[0].get_type(), BlockType::IfThen);
        assert_eq!(children[0].len(), 3);
        assert_eq!(children[1].get_type(), BlockType::Basic);
    }

    #[test]
    fn short_circuit_if_then_tricky() {
        // like short_circuit_if_then, but at some point the next and cond are swapped
        // 0 -> 1 -> 2 uncond, 3 -> 4 uncond, 2 -> 4 uncond, 0 -> 4 cond, 1 -> 4 cond, 2 -> 3 cond
        let nodes = create_nodes(5);
        let edges = hashmap! {
            nodes[0].clone() => [Some(nodes[1].clone()),Some(nodes[4].clone())],
            nodes[1].clone() => [Some(nodes[2].clone()),Some(nodes[4].clone())],
            nodes[2].clone() => [Some(nodes[4].clone()),Some(nodes[3].clone())],
            nodes[3].clone() => [Some(nodes[4].clone()),None],
            nodes[4].clone() => [None,None],
        };
        let cfg = CFG {
            root: Some(nodes[0].clone()),
            edges,
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 2);
        assert_eq!(sequence.get_type(), BlockType::Sequence);
        let children = sequence.children();
        assert_eq!(children[0].get_type(), BlockType::IfThen);
        assert_eq!(children[0].len(), 4);
        assert_eq!(children[1].get_type(), BlockType::Basic);
    }

    #[test]
    fn reduce_if_else() {
        // 0 -> 1 -> 2 -> 4 -> 5 uncond, 3->4 uncond,  1 -> 3 cond
        let nodes = create_nodes(6);
        let edges = hashmap! {
            nodes[0].clone() => [Some(nodes[1].clone()),None],
            nodes[1].clone() => [Some(nodes[2].clone()),Some(nodes[3].clone())],
            nodes[2].clone() => [Some(nodes[4].clone()),None],
            nodes[3].clone() => [Some(nodes[4].clone()),None],
            nodes[4].clone() => [Some(nodes[5].clone()),None],
            nodes[5].clone() => [None,None],
        };
        let cfg = CFG {
            root: Some(nodes[0].clone()),
            edges,
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 4);
        assert_eq!(sequence.get_type(), BlockType::Sequence);
        let children = sequence.children();
        assert_eq!(children[0].get_type(), BlockType::Basic);
        assert_eq!(children[1].get_type(), BlockType::IfThenElse);
        assert_eq!(children[2].get_type(), BlockType::Basic);
        assert_eq!(children[3].get_type(), BlockType::Basic);
        assert_eq!(children[1].len(), 3);
    }

    #[test]
    fn short_circuit_if_else() {
        let nodes = create_nodes(6);
        let edges = hashmap! {
            nodes[0].clone() => [Some(nodes[1].clone()),Some(nodes[3].clone())],
            nodes[1].clone() => [Some(nodes[2].clone()),Some(nodes[3].clone())],
            nodes[2].clone() => [Some(nodes[3].clone()),Some(nodes[4].clone())],
            nodes[3].clone() => [Some(nodes[5].clone()),None],
            nodes[4].clone() => [Some(nodes[5].clone()),None],
            nodes[5].clone() => [None,None],
        };
        let cfg = CFG {
            root: Some(nodes[0].clone()),
            edges,
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 2);
        assert_eq!(sequence.get_type(), BlockType::Sequence);
        let children = sequence.children();
        assert_eq!(children[0].get_type(), BlockType::IfThenElse);
        assert_eq!(children[0].len(), 5);
        assert_eq!(children[1].get_type(), BlockType::Basic);
    }

    #[test]
    fn if_else_looping() {
        // this test replicates a bug
        // 0 -> 1 -> 3 uncond, 2 -> 3 uncond, 0 -> 2 cond, 1 -> 1 cond, 2 -> 2 cond
        let nodes = create_nodes(4);
        let edges = hashmap! {
            nodes[0].clone() => [Some(nodes[1].clone()),Some(nodes[2].clone())],
            nodes[1].clone() => [Some(nodes[3].clone()),Some(nodes[1].clone())],
            nodes[2].clone() => [Some(nodes[3].clone()),Some(nodes[2].clone())],
            nodes[3].clone() => [None,None],
        };
        let cfg = CFG {
            root: Some(nodes[0].clone()),
            edges,
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 2);
        assert_eq!(sequence.get_type(), BlockType::Sequence);
    }
}
