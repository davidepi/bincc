use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// A trait used to represent a generic graph.
///
/// After implementing the methods to return the node given the ID, the children given the node ID
/// and the graph len, it is possible to perform several graph visits.
///
/// Note that the *node ID* does not refer to any particular field, it is just an identifier used
/// to distinguish between various nodes. It is duty of the implementor to use this information
/// and return the correct node accounting for graph modifications and out of bound requests.
pub trait Graph {
    /// Type of elements contained in the graph
    type Item: Hash + Eq;

    /// Returns the starting node of the graph.
    ///
    /// Note that graphs may not be rooted. In this case this method should return any node
    /// belonging to the graph, but consider that any visit of the graph will start from this node.
    ///
    /// Returns None if the graph is empty.
    fn root(&self) -> Option<&Self::Item>;

    /// Given a node, returns a vector with its children.
    ///
    /// Returns *None* if the input node does not exist in the graph.
    fn children(&self, node: &Self::Item) -> Option<Vec<&Self::Item>>;

    /// Returns the size of the graph in the number of nodes.
    fn len(&self) -> usize;

    /// Returns true if the graph is empty (i.e. it has no nodes), false otherwise.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Visits the graph nodes in pre-order.
    ///
    /// Returns an iterator visiting every node reachable from the CFG root using a
    /// depth-first pre-order.
    ///
    /// In the default implementation this visit is iterative.
    ///
    /// This method panics if the graph representation is inconsistent,
    /// i.e. if [Graph::children()] method returns non-existing IDs.
    fn preorder(&self) -> GraphIter<'_, Self::Item> {
        if !self.is_empty() {
            let mut buffer = vec![self.root().unwrap()];
            let mut retval = Vec::with_capacity(self.len());
            let mut marked = HashSet::with_capacity(self.len());
            while let Some(current) = buffer.pop() {
                retval.push(current);
                marked.insert(current);
                let children = self.children(current).unwrap();
                for child in children.into_iter().rev() {
                    if !marked.contains(child) {
                        marked.insert(child);
                        buffer.push(child);
                    }
                }
            }
            retval.reverse();
            GraphIter { stack: retval }
        } else {
            GraphIter { stack: Vec::new() }
        }
    }

    /// Visits the graph nodes in post-order.
    ///
    /// Returns an iterator visiting every node reachable from the CFG root using a
    /// depth-first post-order.
    ///
    /// In the default implementation this visit is iterative.
    ///
    /// This method panics if the graph representation is inconsistent,
    /// i.e. if [Graph::children()] method returns non-existing IDs.
    fn postorder(&self) -> GraphIter<'_, Self::Item> {
        if !self.is_empty() {
            let mut buffer = vec![self.root().unwrap()];
            let mut retval = Vec::with_capacity(self.len());
            let mut marked = HashSet::with_capacity(self.len());
            while let Some(current) = buffer.last() {
                let mut to_push = Vec::new();
                marked.insert(*current);
                let children = self.children(*current).unwrap();
                for child in children.into_iter().rev() {
                    if !marked.contains(child) {
                        marked.insert(child);
                        to_push.push(child);
                    }
                }
                // if all children has been processed, push current node
                if to_push.is_empty() {
                    let current = buffer.pop().unwrap();
                    retval.push(current);
                } else {
                    buffer.append(&mut to_push);
                }
            }
            retval.reverse();
            GraphIter { stack: retval }
        } else {
            GraphIter { stack: Vec::new() }
        }
    }

    /// Returns the list of direct predecessors for each node.
    ///
    /// The direct predecessors for a given node are its parent nodes.
    ///
    /// The map returned from this method is guaranteed to have an entry for each node in the graph.
    fn predecessors(&self) -> HashMap<&Self::Item, HashSet<&Self::Item>>
    where
        Self::Item: Hash + Eq,
    {
        let mut pmap = HashMap::with_capacity(self.len());
        let visit = self.preorder();
        for node in visit {
            // the next line is used to have a set for each node. Otherwise nodes with no children
            // will never get their entry inside the map.
            pmap.entry(node).or_insert_with(HashSet::new);
            let children = self.children(node).unwrap();
            for child in children {
                let child_map = pmap.entry(child).or_insert_with(HashSet::new);
                child_map.insert(node);
            }
        }
        pmap
    }

    /// Calculates the strongly connected components of the current graph.
    ///
    /// Returns a map containing the connected component index assigned to each node belonging to
    /// the current graph.
    ///
    /// This method uses an iterative version of Tarjan's algorithm with O(|V|+|E|) complexity.
    fn scc(&self) -> HashMap<&Self::Item, usize>
    where
        Self::Item: Hash + Eq,
    {
        // assign indices to everything (array indexing will be used a lot in this method)
        let ids = self
            .preorder()
            .enumerate()
            .map(|(index, item)| (item, index))
            .collect::<HashMap<_, _>>();
        let mut adj = vec![Vec::new(); ids.len()];
        for (node, index) in &ids {
            let children = self
                .children(node)
                .unwrap()
                .into_iter()
                .flat_map(|x| ids.get(x))
                .copied()
                .collect();
            adj[*index] = children;
        }
        let mut lowlink = vec![0; ids.len()];
        let mut index = vec![usize::MAX; ids.len()];
        let mut on_stack = vec![false; ids.len()];
        let mut stack = Vec::new();
        let mut call_stack = Vec::new();
        let mut next_scc = 0;
        let mut sccs = vec![usize::MAX; ids.len()];
        let mut i = 0;
        for v in 0..ids.len() {
            if index[v] == usize::MAX {
                call_stack.push((v, 0));
                while let Some((v, mut pi)) = call_stack.pop() {
                    if pi == 0 {
                        index[v] = i;
                        lowlink[v] = i;
                        i += 1;
                        stack.push(v);
                        on_stack[v] = true;
                    } else if pi > 0 {
                        lowlink[v] = min(lowlink[v], lowlink[adj[v][pi - 1]]);
                    }
                    while pi < adj[v].len() && index[adj[v][pi]] != usize::MAX {
                        let w = adj[v][pi];
                        if on_stack[w] {
                            lowlink[v] = min(lowlink[v], index[w]);
                        }
                        pi += 1;
                    }
                    if pi < adj[v].len() {
                        let w = adj[v][pi];
                        call_stack.push((v, pi + 1));
                        call_stack.push((w, 0));
                    } else if lowlink[v] == index[v] {
                        loop {
                            let w = stack.pop().unwrap();
                            on_stack[w] = false;
                            sccs[w] = next_scc;
                            if w == v {
                                break;
                            }
                        }
                        next_scc += 1;
                    }
                }
            }
        }
        ids.into_iter()
            .map(|(node, index)| (node, sccs[index]))
            .collect()
    }
}

/// Generic Directed Graph.
///
/// Generic implementation of a directed graph using a vector of neighbours.
///
/// This method stores multiple copies of each node: consider using primitive types or a Reference
/// Counted pointer.
///
/// No constructor or specific methods are provided for this class, as one should update the nodes
/// and edges vectors manually.
pub struct DirectedGraph<T> {
    /// root of the graph (if rooted and not empty)
    pub root: Option<T>,
    // neighbour for a given node in the graph.
    pub adjacency: HashMap<T, Vec<T>>,
}

/// Iterator for Graph elements.
pub struct GraphIter<'a, T> {
    stack: Vec<&'a T>,
}

impl<'a, T> Iterator for GraphIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop()
    }
}

impl<T> Default for DirectedGraph<T> {
    fn default() -> Self {
        DirectedGraph {
            root: None,
            adjacency: HashMap::new(),
        }
    }
}

impl<T: Hash + Eq> Graph for DirectedGraph<T> {
    type Item = T;

    fn root(&self) -> Option<&Self::Item> {
        if let Some(root) = &self.root {
            Some(&root)
        } else {
            None
        }
    }

    fn children(&self, node: &Self::Item) -> Option<Vec<&Self::Item>> {
        if let Some(children) = self.adjacency.get(node) {
            let mut retval = Vec::with_capacity(children.len());
            for child in children {
                retval.push(&*child);
            }
            Some(retval)
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.adjacency.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::analysis::{DirectedGraph, Graph};
    use std::collections::{HashMap, HashSet};

    fn diamond() -> DirectedGraph<u8> {
        let nodes = (0..).take(7).collect::<Vec<_>>();
        let mut graph = DirectedGraph {
            root: Some(nodes[0].clone()),
            adjacency: HashMap::new(),
        };
        graph
            .adjacency
            .insert(nodes[0].clone(), vec![nodes[1].clone(), nodes[2].clone()]);
        graph
            .adjacency
            .insert(nodes[1].clone(), vec![nodes[6].clone()]);
        graph
            .adjacency
            .insert(nodes[2].clone(), vec![nodes[3].clone(), nodes[4].clone()]);
        graph
            .adjacency
            .insert(nodes[3].clone(), vec![nodes[5].clone()]);
        graph
            .adjacency
            .insert(nodes[4].clone(), vec![nodes[5].clone()]);
        graph
            .adjacency
            .insert(nodes[5].clone(), vec![nodes[6].clone()]);
        graph.adjacency.insert(nodes[6].clone(), vec![]);
        graph
    }

    #[test]
    fn directed_graph_root_empty() {
        let graph: DirectedGraph<u8> = DirectedGraph::default();
        let node = graph.root();
        assert!(node.is_none());
    }

    #[test]
    fn directed_graph_root_existing() {
        let graph = diamond();
        let node = graph.root();
        assert!(node.is_some());
        assert_eq!(*node.unwrap(), 0)
    }

    #[test]
    fn directed_graph_children_empty() {
        let graph: DirectedGraph<u8> = DirectedGraph::default();
        let children = graph.children(&0);
        assert!(children.is_none());
    }

    #[test]
    fn directed_graph_children_existing() {
        let graph = diamond();
        let children = graph.children(graph.root().unwrap());
        assert!(children.is_some());
        assert_eq!(*children.unwrap()[0], 1);
    }

    #[test]
    fn directed_graph_len() {
        let graph = diamond();
        assert_eq!(graph.len(), 7);
    }

    #[test]
    fn graph_is_empty() {
        let graph: DirectedGraph<u8> = DirectedGraph::default();
        assert!(graph.is_empty());
        assert_eq!(graph.len(), 0);
    }

    #[test]
    fn graph_is_not_empty() {
        let graph = diamond();
        assert!(!graph.is_empty());
    }

    #[test]
    fn preorder_empty() {
        let graph: DirectedGraph<u8> = DirectedGraph::default();
        let order = graph.preorder();
        assert_eq!(order.count(), 0);
    }

    #[test]
    fn preorder() {
        let graph = diamond();
        let expected = vec![0, 1, 6, 2, 3, 5, 4];
        for (index, val) in graph.preorder().enumerate() {
            assert_eq!(*val, expected[index]);
        }
    }

    #[test]
    fn postorder_empty() {
        let graph: DirectedGraph<u8> = DirectedGraph::default();
        let order = graph.postorder();
        assert_eq!(order.count(), 0);
    }

    #[test]
    fn postorder() {
        let graph = diamond();
        let expected = vec![6, 1, 5, 3, 4, 2, 0];
        for (index, val) in graph.postorder().enumerate() {
            assert_eq!(*val, expected[index]);
        }
    }

    #[test]
    fn predecessors_empty() {
        let graph: DirectedGraph<u8> = DirectedGraph::default();
        let pmap = graph.predecessors();
        assert!(pmap.is_empty())
    }

    #[test]
    fn predecessors() {
        let graph = diamond();
        let pmap = graph.predecessors();
        assert_eq!(pmap.len(), graph.len());
        let pred_5 = pmap.get(&5).unwrap();
        let set = [3, 4].iter().collect::<HashSet<_>>();
        assert_eq!(pred_5.len(), set.len());
        for val in pred_5 {
            assert!(set.contains(val));
        }
    }

    #[test]
    fn sccs_empty() {
        let graph: DirectedGraph<u8> = DirectedGraph::default();
        let sccs = graph.scc();
        assert!(sccs.is_empty())
    }

    #[test]
    fn sccs() {
        let mut graph = diamond();
        // edit the graph to introduce a cycle
        let node2 = (graph.adjacency.get_key_value(&2).unwrap().0).clone();
        let node4 = (graph.adjacency.get_key_value(&4).unwrap().0).clone();
        let node5 = (graph.adjacency.get_key_value(&5).unwrap().0).clone();
        let node6 = (graph.adjacency.get_key_value(&6).unwrap().0).clone();
        graph.adjacency.insert(node4.clone(), vec![node2]);
        graph.adjacency.insert(node5, vec![node4, node6]);
        // asserts the sccs indices equalities/inequalities
        let sccs = graph.scc();
        assert_ne!(sccs.get(&0).unwrap(), sccs.get(&2).unwrap());
        assert_ne!(sccs.get(&1).unwrap(), sccs.get(&2).unwrap());
        assert_eq!(sccs.get(&3).unwrap(), sccs.get(&2).unwrap());
        assert_eq!(sccs.get(&4).unwrap(), sccs.get(&2).unwrap());
        assert_eq!(sccs.get(&5).unwrap(), sccs.get(&2).unwrap());
        assert_ne!(sccs.get(&6).unwrap(), sccs.get(&2).unwrap());
        assert_ne!(sccs.get(&0).unwrap(), sccs.get(&1).unwrap());
        assert_ne!(sccs.get(&0).unwrap(), sccs.get(&6).unwrap());
        assert_ne!(sccs.get(&1).unwrap(), sccs.get(&6).unwrap());
    }
}
