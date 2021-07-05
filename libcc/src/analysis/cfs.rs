use crate::analysis::blocks::StructureBlock;
use crate::analysis::{BasicBlock, BlockType, DirectedGraph, Graph, NestedBlock, CFG};
use fnv::FnvHashSet;
use maplit::hashset;
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

// how many times the reduction may NOT decrease the amount of nodes before the CFS is
// terminated.
// This value is high due to the existence of self-loop reductions that are legit and does not
// decrease the node amount.
const BUILD_TOLERANCE: usize = 32;

#[derive(Clone)]
pub struct CFS {
    cfg: CFG,
    tree: DirectedGraph<StructureBlock>,
}

impl CFS {
    pub fn new(cfg: &CFG) -> CFS {
        let sinked_cfg = cfg.clone();
        let tree = build_cfs(&sinked_cfg);
        CFS {
            cfg: sinked_cfg,
            tree,
        }
    }

    pub fn get_graph(&self) -> &DirectedGraph<StructureBlock> {
        &self.tree
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
        dot.pop();
        dot.pop();
        for (node, _) in self.tree.adjacency.iter() {
            print_subgraph(node, 0, &mut dot);
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
        while let Some(node) = stack.pop() {
            let node_id = node.starting_offset();
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
                let child_id = child.starting_offset();
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

fn print_subgraph<T: std::fmt::Write>(node: &StructureBlock, id: usize, fmt: &mut T) -> usize {
    let mut latest = id;
    match node {
        StructureBlock::Basic(bb) => {
            if !bb.is_entry_point() && !bb.is_sink() {
                writeln!(fmt, "{};", bb.first).unwrap();
            }
        }
        StructureBlock::Nested(_) => {
            writeln!(fmt, "subgraph cluster_{}{{", id).unwrap();
            for child in node.children().iter() {
                latest = print_subgraph(child, latest + 1, fmt);
            }
            writeln!(fmt, "label=\"{}\";\n}}", node.block_type()).unwrap();
        }
    }
    latest
}

// result of a reduce_xxx method
struct Reduction<'a> {
    // old nodes that will be removed. Not necessary equal to new.children()
    // for example structures may expand previous structures, forcing the previous structure to
    // be discarded and a new one to be created.
    old: HashSet<&'a StructureBlock>,
    // new node that will replace the old one
    new: StructureBlock,
    // successor of the newly created node
    next: Option<&'a StructureBlock>,
}

fn reduce_self_loop<'a>(
    node: &'a StructureBlock,
    graph: &'a DirectedGraph<StructureBlock>,
    _: &HashMap<&'a StructureBlock, HashSet<&'a StructureBlock>>,
    _: &LoopHelper<'a>,
) -> Option<Reduction<'a>> {
    match node {
        StructureBlock::Basic(_) => {
            let children = graph.neighbours(node);
            if children.len() == 2 && children.contains(node) {
                let next = children.iter().filter(|x| x != &node).last().unwrap();
                let block = Rc::new(NestedBlock::new(BlockType::SelfLooping, vec![node.clone()]));
                Some(Reduction {
                    old: hashset![node],
                    new: StructureBlock::from(block),
                    next: Some(next),
                })
            } else {
                None
            }
        }
        StructureBlock::Nested(_) => None,
    }
}

fn reduce_switch<'a>(
    node: &'a StructureBlock,
    graph: &'a DirectedGraph<StructureBlock>,
    preds: &HashMap<&'a StructureBlock, HashSet<&'a StructureBlock>>,
    _: &LoopHelper<'a>,
) -> Option<Reduction<'a>> {
    let children = graph.neighbours(node);
    if children.len() >= 3 {
        let mut components = HashSet::new();
        components.insert(node);
        let mut oldlen = 0;
        // iteratively add nodes (having all preds inside the switch) until the switch is complete
        while components.len() != oldlen {
            oldlen = components.len();
            let neighbours = components
                .iter()
                .flat_map(|&x| graph.neighbours(x))
                .collect::<HashSet<_>>();
            for child in neighbours {
                if let Some(cur_preds) = preds.get(child) {
                    if !cur_preds.iter().any(|&x| !components.contains(x)) {
                        components.insert(child);
                    }
                }
            }
        }
        // now we need to find the next node.
        // first find the nodes with no children considering only the switch components
        let no_exit = components
            .iter()
            .filter(|&x| !graph.neighbours(x).iter().any(|y| components.contains(y)))
            .collect::<Vec<_>>();
        let next;
        if no_exit.len() == 1 {
            // the exit is part of the components set
            let exit = **no_exit.last().unwrap();
            components.remove(exit);
            next = Some(exit);
            let block = Rc::new(NestedBlock::new(
                BlockType::Switch,
                components.iter().copied().cloned().collect(),
            ));
            Some(Reduction {
                old: components,
                new: StructureBlock::from(block),
                next,
            })
        } else {
            let exit_set = no_exit
                .into_iter()
                .flat_map(|&x| graph.neighbours(x))
                .collect::<HashSet<_>>();
            if exit_set.len() == 1 {
                // all the nodes point to the same exit
                next = Some(exit_set.into_iter().next().unwrap());
                let block = Rc::new(NestedBlock::new(
                    BlockType::Switch,
                    components.iter().copied().cloned().collect(),
                ));
                Some(Reduction {
                    old: components,
                    new: StructureBlock::from(block),
                    next,
                })
            } else {
                None
            }
        }
    } else {
        None
    }
}

fn reduce_sequence<'a>(
    node: &'a StructureBlock,
    graph: &'a DirectedGraph<StructureBlock>,
    preds: &HashMap<&'a StructureBlock, HashSet<&'a StructureBlock>>,
    _: &LoopHelper<'a>,
) -> Option<Reduction<'a>> {
    // conditions for a sequence:
    // - current node has only one successor node
    // - successor has only one predecessor (the current node)
    // - successor has one or none successors
    //   ^--- this is necessary to avoid a double exit sequence
    let children = graph.neighbours(node);
    if children.len() == 1 {
        let next = children.first().unwrap();
        let nextnexts = graph.neighbours(next);
        if preds.get(next).map_or(0, |x| x.len()) == 1 {
            let mut reduction = construct_and_flatten_sequence(node, next);
            match nextnexts.len() {
                0 => Some(reduction),
                1 => {
                    let nextnext = nextnexts.first().unwrap();
                    if nextnext != node {
                        reduction.next = Some(nextnext);
                    } else {
                        // particular type of looping sequence, still don't know how to handle this
                        match reduction.new {
                            StructureBlock::Nested(ref mut nb) => {
                                Rc::get_mut(nb).unwrap().block_type = BlockType::SelfLooping
                            }
                            _ => panic!(),
                        }
                    }
                    Some(reduction)
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
            let head_children = graph.neighbours(cur_head);
            if head_children.len() == 2 {
                // one of the edges must point to the cont block.
                // the other one obviously points to the current head
                if &head_children[0] == cont || &head_children[1] == cont {
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

fn reduce_ifthen<'a>(
    node: &'a StructureBlock,
    graph: &'a DirectedGraph<StructureBlock>,
    preds: &HashMap<&'a StructureBlock, HashSet<&'a StructureBlock>>,
    _: &LoopHelper<'a>,
) -> Option<Reduction<'a>> {
    let children = graph.neighbours(node);
    if children.len() == 2 {
        let head = node;
        let mut cont = &children[0];
        let mut cont_children = graph.neighbours(cont);
        let mut then = &children[1];
        let mut then_children = graph.neighbours(then);
        let mut then_preds = preds.get(then).unwrap();
        let mut cont_preds = preds.get(cont).unwrap();
        if cont_children.len() == 1 && &cont_children[0] == then && cont_preds.len() == 1 {
            swap(&mut cont, &mut then);
            swap(&mut cont_children, &mut then_children);
            swap(&mut cont_preds, &mut then_preds);
        }
        if then_children.len() == 1 && &then_children[0] == cont && then_preds.len() == 1 {
            // we detected the innermost if-then block. Now we try to ascend the various preds
            // to see if these is a chain of if-then. In order to hold, every edge not pointing
            // to the current one should point to the exit.
            let child_rev = ascend_if_chain(vec![then, head], cont, graph, preds);
            //now creates the block itself
            let block = Rc::new(NestedBlock::new(
                BlockType::IfThen,
                child_rev.iter().cloned().cloned().rev().collect(),
            ));
            Some(Reduction {
                old: child_rev.into_iter().collect(),
                new: StructureBlock::from(block),
                next: Some(cont),
            })
        } else {
            None
        }
    } else {
        None
    }
}

fn reduce_ifelse<'a>(
    node: &'a StructureBlock,
    graph: &'a DirectedGraph<StructureBlock>,
    preds: &HashMap<&'a StructureBlock, HashSet<&'a StructureBlock>>,
    _: &LoopHelper<'a>,
) -> Option<Reduction<'a>> {
    let node_children = graph.neighbours(node);
    if node_children.len() == 2 {
        let mut thenb = &node_children[0];
        let mut thenb_preds = preds.get(&thenb).unwrap();
        let mut elseb = &node_children[1];
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
        let thenb_children = graph.neighbours(thenb);
        let elseb_children = graph.neighbours(elseb);
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
                    child_rev.iter().cloned().cloned().rev().collect(),
                ));
                Some(Reduction {
                    old: child_rev.into_iter().collect(),
                    new: StructureBlock::from(block),
                    next: Some(&elseb_children[0]),
                })
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

fn reduce_loop<'a>(
    node: &'a StructureBlock,
    graph: &'a DirectedGraph<StructureBlock>,
    preds: &HashMap<&'a StructureBlock, HashSet<&'a StructureBlock>>,
    lh: &LoopHelper<'a>,
) -> Option<Reduction<'a>> {
    if *lh.loops.get(&node).unwrap() && preds.get(&node).unwrap().len() > 1 {
        let head_children = graph.neighbours(node);
        if head_children.len() == 2 {
            // while loop
            let next = &head_children[0];
            let tail = &head_children[1];
            find_while(node, next, tail, preds, lh, graph)
        } else if head_children.len() == 1 {
            // do-while loop
            let tail = &head_children[0];
            let tail_children = graph.neighbours(tail);
            find_dowhile(node, tail, tail_children, preds, lh, graph)
        } else {
            None
        }
    } else {
        None
    }
}

// in a loop tail should NOT have predecessors coming from OUTSIDE the loop
// checking only the preds is not sufficient (check analysis::cfs::tests::nested_dowhile_sharing for
// a counter-example)
fn tail_preds_ok(
    tail: &StructureBlock,
    preds: &HashMap<&StructureBlock, HashSet<&StructureBlock>>,
    loop_helper: &LoopHelper,
) -> bool {
    !preds
        .get(tail)
        .unwrap()
        .iter()
        .any(|pred| loop_helper.sccs.get(pred).unwrap() != loop_helper.sccs.get(tail).unwrap())
}

fn find_while<'a>(
    node: &'a StructureBlock,
    next: &'a StructureBlock,
    tail: &'a StructureBlock,
    preds: &HashMap<&'a StructureBlock, HashSet<&'a StructureBlock>>,
    lh: &LoopHelper<'a>,
    graph: &'a DirectedGraph<StructureBlock>,
) -> Option<Reduction<'a>> {
    let mut next = next;
    let mut tail = tail;
    if graph.neighbours(next).contains(node) {
        swap(&mut next, &mut tail);
    }
    let tail_children = graph.neighbours(tail);
    if tail_children.len() == 1 && &tail_children[0] == node && tail_preds_ok(tail, preds, lh) {
        let block = Rc::new(NestedBlock::new(
            BlockType::While,
            vec![node.clone(), tail.clone()],
        ));
        Some(Reduction {
            old: hashset![node, tail],
            new: StructureBlock::from(block),
            next: Some(next),
        })
    } else {
        None
    }
}

fn find_dowhile<'a>(
    node: &'a StructureBlock,
    tail: &'a StructureBlock,
    tail_children: &'a [StructureBlock],
    preds: &HashMap<&'a StructureBlock, HashSet<&'a StructureBlock>>,
    lh: &LoopHelper<'a>,
    graph: &'a DirectedGraph<StructureBlock>,
) -> Option<Reduction<'a>> {
    if tail_children.len() == 2 {
        if !tail_children.contains(node) {
            //type 3 or 4 (single node between tail and head) or no loop
            let post_tail_children = [
                graph.neighbours(&tail_children[0]),
                graph.neighbours(&tail_children[1]),
            ];
            let next;
            let post_tail;
            if post_tail_children[0].len() == 1 && &post_tail_children[0][0] == node {
                post_tail = &tail_children[0];
                next = &tail_children[1];
            } else if post_tail_children[1].len() == 1 && &post_tail_children[1][0] == node {
                post_tail = &tail_children[1];
                next = &tail_children[0];
            } else {
                return None;
            }
            if tail_preds_ok(tail, preds, lh) && tail_preds_ok(post_tail, preds, lh) {
                let block = Rc::new(NestedBlock::new(
                    BlockType::DoWhile,
                    vec![node.clone(), tail.clone(), post_tail.clone()],
                ));
                Some(Reduction {
                    old: hashset![node, tail, post_tail],
                    new: StructureBlock::from(block),
                    next: Some(next),
                })
            } else {
                None
            }
        } else {
            //type 1 or 2 (single or no node between head and tail)
            let mut next = &tail_children[0];
            if next == node {
                next = &tail_children[1];
            }
            if node != next && tail != next && tail_preds_ok(tail, preds, lh) {
                let block = Rc::new(NestedBlock::new(
                    BlockType::DoWhile,
                    vec![node.clone(), tail.clone()],
                ));
                Some(Reduction {
                    old: hashset![node, tail],
                    new: StructureBlock::from(block),
                    next: Some(next),
                })
            } else {
                None
            }
        }
    } else {
        None
    }
}

fn reduce_improper_interval<'a>(
    node: &'a StructureBlock,
    graph: &'a DirectedGraph<StructureBlock>,
    _: &HashMap<&'a StructureBlock, HashSet<&'a StructureBlock>>,
    _: &LoopHelper<'a>,
) -> Option<Reduction<'a>> {
    let children = graph.neighbours(node);
    if children.len() == 2 {
        let left = &children[0];
        let right = &children[1];
        let children_left = graph.neighbours(left);
        let children_right = graph.neighbours(right);
        // should be 4 children in total, but one edge is removed during the nat loop resolution
        if children_left.len() + children_right.len() == 3
            && children_left.contains(right)
            && children_right.contains(left)
        {
            let next_set = children_left
                .iter()
                .chain(children_right.iter())
                .filter(|&x| x != left && x != right)
                .collect::<HashSet<_>>();
            if next_set.len() == 1 {
                let block = Rc::new(NestedBlock::new(
                    BlockType::ImproperInterval,
                    vec![node.clone(), left.clone(), right.clone()],
                ));
                Some(Reduction {
                    old: hashset![node, left, right],
                    new: StructureBlock::from(block),
                    next: next_set.into_iter().next(),
                })
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

fn reduce_proper_interval<'a>(
    node: &'a StructureBlock,
    graph: &'a DirectedGraph<StructureBlock>,
    preds: &HashMap<&'a StructureBlock, HashSet<&'a StructureBlock>>,
    _: &LoopHelper<'a>,
) -> Option<Reduction<'a>> {
    let children = graph.neighbours(node);
    if children.len() == 2 {
        let mut content = hashset![node, &children[0], &children[1]];
        let mut left = &children[0];
        let mut right = &children[1];
        let mut cross_exists = false; // at least one cross path should exist
        let next;
        loop {
            let left_children = graph.neighbours(left);
            let right_children = graph.neighbours(right);
            // first remove the current left or right
            let next_left = left_children
                .iter()
                .filter(|&x| x != right)
                .collect::<HashSet<_>>();
            let next_right = right_children
                .iter()
                .filter(|&x| x != left)
                .collect::<HashSet<_>>();
            if next_left.len() != left_children.len() || next_right.len() != right_children.len() {
                // record the removal (this is important)
                cross_exists = true;
            }
            if next_left.is_empty() || next_right.is_empty() {
                return None;
            }
            let total_next = next_left.len() + next_right.len();
            let children_union = next_left
                .union(&next_right)
                .copied()
                .collect::<HashSet<_>>();
            // if there is a backedge return immediately
            if children_union.intersection(&content).next().is_some() {
                return None;
            }
            // if the union of the children is exactly 1, that's the exit point
            match children_union.len() {
                1 => {
                    next = children_union.into_iter().next();
                    break;
                }
                2 => {}
                _ => return None,
            }
            // else, continue iterating
            match total_next {
                2 => {
                    left = next_left.into_iter().next().unwrap();
                    right = next_right.into_iter().next().unwrap();
                }
                3 => {
                    cross_exists = true;
                    if next_left.len() > next_right.len() {
                        right = next_right.into_iter().next().unwrap();
                        left = next_left.into_iter().find(|&x| x != right).unwrap();
                    } else {
                        left = next_left.into_iter().next().unwrap();
                        right = next_right.into_iter().find(|&x| x != left).unwrap();
                    }
                }
                4 => {
                    cross_exists = true;
                    let mut iter = children_union.iter().copied();
                    left = iter.next().unwrap();
                    right = iter.next().unwrap();
                    if left.starting_offset() > right.starting_offset() {
                        // in this case choosing left and right may be nondeterministic. So left
                        // is always the one with the lowest offset.
                        swap(&mut left, &mut right)
                    }
                }
                _ => return None,
            }
            content.insert(left);
            content.insert(right);
            // check preds, everything should come from nodes either in left or right path
            let preds_not_ok = preds
                .get(left)
                .unwrap()
                .iter()
                .chain(preds.get(right).unwrap().iter())
                .any(|&x| !content.contains(x));
            if preds_not_ok {
                return None;
            }
        }
        if cross_exists && next.is_some() {
            // cross exits checks avoid incorrectly resolving a if-else as proper interval
            let block = Rc::new(NestedBlock::new(
                BlockType::ProperInterval,
                content.iter().copied().cloned().collect(),
            ));
            Some(Reduction {
                old: content,
                new: StructureBlock::from(block),
                next,
            })
        } else {
            None
        }
    } else {
        None
    }
}

fn construct_and_flatten_sequence<'a>(
    node: &'a StructureBlock,
    next: &'a StructureBlock,
) -> Reduction<'a> {
    let flatten = |node: &'a StructureBlock| match node {
        StructureBlock::Basic(_) => {
            vec![node]
        }
        StructureBlock::Nested(nb) => {
            if nb.block_type == BlockType::Sequence {
                nb.content.iter().collect()
            } else {
                vec![node]
            }
        }
    };
    let mut reduction = Reduction {
        old: flatten(node).into_iter().chain(flatten(next)).collect(),
        new: StructureBlock::from(Rc::new(NestedBlock::new(
            BlockType::Sequence,
            flatten(node)
                .into_iter()
                .chain(flatten(next))
                .cloned()
                .collect(),
        ))),
        next: None,
    };
    if node.block_type() == BlockType::Sequence {
        reduction.old.insert(node);
    }
    if next.block_type() == BlockType::Sequence {
        reduction.old.insert(next);
    }
    reduction
}

fn remap_nodes(
    reduction: Reduction,
    graph: &DirectedGraph<StructureBlock>,
) -> DirectedGraph<StructureBlock> {
    if !graph.is_empty() {
        let mut new_adjacency = HashMap::new();
        for (node, children) in graph.adjacency.iter() {
            if !reduction.old.contains(&node) {
                let children_replaced = children
                    .iter()
                    .map(|child| {
                        if !reduction.old.contains(&child) {
                            child.clone()
                        } else {
                            reduction.new.clone()
                        }
                    })
                    .collect();
                new_adjacency.insert(node.clone(), children_replaced);
            }
        }
        let replacement = match reduction.next {
            None => vec![],
            Some(next_unwrapped) => vec![next_unwrapped.clone()],
        };
        new_adjacency.insert(reduction.new.clone(), replacement);

        let new_root = if !reduction.old.contains(graph.root.as_ref().unwrap()) {
            graph.root.clone()
        } else {
            Some(reduction.new)
        };
        DirectedGraph {
            root: new_root,
            adjacency: new_adjacency,
        }
    } else {
        graph.clone()
    }
}

struct LoopHelper<'a> {
    loops: HashMap<&'a StructureBlock, bool>,
    sccs: HashMap<&'a StructureBlock, usize>,
}

impl<'a> LoopHelper<'a> {
    fn new(graph: &'a DirectedGraph<StructureBlock>) -> LoopHelper<'a> {
        let sccs = graph.scc();
        let loops = is_loop(&sccs);
        LoopHelper { loops, sccs }
    }
}

fn build_cfs(cfg: &CFG) -> DirectedGraph<StructureBlock> {
    let nonat_cfg = remove_natural_loops(&cfg.scc(), &cfg.predecessors(), cfg.clone())
        .add_sink()
        .add_entry_point();
    let mut current_tolerance = 0;
    let mut graph = deep_copy(&nonat_cfg);
    let mut prev_len = nonat_cfg.len();
    loop {
        if graph.len() == 1 {
            break;
        }
        let mut modified = false;
        let preds = graph.predecessors();
        let loop_helper = LoopHelper::new(&graph);
        for node in graph.dfs_postorder() {
            let reductions = [
                reduce_self_loop,
                reduce_loop,
                reduce_ifthen,
                reduce_ifelse,
                reduce_sequence,
                reduce_switch,
                reduce_proper_interval,
                reduce_improper_interval,
            ];
            let mut reduced = None;
            for reduction in &reductions {
                reduced = (reduction)(node, &graph, &preds, &loop_helper);
                if reduced.is_some() {
                    break;
                }
            }
            if let Some(reduction) = reduced {
                graph = remap_nodes(reduction, &graph);
                if graph.len() < prev_len {
                    current_tolerance = 0;
                    prev_len = graph.len();
                } else {
                    current_tolerance += 1;
                }
                modified = true;
                break;
            }
        }
        if !modified || current_tolerance >= BUILD_TOLERANCE {
            break;
        }
    }
    // throw away unreachable nodes
    let visit = graph.bfs().cloned().collect::<HashSet<_>>();
    graph.adjacency = graph
        .adjacency
        .into_iter()
        .filter(|(node, _)| visit.contains(node))
        .collect();
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
                    .cloned()
                    .map(StructureBlock::from)
                    .collect();
                stack.extend(cfg.edges.get(&node).iter().flat_map(|x| x.iter()).cloned());
                graph.adjacency.insert(StructureBlock::from(node), children);
            }
        }
    }
    graph
}

// calculates the depth of the spanning tree at each node.
fn calculate_depth(cfg: &CFG) -> HashMap<Rc<BasicBlock>, usize> {
    let mut depth_map = HashMap::new();
    for node in cfg.dfs_postorder() {
        let children = cfg.neighbours(node);
        let mut depth = 0;
        for child in children {
            if let Some(child_depth) = depth_map.get(child) {
                depth = max(depth, child_depth + 1);
            }
        }
        depth_map.insert(node.clone(), depth);
    }
    depth_map
}

// calculates the exit nodes and target (of the exit) for a node in a particular loop
fn exits_and_targets(
    node: &Rc<BasicBlock>,
    sccs: &HashMap<&Rc<BasicBlock>, usize>,
    cfg: &CFG,
) -> (HashSet<Rc<BasicBlock>>, HashSet<Rc<BasicBlock>>) {
    let mut visit = vec![node];
    let mut visited = IntoIter::new([node]).collect::<HashSet<_>>();
    let mut exits = HashSet::new();
    let mut targets = HashSet::new();
    // checks the exits from the loop
    while let Some(node) = visit.pop() {
        let node_scc_id = *sccs.get(node).unwrap();
        for child in cfg.neighbours(node) {
            let child_scc_id = *sccs.get(child).unwrap();
            if child_scc_id != node_scc_id {
                exits.insert(node.clone());
                targets.insert(child.clone());
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
    cfg: CFG,
) -> CFG {
    type EdgesMap = HashMap<Rc<BasicBlock>, Vec<Rc<BasicBlock>>>;
    let (keep, edit): (EdgesMap, EdgesMap) = cfg
        .edges
        .into_iter()
        .partition(|(src, _)| !input_set.contains(src));
    let done = edit
        .into_iter()
        .map(|(src, dst)| {
            (
                src,
                dst.into_iter()
                    .filter(|child| !targets.contains(child))
                    .collect(),
            )
        })
        .chain(keep)
        .collect();
    CFG {
        root: cfg.root,
        edges: done,
    }
}

fn denaturate_loop(
    node: &Rc<BasicBlock>,
    sccs: &HashMap<&Rc<BasicBlock>, usize>,
    preds: &HashMap<&Rc<BasicBlock>, HashSet<&Rc<BasicBlock>>>,
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
                .fold(0, |acc, x| acc.max(preds.get(x).unwrap().len()));
            let exits_vec = exits
                .iter()
                .cloned()
                .filter(|x| preds.get(x).unwrap().len() == max_preds)
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
    sccs: &HashMap<&Rc<BasicBlock>, usize>,
    preds: &HashMap<&Rc<BasicBlock>, HashSet<&Rc<BasicBlock>>>,
    mut cfg: CFG,
) -> CFG {
    let mut loops_done = FnvHashSet::default();
    let depth_map = calculate_depth(&cfg);
    let nodes = cfg.dfs_preorder().cloned().collect::<Vec<_>>();
    for node in nodes {
        let scc_id = sccs.get(&node).unwrap();
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
                        .map(|x| Rc::new(BasicBlock { first: x, last: x+1 }))
                        .collect::<Vec<_>>();
            #[allow(unused_mut)]
            let mut edges = std::collections::HashMap::with_capacity(cap);
            $(
                let targets = $value.iter().map(|x: &usize| nodes[*x].clone()).collect::<Vec<_>>();
                edges.insert(nodes[$src].clone(), targets);
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
        let mut visit = cfg.dfs_postorder().collect::<Vec<_>>();
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
    fn reduce_switch() {
        let cfg = create_cfg! {
            0 => [1],
            1 => [2, 3, 4, 5, 6],
            2 => [7],
            3 => [7],
            4 => [7],
            5 => [7],
            6 => [7],
            7 => []
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 3);
        let children = sequence.children();
        assert_eq!(children[1].block_type(), BlockType::Switch);
        assert_eq!(children[1].len(), 6);
    }

    #[test]
    fn switch_fallthrough_single() {
        let cfg = create_cfg! { 0 => [1, 2, 3], 1 => [2], 2 => [4], 3 => [4], 4 => [] };
        let cfs = CFS::new(&cfg);
        assert!(cfs.get_tree().is_some());
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 2);
        let children = sequence.children();
        assert_eq!(children[0].block_type(), BlockType::Switch);
    }

    #[test]
    fn switch_fallthrough_multiple() {
        let cfg = create_cfg! { 0 => [1, 2, 3], 1 => [2, 3], 2 => [4], 3 => [4], 4 => [] };
        let cfs = CFS::new(&cfg);
        assert!(cfs.get_tree().is_some());
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 2);
        let children = sequence.children();
        assert_eq!(children[0].block_type(), BlockType::Switch);
    }

    #[test]
    fn switch_indirect() {
        let cfg = create_cfg! {
            0 => [1, 2, 3, 4],
            1 => [2],
            2 => [3, 4],
            3 => [5],
            4 => [5],
            5 => []
        };
        let cfs = CFS::new(&cfg);
        assert!(cfs.get_tree().is_some());
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 2);
        let children = sequence.children();
        assert_eq!(children[0].block_type(), BlockType::Switch);
    }

    #[test]
    fn switch_exit_in_components() {
        let cfg = create_cfg! {
            0 => [1, 2, 4, 5],
            1 => [2],
            2 => [4, 3],
            3 => [4, 5],
            4 => [6],
            5 => [6],
            6 => []
        };
        let cfs = CFS::new(&cfg);
        assert!(cfs.get_tree().is_some());
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 2);
        let children = sequence.children();
        assert_eq!(children[0].block_type(), BlockType::Switch);
    }

    #[test]
    fn switch_exit_not_in_components() {
        let cfg = create_cfg! {
            0 => [1, 7],
            1 => [2, 3, 5, 6],
            2 => [3],
            3 => [5, 4],
            4 => [5, 6],
            5 => [8],
            6 => [8],
            7 => [8],
            8 => []
        };
        let cfs = CFS::new(&cfg);
        assert!(cfs.get_tree().is_some());
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.len(), 2);
        let children = sequence.children();
        assert_eq!(children[0].block_type(), BlockType::IfThenElse);
        assert_eq!(children[0].children()[1].block_type(), BlockType::Switch);
    }

    #[test]
    fn switch_exit_impossible() {
        let cfg = create_cfg! {
            0 => [1, 7],
            1 => [2, 3, 5, 6],
            2 => [3],
            3 => [5, 4],
            4 => [5, 6],
            5 => [7],
            6 => [8],
            7 => [8],
            8 => []
        };
        let cfs = CFS::new(&cfg);
        assert!(cfs.get_tree().is_none())
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
    fn while_no_entry() {
        let cfg = create_cfg! { 0 => [1, 2], 1 => [0], 2 => [] };
        let cfs = CFS::new(&cfg.add_entry_point());
        assert!(cfs.get_tree().is_some());
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.children()[1].block_type(), BlockType::While);
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
    fn dowhile_no_entry() {
        let cfg = create_cfg! { 0 => [1], 1 => [0, 2], 2 => [] };
        let cfs = CFS::new(&cfg.add_entry_point());
        assert!(cfs.get_tree().is_some());
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.children()[1].block_type(), BlockType::DoWhile);
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
        let cfg = create_cfg! { 0 => [1], 1 =>[4, 2], 2 => [3, 1], 3 => [2], 4 => [] };
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
    fn proper_interval_mini() {
        let cfg = create_cfg! { 0 => [1, 2], 1 => [2, 3], 2 => [3], 3 => [] };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(
            sequence.children()[0].block_type(),
            BlockType::ProperInterval
        );
    }

    #[test]
    fn proper_interval_left() {
        let cfg = create_cfg! {
            0 => [1, 2], 1 => [3, 4], 2 => [4], 3 => [5], 4 => [5], 5 => []
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(
            sequence.children()[0].block_type(),
            BlockType::ProperInterval
        );
    }

    #[test]
    fn proper_interval_right() {
        let cfg = create_cfg! {
            0 => [1, 2], 1 => [3], 2 => [3, 4], 3 => [5], 4 => [5], 5 => []
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(
            sequence.children()[0].block_type(),
            BlockType::ProperInterval
        );
    }

    #[test]
    fn proper_interval_cross() {
        let cfg = create_cfg! {
            0 => [1, 2], 1 => [3, 4], 2 => [3, 4], 3 => [5], 4 => [5], 5 => []
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(
            sequence.children()[0].block_type(),
            BlockType::ProperInterval
        );
    }

    #[test]
    #[ignore]
    fn proper_interval_multilevel() {
        // currently not supported, not sure if it is a good idea to do so
        let cfg = create_cfg! {
            0 => [1, 2],
            1 => [3, 6],
            2 => [3, 4],
            3 => [5],
            4 => [5, 6],
            5 => [7],
            6 => [7],
            7 => [],
        };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(
            sequence.children()[0].block_type(),
            BlockType::ProperInterval
        );
    }

    #[test]
    fn proper_interval_wrong() {
        let cfg = create_cfg! {
            0 => [1, 2], 1 => [3, 4], 2 => [4, 5], 3 => [6], 4 => [6], 5 => [6], 6 => []
        };
        let cfs = CFS::new(&cfg);
        assert!(cfs.get_tree().is_none());
    }

    #[test]
    fn improper_interval() {
        let cfg = create_cfg! { 0 => [1, 2], 1 => [2, 3], 2 => [1 ,3], 3 => [] };
        let cfs = CFS::new(&cfg);
        let sequence = cfs.get_tree().unwrap();
        assert_eq!(sequence.block_type(), BlockType::Sequence);
        assert_eq!(
            sequence.children()[0].block_type(),
            BlockType::ImproperInterval
        );
    }

    #[test]
    fn sequence_extension() {
        // some interesting stuff here:
        // - loop with no evident entry point
        // - sequence extension
        let cfg = create_cfg! {
            0 => [1, 2], 1 => [2], 2 => [3], 3 => [4, 6], 4 => [5, 6], 5 => [0], 6 => []
        };
        let cfs = CFS::new(&cfg.add_entry_point());
        assert!(cfs.get_tree().is_some());
    }
}
