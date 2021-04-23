use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;

/// A trait used to represent a generic graph.
///
/// After implementing the methods to return the node given the ID, the children given the node ID
/// and the graph len, it is possible to perform several graph visits.
///
/// Note that the *node ID* does not refer to any particular field, it is just an identifier used
/// to distinguish between various nodes. It is duty of the implementor to use this information
/// and return the correct node accounting for graph modifications and out of bound requests.
// TODO: update description after adding SCC and dominator visit
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
}

/// Generic Directed Graph.
///
/// Generic implementation of a directed graph using a vector of neighbours.
///
/// No constructor or specific methods are provided for this class, as one should update the nodes
/// and edges vectors manually.
pub struct DirectedGraph<T> {
    /// root of the graph (if rooted and not empty)
    pub root: Option<Rc<T>>,
    // neighbour for a given node in the graph.
    pub adjacency: HashMap<Rc<T>, Vec<Rc<T>>>,
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
                retval.push(&**child);
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
    use std::collections::HashMap;
    use std::rc::Rc;

    fn diamond() -> DirectedGraph<u8> {
        let nodes = (0..).take(7).map(Rc::new).collect::<Vec<_>>();
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
}
