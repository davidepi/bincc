use crate::analysis::blocks::StructureBlock;
use crate::analysis::{BasicBlock, BlockType, DirectedGraph, Graph, NestedBlock, CFG};
use fnv::FnvHashSet;
use std::array::IntoIter;
use std::cmp::{max, Ordering};
use std::collections::{HashMap, HashSet};
use std::fmt::Write as WriteFmt;
use std::fs::File;
use std::hash::Hash;
use std::io;
use std::io::Write as WriteIo;
use std::mem::swap;
use std::path::Path;
use std::rc::Rc;

#[derive(Clone)]
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

    pub fn to_dot(&self) -> String {
        let mut dot = self.cfg.to_dot();
        let old_ids = self.cfg.node_id_map();
        dot.pop();
        dot.pop();
        if let Some(root) = self.get_tree() {
            print_subgraph(&root, 0, &old_ids, &mut dot);
        }
        dot.push('}');
        dot.push('\n');
        dot
    }

    pub fn to_file<S: AsRef<Path>>(&self, filename: S) -> Result<(), io::Error> {
        let mut file = File::create(filename)?;
        file.write_all(self.to_dot().as_bytes())
    }

    pub fn to_dot_tree(&self) -> String {
        let mut dot = "digraph {\n".to_string();
        let mut stack = self.get_tree().iter().cloned().collect::<Vec<_>>();
        let mut ids = stack
            .iter()
            .cloned()
            .map(|x| (x, 0))
            .collect::<HashMap<_, _>>();
        while let Some(node) = stack.pop() {
            let ids_len = ids.len();
            let node_id = *ids.entry(node.clone()).or_insert(ids_len);
            match node {
                StructureBlock::Basic(_) => writeln!(
                    dot,
                    "{}[label=\"{}\";shape=\"box\"];",
                    node_id,
                    node.block_type()
                ),
                StructureBlock::Nested(_) => {
                    writeln!(dot, "{}[label=\"{}\"];", node_id, node.block_type())
                }
            }
            .unwrap();
            for child in node.children().iter().cloned() {
                let ids_len = ids.len();
                let child_id = *ids.entry(child.clone()).or_insert(ids_len);
                writeln!(dot, "{}->{}", node_id, child_id).unwrap();
                stack.push(child);
            }
        }
        dot.push('}');
        dot.push('\n');
        dot
    }

    pub fn to_file_tree<S: AsRef<Path>>(&self, filename: S) -> Result<(), io::Error> {
        let mut file = File::create(filename)?;
        file.write_all(self.to_dot_tree().as_bytes())
    }
}

fn print_subgraph<T: std::fmt::Write>(
    node: &StructureBlock,
    id: usize,
    cfg_ids: &HashMap<Rc<BasicBlock>, usize>,
    fmt: &mut T,
) {
    match node {
        StructureBlock::Basic(bb) => {
            writeln!(fmt, "{};", *cfg_ids.get(bb).unwrap()).unwrap();
        }
        StructureBlock::Nested(_) => {
            writeln!(fmt, "subgraph cluster_{}{{", id).unwrap();
            for (child_no, child) in node.children().iter().enumerate() {
                print_subgraph(child, id + child_no + 1, cfg_ids, fmt);
            }
            writeln!(fmt, "label=\"{}\";\n}}", node.block_type()).unwrap();
        }
    }
}

fn reduce_self_loop(
    node: &StructureBlock,
    graph: &DirectedGraph<StructureBlock>,
    _: &HashMap<&StructureBlock, HashSet<&StructureBlock>>,
    _: &HashMap<&StructureBlock, bool>,
) -> Option<(StructureBlock, Option<StructureBlock>)> {
    match node {
        StructureBlock::Basic(_) => {
            let children = graph.children(node).unwrap();
            if children.len() == 2 && children.contains(&node) {
                let next = children.into_iter().filter(|x| x != &node).last().unwrap();
                let block = Rc::new(NestedBlock::new(BlockType::SelfLooping, vec![node.clone()]));
                Some((StructureBlock::from(block), Some(next.clone())))
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
    _: &HashMap<&StructureBlock, bool>,
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
        if preds.get(next).map_or(0, |x| x.len()) == 1 {
            match nextnexts.len() {
                0 => Some((
                    StructureBlock::from(Rc::new(construct_and_flatten_sequence(node, next))),
                    None,
                )),
                1 => {
                    let nextnext = nextnexts.pop().unwrap();
                    if nextnext != node {
                        Some((
                            StructureBlock::from(Rc::new(construct_and_flatten_sequence(
                                node, next,
                            ))),
                            Some(nextnext.clone()),
                        ))
                    } else {
                        // particular type of looping sequence, still don't know how to handle this
                        let mut seq = construct_and_flatten_sequence(node, next);
                        seq.block_type = BlockType::SelfLooping;
                        Some((StructureBlock::from(Rc::new(seq)), None))
                    }
                }
                _ => None,
            }
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
) -> Vec<&'a StructureBlock> {
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
    rev_chain
}

fn reduce_ifthen(
    node: &StructureBlock,
    graph: &DirectedGraph<StructureBlock>,
    preds: &HashMap<&StructureBlock, HashSet<&StructureBlock>>,
    _: &HashMap<&StructureBlock, bool>,
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
            let child_rev = ascend_if_chain(vec![then, head], cont, graph, preds);
            //now creates the block itself
            let block = Rc::new(NestedBlock::new(
                BlockType::IfThen,
                child_rev.into_iter().cloned().rev().collect(),
            ));
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
    _: &HashMap<&StructureBlock, bool>,
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
            let child_rev = ascend_if_chain(vec![elseb, thenb, node], elseb, graph, preds);
            let child_set = child_rev.iter().collect::<HashSet<_>>();
            let preds_ok = elseb_preds
                .iter()
                .fold(true, |acc, x| acc & child_set.contains(x));
            if preds_ok {
                // in most cases the preds will be ok. However, to avoid wrong resolution due to
                // visiting order, this check is inserted (mostly to avoid resolving a "proper
                // interval" to a "if-then-else")
                let block = Rc::new(NestedBlock::new(
                    BlockType::IfThenElse,
                    child_rev.into_iter().cloned().rev().collect(),
                ));
                Some((StructureBlock::from(block), Some(elseb_children[0].clone())))
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

fn reduce_loop(
    node: &StructureBlock,
    graph: &DirectedGraph<StructureBlock>,
    preds: &HashMap<&StructureBlock, HashSet<&StructureBlock>>,
    loops: &HashMap<&StructureBlock, bool>,
) -> Option<(StructureBlock, Option<StructureBlock>)> {
    if *loops.get(&node).unwrap() && preds.get(&node).unwrap().len() > 1 {
        let head_children = graph.children(&node).unwrap();
        if head_children.len() == 2 {
            // while loop
            let next = head_children[0];
            let tail = head_children[1];
            find_while(node, next, tail, graph)
        } else if head_children.len() == 1 {
            // do-while loop
            let tail = head_children[0];
            let tail_children = graph.children(tail).unwrap();
            find_dowhile(node, tail, &tail_children, graph)
        } else {
            None
        }
    } else {
        None
    }
}

fn find_while(
    node: &StructureBlock,
    next: &StructureBlock,
    tail: &StructureBlock,
    graph: &DirectedGraph<StructureBlock>,
) -> Option<(StructureBlock, Option<StructureBlock>)> {
    let mut next = next;
    let mut tail = tail;
    if graph.children(&next).unwrap().contains(&node) {
        swap(&mut next, &mut tail);
    }
    let tail_children = graph.children(&tail).unwrap();
    if tail_children.len() == 1 && tail_children[0] == node {
        let block = Rc::new(NestedBlock::new(
            BlockType::While,
            vec![node.clone(), tail.clone()],
        ));
        Some((StructureBlock::from(block), Some(next.clone())))
    } else {
        None
    }
}

fn find_dowhile(
    node: &StructureBlock,
    tail: &StructureBlock,
    tail_children: &[&StructureBlock],
    graph: &DirectedGraph<StructureBlock>,
) -> Option<(StructureBlock, Option<StructureBlock>)> {
    if tail_children.len() == 2 {
        if !tail_children.contains(&node) {
            //type 3 or 4 (single node between tail and head) or no loop
            let post_tail_children = [
                graph.children(tail_children[0]).unwrap(),
                graph.children(tail_children[1]).unwrap(),
            ];
            let next;
            let post_tail;
            if post_tail_children[0].len() == 1 && post_tail_children[0][0] == node {
                post_tail = tail_children[0];
                next = tail_children[1];
            } else if post_tail_children[1].len() == 1 && post_tail_children[1][0] == node {
                post_tail = tail_children[1];
                next = tail_children[0];
            } else {
                return None;
            }
            let block = Rc::new(NestedBlock::new(
                BlockType::DoWhile,
                vec![node.clone(), tail.clone(), post_tail.clone()],
            ));
            Some((StructureBlock::from(block), Some(next.clone())))
        } else {
            //type 1 or 2 (single or no node between head and tail)
            let mut next = tail_children[0];
            if next == node {
                next = tail_children[1];
            }
            if node != next && tail != next {
                let block = Rc::new(NestedBlock::new(
                    BlockType::DoWhile,
                    vec![node.clone(), tail.clone()],
                ));
                Some((StructureBlock::from(block), Some(next.clone())))
            } else {
                None
            }
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
    NestedBlock::new(BlockType::Sequence, content)
}

fn remap_nodes(
    new: StructureBlock,
    next: Option<StructureBlock>,
    graph: DirectedGraph<StructureBlock>,
) -> DirectedGraph<StructureBlock> {
    if !graph.is_empty() {
        let mut new_adjacency = HashMap::new();
        let mut remove_list = new.children().iter().collect::<HashSet<_>>();
        let extended_from = if new.block_type() == BlockType::Sequence {
            // checks if the newly created node was an extension of a previous sequence. if this is
            // the case it is IMPORTANT to add the previous sequence to the remove_list, otherwise
            // infinite loop ;)
            graph
                .adjacency
                .iter()
                .filter(|(node, _)| node.block_type() == BlockType::Sequence)
                //todo: may I add additional checks to limit as much as possible the next filter?
                .filter(|(node, _)| {
                    node.children()
                        .iter()
                        .collect::<HashSet<_>>()
                        .difference(&remove_list)
                        .count()
                        == 0
                })
                .map(|(node, _)| node)
                .cloned()
                .collect()
        } else {
            HashSet::new()
        };
        remove_list.extend(extended_from.iter());
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
        let loops = is_loop(&graph.scc());
        for node in graph.postorder() {
            let reductions = [
                reduce_self_loop,
                reduce_loop,
                reduce_ifthen,
                reduce_ifelse,
                reduce_sequence,
            ];
            let mut reduced = None;
            for reduction in &reductions {
                reduced = (reduction)(node, &graph, &preds, &loops);
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
) -> (HashSet<Rc<BasicBlock>>, HashSet<Rc<BasicBlock>>) {
    let mut visit = vec![node];
    let mut visited = IntoIter::new([node]).collect::<HashSet<_>>();
    let mut exits = HashSet::new();
    let mut targets = HashSet::new();
    // checks the exits from the loop
    while let Some(node) = visit.pop() {
        let node_scc_id = *sccs.get(node).unwrap();
        for child in cfg.children(node).unwrap() {
            let child_scc_id = *sccs.get(child).unwrap();
            if child_scc_id != node_scc_id {
                let node_rc = cfg.rc(node).unwrap();
                let child_rc = cfg.rc(child).unwrap();
                exits.insert(node_rc);
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

fn is_loop<'a, T: Hash + Eq>(sccs: &HashMap<&'a T, usize>) -> HashMap<&'a T, bool> {
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
fn remove_edges(
    input_set: HashSet<Rc<BasicBlock>>,
    targets: HashSet<Rc<BasicBlock>>,
    mut cfg: CFG,
) -> CFG {
    for (node, edges) in cfg.edges.iter_mut() {
        if input_set.contains(node) {
            if let Some(next) = &mut edges[0] {
                if targets.contains(next) {
                    edges[0] = None;
                }
            }
            if let Some(cond) = &mut edges[1] {
                if targets.contains(cond) {
                    edges[1] = None;
                }
            }
            if edges[0].is_none() && edges[1].is_some() {
                edges.swap(0, 1);
            }
        }
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
    let distance = |x, y| {
        if x < y {
            y - x
        } else {
            x - y
        }
    };
    let (exits, mut targets) = exits_and_targets(node, sccs, &cfg);
    let is_loop = *is_loop(sccs).get(node).unwrap();
    if exits.len() > 1 && is_loop {
        // harder case, more than 2 output targets, keep the target with the highest depth
        if targets.len() >= 2 {
            let correct = targets
                .iter()
                .reduce(|a, b| {
                    // keep the deepest. If two or more have the same depth keep closest to me
                    match depth_map.get(a).cmp(&depth_map.get(b)) {
                        Ordering::Less => b,
                        Ordering::Equal => {
                            let difference_a = distance(node.first, a.first);
                            let difference_b = distance(node.first, b.first);
                            match difference_a.cmp(&difference_b) {
                                Ordering::Less => a,
                                Ordering::Equal => {
                                    /* bb offsets should be UNIQUE */
                                    panic!()
                                }
                                Ordering::Greater => b,
                            }
                        }
                        Ordering::Greater => a,
                    }
                })
                .unwrap()
                .clone();
            targets.remove(&correct);
            cfg = remove_edges(exits, targets, cfg);
        }
        let (exits, target) = exits_and_targets(node, sccs, &cfg);
        let correct_exit = if let Some(head) = exits.get(node) {
            // keep the exit which is either: the head (while case)
            let mut set = HashSet::new();
            set.insert(head.clone());
            set
        } else {
            // or farther away from the entry point (do-while case) -> highest predecessor number
            let max_preds = exits
                .iter()
                .fold(0, |acc, x| acc.max(preds.get(&**x).unwrap().len()));
            let exits_vec = exits
                .iter()
                .cloned()
                .filter(|x| preds.get(&**x).unwrap().len() == max_preds)
                .collect::<Vec<_>>();
            let exit = if exits_vec.len() == 1 {
                exits_vec.last().cloned().unwrap()
            } else {
                //two or more exits with same amount of predecessors to the same target
                //keep the one with further offset
                let (_, index_max_diff) = exits_vec
                    .iter()
                    .map(|x| distance(node.first, x.first))
                    .enumerate()
                    .map(|(index, value)| (value, index))
                    .max()
                    .unwrap();
                exits_vec[index_max_diff].clone()
            };
            let mut set = HashSet::new();
            set.insert(exit);
            set
        };
        let wrong_exits = exits
            .difference(&correct_exit)
            .cloned()
            .collect::<HashSet<_>>();
        cfg = remove_edges(wrong_exits, target, cfg);
    }
    // 1 exit and >1 targets can't exist in a CFG loop
    cfg
}

fn remove_natural_loops(
    sccs: &HashMap<&BasicBlock, usize>,
    preds: &HashMap<&BasicBlock, HashSet<&BasicBlock>>,
    mut cfg: CFG,
) -> CFG {
    let mut loops_done = FnvHashSet::default();
    let depth_map = calculate_depth(&cfg);
    let nodes = cfg
        .preorder()
        .map(|x| cfg.rc(x).unwrap())
        .collect::<Vec<_>>();
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
    use std::collections::HashMap;
    use std::rc::Rc;

    macro_rules! create_cfg {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(create_cfg!(@single $rest)),*]));
    ($($src:expr => $value:expr,)+) => { create_cfg!($($src => $value),+) };
    ($($src:expr => $value:expr),*) => {
        {
            let cap = create_cfg!(@count $($src),*);
            let nodes = (0..)
                        .take(cap)
                        .map(|x| Rc::new(BasicBlock { first: x, last: 0 }))
                        .collect::<Vec<_>>();
            #[allow(unused_mut)]
            let mut edges = std::collections::HashMap::with_capacity(cap);
            $(
                let mut targets = $value
                                  .iter()
                                  .map(|x: &usize| Some(nodes[*x].clone()))
                                  .collect::<Vec<_>>();
                targets.resize(2, None);
                targets.reverse();
                edges.insert(nodes[$src].clone(), [targets.pop().unwrap(), targets.pop().unwrap()]);
            )*
            let root = nodes.first().map(|x| x.clone());
            CFG {
                root,
                edges,
            }
        }
    };
    }

    fn empty() -> CFG {
        CFG {
            root: None,
            edges: HashMap::default(),
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
        let cfg = create_cfg! {
            0 => [1, 2], 1 => [6], 2 => [3], 3 => [5], 4 => [2], 5 => [6, 4], 6 => []
        };
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
        let cfg = create_cfg! {};
        let cfs = CFS::new(&cfg);
        assert!(cfs.get_tree().is_none());
    }

    #[test]
    fn reduce_sequence() {
        let cfg = create_cfg! { 0 => [1], 1 => [2], 2 => [3], 3 => [4], 4 => [] };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 5);
        assert_eq!(sequence.depth(), 1);
        assert_eq!(sequence.block_type(), BlockType::Sequence);
    }

    #[test]
    fn reduce_self_loop() {
        let cfg = create_cfg! { 0 => [1], 1 => [2, 1], 2 => [] };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 3);
        assert_eq!(sequence.depth(), 2);
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        let children = sequence.children();
        assert_eq!(children[0].block_type(), BlockType::Basic);
        assert_eq!(children[1].block_type(), BlockType::SelfLooping);
        assert_eq!(children[2].block_type(), BlockType::Basic);
    }

    #[test]
    fn reduce_if_then_next() {
        let cfg = create_cfg! { 0 => [1], 1 => [2, 3], 2 => [3], 3 => [4], 4 => [] };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 4);
        assert_eq!(sequence.depth(), 2);
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        let children = sequence.children();
        assert_eq!(children[0].block_type(), BlockType::Basic);
        assert_eq!(children[1].block_type(), BlockType::IfThen);
        assert_eq!(children[2].block_type(), BlockType::Basic);
        assert_eq!(children[3].block_type(), BlockType::Basic);
        assert_eq!(children[1].len(), 2);
    }

    #[test]
    fn reduce_if_then_cond() {
        let cfg = create_cfg! { 0 => [1], 1 => [3, 2], 2 => [3], 3 => [4], 4 => [] };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 4);
        assert_eq!(sequence.depth(), 2);
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        let children = sequence.children();
        assert_eq!(children[0].block_type(), BlockType::Basic);
        assert_eq!(children[1].block_type(), BlockType::IfThen);
        assert_eq!(children[2].block_type(), BlockType::Basic);
        assert_eq!(children[3].block_type(), BlockType::Basic);
        assert_eq!(children[1].len(), 2);
    }

    #[test]
    fn short_circuit_if_then() {
        // 2 is reached iff 0 and 1 holds
        let cfg = create_cfg! { 0 => [1, 3], 1 => [2, 3], 2 => [3], 3 => [] };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 2);
        assert_eq!(sequence.depth(), 2);
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        let children = sequence.children();
        assert_eq!(children[0].block_type(), BlockType::IfThen);
        assert_eq!(children[0].depth(), 1);
        assert_eq!(children[0].len(), 3);
        assert_eq!(children[1].block_type(), BlockType::Basic);
    }

    #[test]
    fn short_circuit_if_then_tricky() {
        // like short_circuit_if_then, but at some point the next and cond are swapped
        let cfg = create_cfg! {
            0 => [1, 4], 1 => [2, 4], 2 => [4, 3], 3 => [4], 4 => []
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 2);
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        let children = sequence.children();
        assert_eq!(children[0].block_type(), BlockType::IfThen);
        assert_eq!(children[0].len(), 4);
        assert_eq!(children[1].block_type(), BlockType::Basic);
        assert_eq!(sequence.depth(), 2);
    }

    #[test]
    fn reduce_if_else() {
        let cfg = create_cfg! {
            0 => [1], 1 => [2, 3], 2 => [4], 3 => [4], 4 => [5], 5 => []
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 4);
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        let children = sequence.children();
        assert_eq!(children[0].block_type(), BlockType::Basic);
        assert_eq!(children[1].block_type(), BlockType::IfThenElse);
        assert_eq!(children[2].block_type(), BlockType::Basic);
        assert_eq!(children[3].block_type(), BlockType::Basic);
        assert_eq!(children[1].len(), 3);
        assert_eq!(sequence.depth(), 2);
    }

    #[test]
    fn short_circuit_if_else() {
        let cfg = create_cfg! {
            0 => [1, 3], 1 => [2, 3], 2 => [3, 4], 3 => [5], 4 => [5], 5 => []
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 2);
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        let children = sequence.children();
        assert_eq!(children[0].block_type(), BlockType::IfThenElse);
        assert_eq!(children[0].len(), 5);
        assert_eq!(children[1].block_type(), BlockType::Basic);
        assert_eq!(sequence.depth(), 2);
    }

    #[test]
    fn if_else_looping() {
        // this test replicates a bug
        let cfg = create_cfg! { 0 => [1, 2], 1 => [3, 1], 2 => [3, 2], 3 => [] };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 2);
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        let children = sequence.children();
        assert_eq!(children[0].block_type(), BlockType::IfThenElse);
        assert_eq!(children[0].len(), 3);
        assert_eq!(
            children[0].children()[1].block_type(),
            BlockType::SelfLooping
        );
        assert_eq!(
            children[0].children()[2].block_type(),
            BlockType::SelfLooping
        );
        assert_eq!(sequence.depth(), 3);
        assert_eq!(children[0].depth(), 2);
    }

    #[test]
    fn whileb() {
        let cfg = create_cfg! { 0 => [1], 1 => [2, 3], 2 => [1], 3 => [] };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(sequence.len(), 3);
        assert_eq!(sequence.children()[1].block_type(), BlockType::While);
        assert_eq!(sequence.depth(), 2);
    }

    #[test]
    fn dowhile_type1() {
        // only 2 nodes, head and tail form the block
        let cfg = create_cfg! { 0 => [1], 1 => [2], 2 => [1, 3], 3 => [] };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(sequence.len(), 3);
        assert_eq!(sequence.children()[1].block_type(), BlockType::DoWhile);
        assert_eq!(sequence.depth(), 2);
    }

    #[test]
    fn dowhile_type2() {
        // three nodes form the block: head, extra, tail
        let cfg = create_cfg! { 0 => [1], 1 => [2], 2 => [3], 3 => [4, 1], 4 => [] };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(sequence.len(), 3);
        assert_eq!(sequence.children()[1].block_type(), BlockType::DoWhile);
        assert_eq!(sequence.depth(), 3);
    }

    #[test]
    fn dowhile_type3() {
        // three nodes form the block: head, tail, extra
        let cfg = create_cfg! { 0 => [1], 1 => [2], 2 => [3, 4], 3 => [1], 4 => [] };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(sequence.len(), 3);
        assert_eq!(sequence.children()[1].block_type(), BlockType::DoWhile);
        assert_eq!(sequence.depth(), 2);
    }

    #[test]
    fn dowhile_type4() {
        // four nodes form the block: head, extra, tail, extra
        let cfg = create_cfg! {
            0 => [1], 1 => [2], 2 => [3], 3 => [4, 5], 4 => [1], 5 => []
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(sequence.len(), 3);
        assert_eq!(sequence.children()[1].block_type(), BlockType::DoWhile);
        assert_eq!(sequence.depth(), 3);
    }

    #[test]
    fn structures_inside_loop() {
        // test several nested structures inside a loop
        let cfg = create_cfg! {
            0 => [1], 1 => [2], 2 => [3, 4], 3 => [5, 3], 4 => [5], 5 => [6, 1], 6 => []
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(sequence.len(), 3);
        assert_eq!(sequence.children()[1].block_type(), BlockType::DoWhile);
        assert_eq!(sequence.depth(), 5);
        assert_eq!(sequence.children()[1].depth(), 4);
    }

    #[test]
    fn nested_while() {
        // while inside while, sharing a head-tail
        let cfg = create_cfg! {
            0 => [1], 1 =>[4, 2], 2 => [3, 1], 3 => [2], 4 => []
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(sequence.len(), 3);
        assert_eq!(sequence.children()[1].block_type(), BlockType::While);
        assert_eq!(
            sequence.children()[1].children()[1].block_type(),
            BlockType::While
        );
        assert_eq!(sequence.depth(), 3);
        assert_eq!(sequence.children()[1].depth(), 2);
    }

    #[test]
    fn nested_dowhile_sharing() {
        // do-while inside do-while, sharing a head-tail
        let cfg = create_cfg! { 0 => [1], 1 => [2], 2 => [3, 1], 3 => [4, 2], 4 => [] };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(sequence.len(), 3);
        assert_eq!(sequence.children()[1].block_type(), BlockType::DoWhile);
        assert_eq!(
            sequence.children()[1].children()[0].block_type(),
            BlockType::DoWhile
        );
        assert_eq!(sequence.depth(), 3);
        assert_eq!(sequence.children()[1].depth(), 2);
    }

    #[test]
    fn nested_dowhile() {
        // do-while inside do-while, sharing no parts
        let cfg = create_cfg! {
            0 => [1], 1 => [2], 2 => [3], 3 => [4, 2], 4 => [5, 1], 5 => []
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(sequence.len(), 3);
        assert_eq!(sequence.children()[1].block_type(), BlockType::DoWhile);
        assert_eq!(
            sequence.children()[1].children()[0].children()[1].block_type(),
            BlockType::DoWhile
        );
        assert_eq!(sequence.depth(), 4);
    }

    #[test]
    fn nat_loop_break_while() {
        let cfg = create_cfg! { 0 => [1], 1 => [2, 4], 2 => [3, 4], 3 => [1], 4 => [] };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(sequence.len(), 3);
        assert_eq!(sequence.children()[1].block_type(), BlockType::While);
        assert_eq!(sequence.depth(), 3);
    }

    #[test]
    fn nat_loop_break_dowhile() {
        let cfg = create_cfg! { 0 => [1], 1 => [2], 2 => [3, 4], 3 => [4, 1], 4 => [] };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(sequence.len(), 3);
        assert_eq!(sequence.children()[1].block_type(), BlockType::DoWhile);
        assert_eq!(sequence.depth(), 3);
    }

    #[test]
    fn nat_loop_return_while() {
        let cfg = create_cfg! {
            0 => [1],
            1 => [2, 6],
            2 => [3, 6],
            3 => [6, 4],
            4 => [5, 8],
            5 => [8, 1],
            6 => [7],
            7 => [8],
            8 => []
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(sequence.len(), 5);
        assert_eq!(sequence.children()[1].block_type(), BlockType::While);
        assert_eq!(sequence.depth(), 3);
    }

    #[test]
    fn nat_loop_return_do_while() {
        let cfg = create_cfg! {
            0 => [1],
            1 => [2],
            2 => [3, 6],
            3 => [6, 4],
            4 => [5, 8],
            5 => [8, 1],
            6 => [7],
            7 => [8],
            8 => []
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(sequence.len(), 5);
        assert_eq!(sequence.children()[1].block_type(), BlockType::DoWhile);
        assert_eq!(sequence.depth(), 3);
    }

    #[test]
    fn nat_loop_return_orphaning() {
        // resolving this loop will create orphan nodes
        let cfg = create_cfg! {
            0 => [1],
            1 => [2, 8],
            2 => [3, 6],
            3 => [6, 4],
            4 => [5, 7],
            5 => [8, 1],
            6 => [7],
            7 => [9],
            8 => [9],
            9 => []
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(sequence.len(), 5);
        assert_eq!(sequence.children()[1].block_type(), BlockType::DoWhile);
        assert_eq!(sequence.depth(), 3);
    }

    #[test]
    fn looping_sequence() {
        // this caused a panic, assert it is not the case anymore
        let cfg = create_cfg! {
            13 => [ 7]    ,
            14 => [ 7]    ,
            15 => [ 5]    ,
             8 => [ 9, 11],
             6 => [ 7]    ,
            12 => [13, 14],
             9 => [10]    ,
            16 => [ 3]    ,
             4 => [ 5]    ,
             0 => [ 1, 16],
            11 => [ 3]    ,
             1 => [ 2, 16],
            10 => [11, 10],
             2 => [ 3]    ,
             3 => [ 4, 15],
             5 => [ 6, 12],
             7 => [ 8, 11],
        };
        let cfs = CFS::new(&cfg);
        assert!(cfs.get_tree().is_some());
    }

    #[test]
    fn proper_interval() {
        let cfg = create_cfg! {
          0 => [1, 2],
          1 => [3],
          2 => [1 ,3],
          3 => []
        };
        let cfs = CFS::new(&cfg);
        assert!(cfs.get_tree().is_none());
    }

    #[test]
    fn proper_interval_recursive() {
        let cfg = create_cfg! {
          0 => [1, 2],
          1 => [3],
          2 => [3, 4],
          3 => [5],
          4 => [5],
          5 => []
        };
        let cfs = CFS::new(&cfg);
        assert!(cfs.get_tree().is_none());
    }

    #[test]
    fn improper_interval() {
        let cfg = create_cfg! {
          0 => [1, 2],
          1 => [2, 3],
          2 => [1 ,3],
          3 => []
        };
        let cfs = CFS::new(&cfg);
        assert!(cfs.get_tree().is_none());
    }

    #[test]
    fn sequence_extension() {
        let cfg = create_cfg! {
          0 => [1, 2],
          1 => [2],
          2 => [3],
          3 => [4, 6],
          4 => [5, 6],
          5 => [0],
          6 => []
        };
        CFS::new(&cfg);
        // should not panic
    }
}
