use super::*;
use locators::LocResult;

enum Fragment<'a, D: Data, T: Clone = ()>
where
    D::Value: Clone,
{
    Value(&'a mut D::Value),
    Node(&'a mut PersistentNode<D, T>),
}

/// Mutable iterator iterating over a segment of the tree. Since it is a mutable
/// iterator, the tree will probably not be in a legal state if the values are modified.
/// please use walkers instead.
///
/// We would've likes to rebuild the nodes as we change them. However, because rust's iterators
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
/// Instead, this type is wrapped inside the `IterLocator` type, which is exported.
struct IterLocatorMut<'a, D: Data, L, T:Clone = ()>
where
    D::Value: Clone,
{
    left: D::Summary,
    // a stack of the fragments, and for every fragment,
    // the summary of everything to its right
    stack: Vec<(Fragment<'a, D, T>, D::Summary)>,
    locator: L,
}

impl<'a, D: Data, L, T: Clone> IterLocatorMut<'a, D, L, T>
where
    D::Value: Clone,
{
    pub fn new(tree: &'a mut PersistentTree<D, T>, locator: L) -> Self {
        let mut res = IterLocatorMut {
            left: Default::default(),
            stack: vec![],
            locator,
        };
        res.push(tree, Default::default());
        res
    }

    /// Internal method: same as stack.push(...), but deals with the [`Empty`] case.
    /// If empty, do nothing.
    fn push(&mut self, tree: &'a mut PersistentTree<D, T>, summary: D::Summary) {
        if let Some(node) = tree.node_mut() {
            self.stack.push((Fragment::Node(node), summary));
        }
    }
}

impl<'a, D: Data, L: Locator<D>, T: Clone> Iterator for IterLocatorMut<'a, D, L, T>
where
    D::Value: Clone,
{
    type Item = &'a mut D::Value;

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Iterator is empty
        if self.stack.is_empty() {
            (0, Some(0))
        } else {
            // We know that every stack fragment contains at least one element.
            // We don't know any upper bound.
            // If we could specialize for `D: SizedData`, we could know the exact size,
            // but we can't.
            (self.stack.len(), None)
        }
    }

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
                    self.left = self.left + (*val).to_summary();
                    return Some(val);
                }
                Fragment::Node(node) => node,
            };

            node.access();
            let value = &mut node.node_value;
            let right_node = &mut node.right;
            let left_node = &mut node.left;

            let value_summary = (*value).to_summary();
            let near_left_summary: D::Summary = self.left + left_node.subtree_summary();
            let near_right_summary: D::Summary = right_node.subtree_summary() + summary;

            let dir = self
                .locator
                .locate(near_left_summary, value, near_right_summary);
            match dir {
                LocResult::GoLeft => {
                    if !self.stack.is_empty() {
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
pub struct IterLocator<'a, D: Data, L, T: Clone = ()>
where
    D::Value: Clone,
{
    mut_iter: IterLocatorMut<'a, D, L, T>,
}

impl<'a, D: Data, L: Locator<D>, T: Clone> IterLocator<'a, D, L, T>
where
    D::Value: Clone,
{
    /// Creates a new immutable iterator for a segment of the given tree.
    pub fn new(tree: &'a mut PersistentTree<D, T>, locator: L) -> Self {
        IterLocator {
            mut_iter: IterLocatorMut::new(tree, locator),
        }
    }
}

impl<'a, D: Data, L: Locator<D>, T: Clone> Iterator for IterLocator<'a, D, L, T>
where
    D::Value: Clone,
{
    type Item = &'a D::Value;

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.mut_iter.size_hint()
    }

    /// Creates a new immutable iterator for a segment of the given tree.
    fn next(&mut self) -> Option<Self::Item> {
        Some(&*self.mut_iter.next()?)
    }
}

// Owning iterators make a small amount of sense for Persistent trees.
// If the nodes are owned only by this tree, then it's somewhat useful
// to not clone the values.

/// Owning fragment
enum OFragment<D: Data, T: Clone = ()>
where
    D::Value: Clone,
{
    Value(D::Value),
    Node(Rc<PersistentNode<D, T>>),
}
/// Owning iterator iterating over a segment of the tree.
pub struct IntoIter<D: Data, L, T: Clone = ()>
where
    D::Value: Clone,
{
    left: D::Summary,
    // a stack of the fragments, and for every fragment,
    // the summary of everything to its right
    stack: Vec<(OFragment<D, T>, D::Summary)>,
    locator: L,
}

impl<D: Data, L, T: Clone> IntoIter<D, L, T>
where
    D::Value: Clone,
{
    /// Creates a new owning iterator for a segment of the given tree.
    pub fn new(tree: PersistentTree<D, T>, locator: L) -> Self {
        let mut res = IntoIter {
            left: Default::default(),
            stack: vec![],
            locator,
        };
        res.push(tree, Default::default());
        res
    }

    /// Internal method: same as stack.push(...), but deals with the [`Empty`] case.
    /// If empty, do nothing.
    fn push(&mut self, tree: PersistentTree<D, T>, summary: D::Summary) {
        if let Some(rc_node) = tree.into_node_rc() {
            self.stack.push((OFragment::Node(rc_node), summary));
        }
    }
}

impl<D: Data, L: Locator<D>, T> Iterator for IntoIter<D, L, T>
where
    D::Value: Clone,
    T: Clone,
{
    type Item = D::Value;

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Iterator is empty
        if self.stack.is_empty() {
            (0, Some(0))
        } else {
            // We know that every stack fragment contains at least one element.
            // We don't know any upper bound.
            // If we could specialize for `D: SizedData`, we could know the exact size,
            // but we can't.
            (self.stack.len(), None)
        }
    }

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (frag, summary) = match self.stack.pop() {
                None => return None,
                Some(x) => x,
            };

            let node = match frag {
                // if value has been inserted to the stack, the locator has already been called
                // on it and returned `Accept`.
                OFragment::Value(val) => {
                    self.left = self.left + val.to_summary();
                    return Some(val);
                }
                OFragment::Node(node) => node,
            };

            let value: D::Value;
            let right_node: PersistentTree<D, T>;
            let left_node: PersistentTree<D, T>;

            match Rc::try_unwrap(node) {
                // no need to clone any Rc if we are the unique owner
                Ok(mut owned_node) => {
                    owned_node.access();
                    value = owned_node.node_value;
                    right_node = owned_node.right;
                    left_node = owned_node.left;
                },
                // no need to clone `node` if the action is the identity
                Err(node) if node.action.is_identity() => {
                    value = node.node_value.clone();
                    right_node = node.right.clone();
                    left_node = node.left.clone();
                },
                // we need to clone everything
                Err(mut node) => {
                    let node_ref = Rc::make_mut(&mut node);
                    node_ref.access();
                    value = node_ref.node_value.clone();
                    right_node = node_ref.right.clone();
                    left_node = node_ref.left.clone();
                },
            }
            
            let value_summary = value.to_summary();
            let near_left_summary: D::Summary = self.left + left_node.subtree_summary();
            let near_right_summary: D::Summary = right_node.subtree_summary() + summary;

            let dir = self
                .locator
                .locate(near_left_summary, &value, near_right_summary);
            match dir {
                LocResult::GoLeft => {
                    if !self.stack.is_empty() {
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
