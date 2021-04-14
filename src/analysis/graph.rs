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
    type Item;

    /// Returns the graph node with a specific ID.
    ///
    /// Returns None if the node with the given ID does not exist.
    fn node(&self, id: usize) -> Option<&Self::Item>;

    /// Given a node ID, returns a vector containing the IDs of all its children.
    ///
    /// Returns *None* if the node with the given ID does not exist.
    fn children(&self, id: usize) -> Option<Vec<usize>>;

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
            let mut buffer = vec![0];
            let mut retval = Vec::with_capacity(self.len());
            let mut marked = vec![false; self.len()];
            while let Some(current_id) = buffer.pop() {
                let current = self.node(current_id).unwrap();
                retval.push(current);
                marked[current_id] = true;
                let children = self.children(current_id).unwrap();
                for child_id in children.iter().rev() {
                    if !marked[*child_id] {
                        marked[*child_id] = true;
                        buffer.push(*child_id);
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
            let mut buffer = vec![0];
            let mut retval = Vec::with_capacity(self.len());
            let mut marked = vec![false; self.len()];
            while let Some(current_id) = buffer.last() {
                let mut to_push = Vec::new();
                marked[*current_id] = true;
                let children = self.children(*current_id).unwrap();
                for child_id in children.iter().rev() {
                    if !marked[*child_id] {
                        marked[*child_id] = true;
                        to_push.push(*child_id);
                    }
                }
                // if all children has been processed, push current node
                if to_push.is_empty() {
                    let current = self.node(buffer.pop().unwrap()).unwrap();
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
    /// Vector containing the nodes of this directed graph.
    pub nodes: Vec<T>,
    /// Vector containing the edges of this directed graph.
    /// Each index of this array represent the node with the same index on [DirectedGraph::nodes].
    /// Each element of this array contains a vector of children IDs for the specific node it
    /// represents.
    pub edges: Vec<Vec<usize>>,
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
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

impl<T> Graph for DirectedGraph<T> {
    type Item = T;

    fn node(&self, id: usize) -> Option<&Self::Item> {
        if id < self.nodes.len() {
            Some(&self.nodes[id])
        } else {
            None
        }
    }

    fn children(&self, id: usize) -> Option<Vec<usize>> {
        if id < self.edges.len() {
            Some(self.edges[id].to_vec())
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::analysis::{DirectedGraph, Graph};

    fn diamond() -> DirectedGraph<u8> {
        DirectedGraph {
            nodes: (0..).take(7).collect(),
            edges: vec![
                vec![1, 2],
                vec![6],
                vec![3, 4],
                vec![5],
                vec![5],
                vec![6],
                vec![],
            ],
        }
    }

    #[test]
    fn directed_graph_node_nonexisting() {
        let graph = diamond();
        let node = graph.node(999);
        assert!(node.is_none());
    }

    #[test]
    fn directed_graph_node_existing() {
        let graph = diamond();
        let node = graph.node(5);
        assert!(node.is_some());
        assert_eq!(*node.unwrap(), 5)
    }

    #[test]
    fn directed_graph_edges_nonexisting() {
        let graph = diamond();
        let children = graph.children(999);
        assert!(children.is_none());
    }

    #[test]
    fn directed_graph_edges_existing() {
        let graph = diamond();
        let children = graph.children(5);
        assert!(children.is_some());
        assert_eq!(children.unwrap(), vec![6]);
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
