use std::cmp::min;
use std::collections::{HashMap, HashSet, VecDeque};
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

    /// Given a node, returns a vector with its neighbours.
    ///
    /// Returns *None* if the input node does not exist in the graph.
    fn neighbours(&self, node: &Self::Item) -> &[Self::Item];

    /// Returns the size of the graph in the number of nodes.
    fn len(&self) -> usize;

    /// Returns true if the graph is empty (i.e. it has no nodes), false otherwise.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Visits the graph nodes in a breadth-first fashion.
    ///
    /// Returns an iterator visiting every node reachable from [Graph::root()] using a
    /// breadth-first pre-order visit. All the nodes at the same depth are thus visited before
    /// moving to the next depth level.
    ///
    /// In the default implementation this visit is iterative.
    fn bfs(&self) -> BfsIter<'_, Self>
    where
        Self: Sized,
    {
        if let Some(root) = self.root() {
            let mut queue = VecDeque::with_capacity(self.len());
            queue.push_back(root);
            let mut buffer = VecDeque::new();
            buffer.push_back(root);
            BfsIter {
                queue,
                buffer,
                marked: HashSet::with_capacity(self.len()),
                graph: &self,
            }
        } else {
            BfsIter {
                queue: VecDeque::with_capacity(0),
                buffer: VecDeque::with_capacity(0),
                marked: HashSet::with_capacity(0),
                graph: &self,
            }
        }
    }

    /// Visits the graph nodes using a depth-first search in pre-order.
    ///
    /// Returns an iterator visiting every node reachable from [Graph::root()] using a
    /// depth-first pre-order.
    ///
    /// In the default implementation this visit is iterative.
    fn dfs_preorder(&self) -> DfsPreIter<'_, Self>
    where
        Self: Sized,
    {
        if let Some(root) = self.root() {
            let mut stack = Vec::with_capacity(self.len());
            stack.push(root);
            DfsPreIter {
                stack,
                marked: HashSet::with_capacity(self.len()),
                graph: &self,
            }
        } else {
            DfsPreIter {
                stack: Vec::with_capacity(0),
                marked: HashSet::with_capacity(0),
                graph: &self,
            }
        }
    }

    /// Visits the graph nodes using a depth-first search in post-order.
    ///
    /// Returns an iterator visiting every node reachable from [Graph::root()] using a
    /// depth-first post-order visit.
    ///
    /// In the default implementation this visit is iterative.
    fn dfs_postorder(&self) -> DfsPostIter<'_, Self>
    where
        Self: Sized,
    {
        if let Some(root) = self.root() {
            let mut stack = Vec::with_capacity(self.len());
            stack.push(root);
            DfsPostIter {
                stack,
                buffer: VecDeque::new(),
                marked: HashSet::with_capacity(self.len()),
                graph: &self,
            }
        } else {
            DfsPostIter {
                stack: Vec::with_capacity(0),
                buffer: VecDeque::with_capacity(0),
                marked: HashSet::with_capacity(0),
                graph: &self,
            }
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
        Self: Sized,
    {
        let mut pmap = HashMap::with_capacity(self.len());
        let visit = self.dfs_preorder();
        for node in visit {
            // the next line is used to have a set for each node. Otherwise nodes with no children
            // will never get their entry inside the map.
            pmap.entry(node).or_insert_with(HashSet::new);
            for nbor in self.neighbours(node) {
                let child_map = pmap.entry(nbor).or_insert_with(HashSet::new);
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
        Self: Sized,
    {
        // assign indices to everything (array indexing will be used a lot in this method)
        let ids = self
            .dfs_preorder()
            .enumerate()
            .map(|(index, item)| (item, index))
            .collect::<HashMap<_, _>>();
        let mut adj = vec![Vec::new(); ids.len()];
        for (node, index) in &ids {
            let neighbours = self
                .neighbours(node)
                .iter()
                .flat_map(|x| ids.get(x))
                .copied()
                .collect();
            adj[*index] = neighbours;
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

/// An iterator that performs a Breadth-First visit of a graph.
///
/// This iterator is created from [Graph::bfs].
pub struct BfsIter<'a, G: Graph> {
    queue: VecDeque<&'a G::Item>,
    buffer: VecDeque<&'a G::Item>,
    marked: HashSet<&'a G::Item>,
    graph: &'a G,
}

impl<'a, G: Graph> Iterator for BfsIter<'a, G> {
    type Item = &'a G::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.queue.is_empty() && self.buffer.is_empty() {
            if let Some(node) = self.queue.pop_front() {
                for nbor in self.graph.neighbours(node) {
                    if !self.marked.contains(nbor) {
                        self.marked.insert(nbor);
                        self.queue.push_back(nbor);
                        self.buffer.push_back(nbor);
                    }
                }
            }
        }
        self.buffer.pop_front()
    }
}

/// An iterator that performs a preorder Depth-First visit of a graph.
///
/// This iterator is created from [Graph::dfs_preorder].
pub struct DfsPreIter<'a, G: Graph> {
    stack: Vec<&'a G::Item>,
    marked: HashSet<&'a G::Item>,
    graph: &'a G,
}

impl<'a, G: Graph> Iterator for DfsPreIter<'a, G> {
    type Item = &'a G::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.stack.pop() {
            let retval = current;
            self.marked.insert(current);
            for nbor in self.graph.neighbours(current).iter().rev() {
                if !self.marked.contains(nbor) {
                    self.marked.insert(nbor);
                    self.stack.push(nbor);
                }
            }
            Some(retval)
        } else {
            None
        }
    }
}

/// An iterator that performs a postorder Depth-First visit of a graph.
///
/// This iterator is created from [Graph::dfs_postorder].
pub struct DfsPostIter<'a, G: Graph> {
    stack: Vec<&'a G::Item>,
    buffer: VecDeque<&'a G::Item>,
    marked: HashSet<&'a G::Item>,
    graph: &'a G,
}

impl<'a, G: Graph> Iterator for DfsPostIter<'a, G> {
    type Item = &'a G::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.stack.is_empty() && self.buffer.is_empty() {
            if let Some(current) = self.stack.last() {
                let mut to_push = Vec::new();
                self.marked.insert(current);
                for nbor in self.graph.neighbours(current).iter().rev() {
                    if !self.marked.contains(nbor) {
                        self.marked.insert(nbor);
                        to_push.push(nbor);
                    }
                }
                // if all children has been processed, return current node
                if to_push.is_empty() {
                    let current = self.stack.pop().unwrap();
                    self.buffer.push_back(current);
                } else {
                    self.stack.append(&mut to_push);
                }
            }
        }
        self.buffer.pop_front()
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
#[derive(Clone)]
pub struct DirectedGraph<T> {
    /// root of the graph (if rooted and not empty)
    pub root: Option<T>,
    // neighbour for a given node in the graph.
    pub adjacency: HashMap<T, Vec<T>>,
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
        self.root.as_ref()
    }

    fn neighbours(&self, node: &Self::Item) -> &[Self::Item] {
        if let Some(neighbours) = self.adjacency.get(node) {
            neighbours
        } else {
            &[]
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

    fn sample() -> DirectedGraph<u8> {
        let nodes = (0..).take(7).collect::<Vec<_>>();
        let mut graph = DirectedGraph {
            root: Some(nodes[0]),
            adjacency: HashMap::new(),
        };
        graph.adjacency.insert(nodes[0], vec![nodes[1], nodes[2]]);
        graph.adjacency.insert(nodes[1], vec![nodes[6]]);
        graph.adjacency.insert(nodes[2], vec![nodes[3], nodes[4]]);
        graph.adjacency.insert(nodes[3], vec![nodes[5]]);
        graph.adjacency.insert(nodes[4], vec![nodes[5]]);
        graph.adjacency.insert(nodes[5], vec![nodes[6]]);
        graph.adjacency.insert(nodes[6], vec![]);
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
        let graph = sample();
        let node = graph.root();
        assert!(node.is_some());
        assert_eq!(*node.unwrap(), 0)
    }

    #[test]
    fn directed_graph_children_empty() {
        let graph: DirectedGraph<u8> = DirectedGraph::default();
        let children = graph.neighbours(&0);
        assert!(children.is_empty());
    }

    #[test]
    fn directed_graph_children_existing() {
        let graph = sample();
        let children = graph.neighbours(graph.root().unwrap());
        assert_eq!(children[0], 1);
    }

    #[test]
    fn directed_graph_len() {
        let graph = sample();
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
        let graph = sample();
        assert!(!graph.is_empty());
    }

    #[test]
    fn bfs_empty() {
        let graph: DirectedGraph<u8> = DirectedGraph::default();
        let order = graph.bfs();
        assert_eq!(order.count(), 0);
    }

    #[test]
    fn bfs() {
        let graph = sample();
        let expected = vec![0, 1, 2, 6, 3, 4, 5];
        for (index, val) in graph.bfs().enumerate() {
            assert_eq!(*val, expected[index]);
        }
    }

    #[test]
    fn dfs_preorder_empty() {
        let graph: DirectedGraph<u8> = DirectedGraph::default();
        let order = graph.dfs_preorder();
        assert_eq!(order.count(), 0);
    }

    #[test]
    fn dfs_preorder() {
        let graph = sample();
        let expected = vec![0, 1, 6, 2, 3, 5, 4];
        for (index, val) in graph.dfs_preorder().enumerate() {
            assert_eq!(*val, expected[index]);
        }
    }

    #[test]
    fn dfs_postorder_empty() {
        let graph: DirectedGraph<u8> = DirectedGraph::default();
        let order = graph.dfs_postorder();
        assert_eq!(order.count(), 0);
    }

    #[test]
    fn dfs_postorder() {
        let graph = sample();
        let expected = vec![6, 1, 5, 3, 4, 2, 0];
        for (index, val) in graph.dfs_postorder().enumerate() {
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
        let graph = sample();
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
        let mut graph = sample();
        // edit the graph to introduce a cycle
        let node2 = *(graph.adjacency.get_key_value(&2).unwrap().0);
        let node4 = *(graph.adjacency.get_key_value(&4).unwrap().0);
        let node5 = *(graph.adjacency.get_key_value(&5).unwrap().0);
        let node6 = *(graph.adjacency.get_key_value(&6).unwrap().0);
        graph.adjacency.insert(node4, vec![node2]);
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
