use crate::*;
use basic_tree::*;
use locators::LocResult;

// TODO : Owning iterator
enum Fragment<'a, D: Data, T = ()> {
    Value(&'a mut D::Value),
    Node(&'a mut BasicNode<D, T>),
}

/// Mutable iterator iterating over a segment of the tree. Since it is a mutable
/// iterator, the tree will probably not be in a legal state if the values are modified.
/// please use walkers instead.
///
/// We wopuld've likes to rebuild the nodes as we change them. However, because rust's iterators
/// are not streaming iterators, we can only rebuild the tree after the iterator finished. This is
/// inherently inefficient and unidiomatic, since most uses would only need a streaming iterator,
/// and could rebuild the nodes while iterating.
///
/// There are also two technical problems:
/// * Since the values could have changed, when we go back to rebuild the node,
///   the locator might select a different subsegment, and therefore we might not rebuild some of the nodes
///   we should have rebuilt.
/// * We need to convince the compiler that we can safely walk down the tree mutably a second time
///   after iteration finished. This can be sidestepped by a guard. Conveniently, the `Slice` struct can
///   be used as a guard, but this makes the implementation of `Slice` weird: after iterating mutably,
///   we can't use the slice again, instead we must drop it and make a new one.
///
///   This can almost be sidestepped by using a `RefCell`: the iterator would store a `RefCell` to the tree,
///   and the inner frames would store `RefMut` references into the tree. Then, when the iterator would be dropped,
///   all `RefMut`'s would be dropped and then we could get a fresh reference to the tree to rebuild it.
///   However, this doesn't work because
///   * The iterator could only give out `RefMut<'a, D::Value>` instead of regular references
///   * The `RefMut::split` function can only split a `RefMut` in two, but not in three. Even though
///     this could be trivially implemented in the standard library.
///
/// Therefore, this type isn't exposed - it can't be used productively.
/// Instead, this type is wrapped inside the `ImmIterator` type, which is exported.
struct MutIterator<'a, D: Data, L, T = ()> {
    left: D::Summary,
    // a stack of the fragments, and for every fragment,
    // the summary of everything to its right
    stack: Vec<(Fragment<'a, D, T>, D::Summary)>,
    locator: L,
}

impl<'a, D: Data, L, T> MutIterator<'a, D, L, T> {
    pub fn new(tree: &'a mut BasicTree<D, T>, locator: L) -> Self {
        let mut res = MutIterator {
            left: Default::default(),
            stack: vec![],
            locator,
        };
        res.push(tree, Default::default());
        res
    }

    /// Internal method: same as stack.push(...), but deals with the [`Empty`] case.
    /// If empty, do nothing.
    fn push(&mut self, tree: &'a mut BasicTree<D, T>, summary: D::Summary) {
        if let Some(node) = tree.node_mut() {
            self.stack.push((Fragment::Node(node), summary));
        }
    }
}

impl<'a, D: Data, L: Locator<D>, T> Iterator for MutIterator<'a, D, L, T> {
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
                }
                Fragment::Node(node) => node,
            };

            node.access();
            let value = &mut node.node_value;
            let right_node = &mut node.right;
            let left_node = &mut node.left;

            let value_summary = D::to_summary(value);
            let near_left_summary: D::Summary = self.left + left_node.subtree_summary();
            let near_right_summary: D::Summary = right_node.subtree_summary() + summary;

            let dir = self
                .locator
                .locate(near_left_summary, value, near_right_summary);
            match dir {
                LocResult::GoLeft => {
                    if self.stack.len() > 0 {
                        panic!("GoLeft received in the middle of a segment");
                    }
                    self.push(left_node, value_summary + near_right_summary);
                }
                LocResult::GoRight => {
                    self.push(right_node, summary);
                    self.left = near_left_summary + value_summary;
                }
                LocResult::Accept => {
                    self.push(right_node, summary);
                    self.stack
                        .push((Fragment::Value(value), near_right_summary));
                    self.push(left_node, value_summary + near_right_summary);
                }
            }
        }
    }
}

/// Immutable iterator.
/// The iterator receives a `&mut self` argument instead of a `&self` argument.
/// Because of the way the trees work, immutable iterators can't be written without either mutable access
/// to the tree, or assuming that the values are `Clone`.
///
/// That is essentially because the values stored in the tree still have actions that need to be performed in them
/// to get the current updated value.
///
/// If you use interior mutability to update the values inside the tree, and these changes affect the summaries,
/// the tree may behave incorrectly.
pub struct ImmIterator<'a, D: Data, L, T = ()> {
    mut_iter: MutIterator<'a, D, L, T>,
}

impl<'a, D: Data, L: Locator<D>, T> ImmIterator<'a, D, L, T> {
    /// Creates a new immutable iterator for a segment of the given tree.
    pub fn new(tree: &'a mut BasicTree<D, T>, locator: L) -> Self {
        ImmIterator {
            mut_iter: MutIterator::new(tree, locator),
        }
    }
}

impl<'a, D: Data, L: Locator<D>, T> Iterator for ImmIterator<'a, D, L, T> {
    type Item = &'a D::Value;

    fn next(&mut self) -> Option<Self::Item> {
        Some(&*self.mut_iter.next()?)
    }
}

/// Builds a well balanced [`BasicTree`] from an iterator of values.
pub fn build<D: Data, I>(mut iter: I) -> BasicTree<D>
where
    I: Iterator<Item = D::Value>,
{
    // TODO: rewrite using boxed nodes.
    // the code is a bit messy because we don't know in advance the size of the tree.
    // the stack holds nodes, each of which has no right son, and a left son which is
    // a perfect binary tree. the trees correspond to the binary digits of `count`:
    // the i'th digit of `count` is `1` iff there is a tree in the stack of size `2^i`.
    let mut stack: Vec<BasicNode<D>> = vec![];
    let mut count = 0;
    while let Some(val) = iter.next() {
        let mut tree = BasicTree::Empty;
        for i in 0.. {
            if (count >> i) & 1 == 1 {
                let mut prev_node = stack.pop().unwrap();
                prev_node.right = tree;
                prev_node.rebuild();
                tree = BasicTree::from_node(prev_node);
            } else {
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
        tree = BasicTree::from_node(prev_node);
    }
    tree
}

/// Owning fragment
enum OFragment<D: Data, T = ()> {
    Value(D::Value),
    Node(Box<BasicNode<D, T>>),
}
/// Owning iterator iterating over a segment of the tree.
pub struct OwningIterator<D: Data, L, T = ()> {
    left: D::Summary,
    // a stack of the fragments, and for every fragment,
    // the summary of everything to its right
    stack: Vec<(OFragment<D, T>, D::Summary)>,
    locator: L,
}

impl<D: Data, L, T> OwningIterator<D, L, T> {
    /// Creates a new owning iterator for a segment of the given tree.
    pub fn new(tree: BasicTree<D, T>, locator: L) -> Self {
        let mut res = OwningIterator {
            left: Default::default(),
            stack: vec![],
            locator,
        };
        res.push(tree, Default::default());
        res
    }

    /// Internal method: same as stack.push(...), but deals with the [`Empty`] case.
    /// If empty, do nothing.
    fn push(&mut self, tree: BasicTree<D, T>, summary: D::Summary) {
        if let Some(boxed_node) = tree.into_node_boxed() {
            self.stack.push((OFragment::Node(boxed_node), summary));
        }
    }
}

impl<D: Data, L: Locator<D>, T> Iterator for OwningIterator<D, L, T> {
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
                }
                OFragment::Node(node) => node,
            };

            node.access();
            let value = node.node_value;
            let right_node = node.right;
            let left_node = node.left;

            let value_summary = D::to_summary(&value);
            let near_left_summary: D::Summary = self.left + left_node.subtree_summary();
            let near_right_summary: D::Summary = right_node.subtree_summary() + summary;

            let dir = self
                .locator
                .locate(near_left_summary, &value, near_right_summary);
            match dir {
                LocResult::GoLeft => {
                    if self.stack.len() > 0 {
                        panic!("GoLeft received in the middle of a segment");
                    }
                    self.push(left_node, value_summary + near_right_summary);
                }
                LocResult::GoRight => {
                    self.push(right_node, summary);
                    self.left = near_left_summary + value_summary;
                }
                LocResult::Accept => {
                    self.push(right_node, summary);
                    self.stack
                        .push((OFragment::Value(value), near_right_summary));
                    self.push(left_node, value_summary + near_right_summary);
                }
            }
        }
    }
}
