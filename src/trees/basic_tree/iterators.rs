use crate::*;
use basic_tree::*;
use methods::LocResult;

// TODO : Owning iterator
enum Fragment<'a, D : Data, T=()> {
    Value (&'a mut D::Value),
    Node (&'a mut BasicNode<D, T>)
}

// TODO - fix the mutability problem using a guard?
/// Mutable iterator iterating over a segment of the tree. Since it is a mutable
/// iterator, the tree will probably not be in a legal state if the values are modified.
pub struct MutIterator<'a, D : Data, L, T=()> {
    left : D::Summary,
    // a stack of the fragments, and for every fragment,
    // the summary of everything to its right
    stack : Vec<(Fragment<'a, D, T>, D::Summary)>,
    locator : L,
}

impl<'a, D : Data, L, T> MutIterator<'a, D, L, T> {
    pub fn new(tree : &'a mut BasicTree<D, T>, locator : L) -> Self {
        let mut res = MutIterator {
            left : D::EMPTY,
            stack : vec![],
            locator
        };
        res.push(tree, D::EMPTY);
        res
    }

    /// Internal method: same as stack.push(...), but deals with the [`Empty`] case.
    /// If empty, do nothing.
    fn push(&mut self, tree : &'a mut BasicTree<D, T>, summary : D::Summary) {
        if let Some(node) = tree.node_mut() {
            self.stack.push((Fragment::Node(node), summary));
        }
    }
}

impl<'a, D : Data, L : Locator<D>, T> Iterator for MutIterator<'a, D, L, T> {
    type Item = &'a mut D::Value;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (frag, summary) = match self.stack.pop() {
                None => return None,
                Some(x) => x,
            };

            let node = match frag {
                // if value has been inserted to the stack, the locator has already been called
                // on it and returned `Accept`.
                Fragment::Value(val) => {
                    self.left = self.left + D::to_summary(val);
                    return Some(val); 
                },
                Fragment::Node(node) => {
                    node
                }
            };

            node.access();
            let value = &mut node.node_value;
            let right_node = &mut node.right;
            let left_node = &mut node.left;

            let value_summary = D::to_summary(value);
            let near_left_summary : D::Summary = self.left + left_node.subtree_summary();
            let near_right_summary : D::Summary = right_node.subtree_summary() + summary;

            let dir = self.locator.locate(near_left_summary, value, near_right_summary);
            match dir {
                LocResult::GoLeft => {
                    if self.stack.len() > 0 {
                        panic!("GoLeft received in the middle of a segment");
                    }
                    self.push(left_node, value_summary + near_right_summary);
                },
                LocResult::GoRight => {
                    self.push(right_node, summary);
                    self.left = near_left_summary + value_summary;
                },
                LocResult::Accept => {
                    self.push(right_node, summary);
                    self.stack.push((Fragment::Value(value), near_right_summary));
                    self.push(left_node, value_summary + near_right_summary);
                }
            }
        }
    }
}


/// Immutable iterator. Requires borrowing the tree as mutable.
/// An immutable iterator only requiring immutable access to the tree is possible,
/// just not easy to write, so it's not included currently.
pub struct ImmIterator<'a, D : Data, L, T=()> {
    mut_iter : MutIterator<'a, D, L, T>
}

impl<'a, D : Data, L : Locator<D>, T> ImmIterator<'a, D, L, T> {
    pub fn new(tree : &'a mut BasicTree<D, T>, locator : L) -> Self {
        ImmIterator {
            mut_iter : MutIterator::new(tree, locator)
        }
    }
}

impl<'a, D : Data, L : Locator<D>, T> Iterator for ImmIterator<'a, D, L, T> {
    type Item = &'a D::Value;

    fn next(&mut self) -> Option<Self::Item> {
        Some(&*self.mut_iter.next()?)
    }
}

/// Builds a well balanced [`BasicTree`] from an iterator of values.
pub fn build<D : Data, I>(mut iter : I) -> BasicTree<D> where
    I : Iterator<Item = D::Value>
{
    // the code is a bit messy because we don't know in advance the size of the tree.
    // the stack holds nodes, each of which has no right son, and a left son which is
    // a perfect binary tree. the trees correspond to the binary digits of `count`:
    // the i'th digit of `count` is `1` iff there is a tree in the stack of size `2^i`.
    let mut stack : Vec<BasicNode<D>> = vec![];
    let mut count = 0;
    while let Some(val) = iter.next() {
        let mut tree = BasicTree::Empty;
        for i in 0.. {
            if (count>>i) & 1 == 1 {
                let mut prev_node = stack.pop().unwrap();
                prev_node.right = tree;
                prev_node.rebuild();
                tree = BasicTree::new(prev_node);
            }
            else {
                let mut node = BasicNode::new(val);
                node.left = tree;
                stack.push(node);
                break;
            }
        }
        count += 1;
    }
    
    let mut tree = BasicTree::Empty;
    for mut prev_node in stack.into_iter().rev() {
        prev_node.right = tree;
        prev_node.rebuild();
        tree = BasicTree::new(prev_node);
    }
    tree
}


/// Owning fragment
enum OFragment<D : Data, T=()> {
    Value (D::Value),
    Node (Box<BasicNode<D, T>>)
}
/// Owning iterator iterating over a segment of the tree.
pub struct OwningIterator<D : Data, L, T=()> {
    left : D::Summary,
    // a stack of the fragments, and for every fragment,
    // the summary of everything to its right
    stack : Vec<(OFragment<D, T>, D::Summary)>,
    locator : L,
}

impl<D : Data, L, T> OwningIterator<D, L, T> {
    pub fn new(tree : BasicTree<D, T>, locator : L) -> Self {
        let mut res = OwningIterator {
            left : D::EMPTY,
            stack : vec![],
            locator
        };
        res.push(tree, D::EMPTY);
        res
    }

    /// Internal method: same as stack.push(...), but deals with the [`Empty`] case.
    /// If empty, do nothing.
    fn push(&mut self, tree : BasicTree<D, T>, summary : D::Summary) {
        if let BasicTree::Root(node) = tree {
            self.stack.push((OFragment::Node(node), summary));
        }
    }
}

impl<D : Data, L : Locator<D>, T> Iterator for OwningIterator<D, L, T> {
    type Item = D::Value;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (frag, summary) = match self.stack.pop() {
                None => return None,
                Some(x) => x,
            };

            let mut node = match frag {
                // if value has been inserted to the stack, the locator has already been called
                // on it and returned `Accept`.
                OFragment::Value(val) => {
                    self.left = self.left + D::to_summary(&val);
                    return Some(val); 
                },
                OFragment::Node(node) => {
                    node
                }
            };

            node.access();
            let value = node.node_value;
            let right_node = node.right;
            let left_node = node.left;

            let value_summary = D::to_summary(&value);
            let near_left_summary : D::Summary = self.left + left_node.subtree_summary();
            let near_right_summary : D::Summary = right_node.subtree_summary() + summary;

            let dir = self.locator.locate(near_left_summary, &value, near_right_summary);
            match dir {
                LocResult::GoLeft => {
                    if self.stack.len() > 0 {
                        panic!("GoLeft received in the middle of a segment");
                    }
                    self.push(left_node, value_summary + near_right_summary);
                },
                LocResult::GoRight => {
                    self.push(right_node, summary);
                    self.left = near_left_summary + value_summary;
                },
                LocResult::Accept => {
                    self.push(right_node, summary);
                    self.stack.push((OFragment::Value(value), near_right_summary));
                    self.push(left_node, value_summary + near_right_summary);
                }
            }
        }
    }
}