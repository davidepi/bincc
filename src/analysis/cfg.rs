use std::fmt::{Display, Formatter};

pub struct CFG {
    nodes: Vec<CFGNode>,
    edges: Vec<[Option<usize>; 2]>,
    root: usize,
}

pub struct CFGNode {
}

impl Display for CFG {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CFG({}, {})", self.nodes.len(), self.edges.len())
    }
}

pub struct CFGIter<'a> {
    stack: Vec<&'a CFGNode>,
}

impl<'a> Iterator for CFGIter<'a> {
    type Item = &'a CFGNode;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop()
    }
}

impl CFG {
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
}

#[cfg(test)]
mod tests {
    use crate::analysis::{CFGNode, CFG};

    #[test]
    fn preorder() {
        let nodes = (0..).take(7).map(|id| CFGNode { id }).collect();
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
}
