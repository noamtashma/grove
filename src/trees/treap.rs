//! Implementation of treaps
//!
//! It is a balanced tree algorithm that supports reversals, splitting and concatenation.
//!
//! Its operations take `O(log n)` expected time, probabilistically.
//! Each operation may take up to linear time, but the probability of any operation
//! taking more than `O(log n)` time  is extremely low.
//!
//! The tree rebalances by assigning random priorities to nodes, and ensuring
//! They are in the correct structure mandated byy thos priorities.
//!
//! The tree's structure is completely independent of the actions that were performed on it.

use crate::locators;

use super::basic_tree::*;
use super::*;
use rand;

// The type that is used for bookkeeping.
// convention: a bigger number should go higher up the tree.
type T = u64;

/// A Treap.
pub struct Treap<D: Data> {
    tree: BasicTree<D, T>,
}

impl<D: Data> SomeTree<D> for Treap<D> {
    fn segment_summary<L>(&mut self, locator: L) -> D::Summary
    where
        L: crate::Locator<D>,
    {
        methods::segment_summary(self, locator)
    }

    fn act_segment<L>(&mut self, action: D::Action, locator: L)
    where
        L: crate::Locator<D>,
    {
        if action.to_reverse() == false {
            methods::act_segment(self, action, locator)
        } else {
            // split out the middle
            let mut mid: Treap<D> = self
                .slice(locators::LeftEdgeOf(locator.clone()))
                .split_right()
                .unwrap();

            let mut walker2 = TreapWalker {
                walker: BasicWalker::new_with_context(
                    &mut mid.tree,
                    self.subtree_summary(),
                    Default::default(),
                ),
            };
            methods::search_subtree(&mut walker2, locators::RightEdgeOf(locator));
            let right = walker2.split_right().unwrap();
            drop(walker2);

            // apply action
            mid.act_subtree(action);

            // glue back together
            mid.concatenate_right(right);
            self.concatenate_right(mid);
        }
    }

    type TreeData = T;
    fn iter_locator<'a, L: locators::Locator<D>>(
        &'a mut self,
        locator: L,
    ) -> basic_tree::iterators::IterLocator<'a, D, L, T> {
        iterators::IterLocator::new(&mut self.tree, locator)
    }

    /// Checks that invariants remain correct. i.e., that every node's summary
    /// is the sum of the summaries of its children, and that the priorities are ordered.
    /// If it finds any violation, it panics.
    fn assert_correctness(&self)
    where
        D::Summary: Eq,
    {
        self.tree.assert_correctness_with(|node| {
            Self::assert_priorities_locally_internal(node);
            node.assert_correctness_locally();
        });
    }
}

impl<D: Data> Default for Treap<D> {
    fn default() -> Self {
        Treap::new()
    }
}

impl<'a, D: Data> SomeTreeRef<D> for &'a mut Treap<D> {
    type Walker = TreapWalker<'a, D>;

    fn walker(self) -> Self::Walker {
        TreapWalker {
            walker: self.tree.walker(),
        }
    }
}

impl<'a, D: Data> ModifiableTreeRef<D> for &'a mut Treap<D> {
    type ModifiableWalker = TreapWalker<'a, D>;
}

impl<D: Data> SomeEntry<D> for Treap<D> {
    fn with_value<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut D::Value) -> R,
    {
        self.tree.with_value(f)
    }

    fn node_summary(&self) -> D::Summary {
        self.tree.node_summary()
    }

    fn subtree_summary(&self) -> D::Summary {
        self.tree.subtree_summary()
    }

    fn left_subtree_summary(&self) -> Option<D::Summary> {
        self.tree.left_subtree_summary()
    }

    fn right_subtree_summary(&self) -> Option<D::Summary> {
        self.tree.right_subtree_summary()
    }

    fn act_subtree(&mut self, action: D::Action) {
        self.tree.act_subtree(action);
    }

    fn act_node(&mut self, action: D::Action) -> Option<()> {
        self.tree.act_node(action)
    }

    fn act_left_subtree(&mut self, action: D::Action) -> Option<()> {
        self.tree.act_left_subtree(action)
    }

    fn act_right_subtree(&mut self, action: D::Action) -> Option<()> {
        self.tree.act_right_subtree(action)
    }

    fn assert_correctness_locally(&self)
    where
        D::Summary: Eq,
    {
        if let Some(node) = self.tree.node() {
            Self::assert_priorities_locally_internal(&node);
            node.assert_correctness_locally();
        }
    }
}

impl<D: Data> BasicTree<D, T> {
    /// Returns the node's priority.
    pub fn priority(&self) -> Option<T> {
        Some(self.node()?.alg_data)
    }
}

impl<D: Data> Treap<D> {
    /// Creates an empty treap.
    pub fn new() -> Treap<D> {
        Treap {
            tree: BasicTree::Empty,
        }
    }

    /// Returns the root's priority.
    /// Returns [`None`] if the tree is empty.
    pub fn priority(&self) -> Option<T> {
        self.tree.priority()
    }

    /// Computes the union of two splay trees, ordered by keys.
    /// We order the resulting tree based on the `D::Value: Keyed` instance, assuming that
    /// the values in the existing trees are also in the correct order.
    /// This is different from concatenate, because concatenate puts first all elements of the first tree,
    /// and then all of the elements of the second tree.
    ///
    /// If elements with equal keys are found, they are placed in an arbitrary order.
    ///
    /// # Complexity
    /// If the sizes of the two trees are `n,k`, with `n < k`, then the complexity is
    /// `O(n*log(1+k/n))` in the average case. It is aolso equal to `O(log(n+k 'choose' n))`.
    /// This has the effect that if you start with `n` different singletone trees,
    /// and you united them together in any way whatsoever, the overall complexity would be
    /// `O(n*log(n))`.
    pub fn union(&mut self, tree2: Treap<D>)
    where
        D::Value: Keyed,
    {
        union_internal(&mut self.tree, tree2);
    }

    /// Asserts that the priorities maintain the priority invariant
    /// at the current node.
    /// Panics otherwise.
    pub fn assert_priorities_locally(&self) {
        if let Some(node) = self.tree.node() {
            Self::assert_priorities_locally_internal(&node);
        }
    }

    fn assert_priorities_locally_internal(node: &BasicNode<D, T>) {
        if let Some(left) = node.left.node() {
            assert!(node.alg_data() > left.alg_data());
        }
        if let Some(right) = node.right.node() {
            assert!(node.alg_data() > right.alg_data());
        }
    }

    /// Asserts that the priorities maintain the priority invariant.
    /// Panics otherwise.
    pub fn assert_priorities(&self) {
        self.tree
            .assert_correctness_with(Self::assert_priorities_locally_internal);
    }
}

impl<D: Data> std::iter::FromIterator<D::Value> for Treap<D> {
    /// This takes [`O(n)`] worst-case time.
    fn from_iter<T: IntoIterator<Item = D::Value>>(iter: T) -> Self {
        // TODO: write a specific instantiation instead of calling insert,
        // because we know that we're not using all of insert's generality.
        let mut tree = Treap {
            tree: BasicTree::Empty,
        };
        let mut walker = tree.walker();
        for val in iter {
            walker.insert(val).unwrap();
            // note that it can only go right once
            while let Ok(()) = walker.go_right() {}
        }
        drop(walker);
        tree
    }
}

impl<D: Data> IntoIterator for Treap<D> {
    type Item = D::Value;
    type IntoIter = iterators::IntoIter<D, std::ops::RangeFull, T>;

    fn into_iter(self) -> Self::IntoIter {
        iterators::IntoIter::new(self.tree, ..)
    }
}

/// A walker for a [`Treap`].
pub struct TreapWalker<'a, D: Data> {
    walker: BasicWalker<'a, D, T>,
}

impl<'a, D: Data> SomeWalker<D> for TreapWalker<'a, D> {
    fn go_left(&mut self) -> Result<(), ()> {
        self.walker.go_left()
    }

    fn go_right(&mut self) -> Result<(), ()> {
        self.walker.go_right()
    }

    fn go_up(&mut self) -> Result<Side, ()> {
        self.walker.go_up()
    }

    fn depth(&self) -> usize {
        self.walker.depth()
    }

    fn far_left_summary(&self) -> D::Summary {
        self.walker.far_left_summary()
    }

    fn far_right_summary(&self) -> D::Summary {
        self.walker.far_left_summary()
    }

    fn value(&self) -> Option<&D::Value> {
        self.walker.value()
    }
}

impl<'a, D: Data> SomeEntry<D> for TreapWalker<'a, D> {
    fn with_value<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut D::Value) -> R,
    {
        self.walker.with_value(f)
    }

    fn node_summary(&self) -> D::Summary {
        self.walker.node_summary()
    }

    fn subtree_summary(&self) -> D::Summary {
        self.walker.subtree_summary()
    }

    fn left_subtree_summary(&self) -> Option<D::Summary> {
        self.walker.left_subtree_summary()
    }

    fn right_subtree_summary(&self) -> Option<D::Summary> {
        self.walker.right_subtree_summary()
    }

    fn act_subtree(&mut self, action: D::Action) {
        self.walker.act_subtree(action);
    }

    fn act_node(&mut self, action: D::Action) -> Option<()> {
        self.walker.act_node(action)
    }

    fn act_left_subtree(&mut self, action: D::Action) -> Option<()> {
        self.walker.act_left_subtree(action)
    }

    fn act_right_subtree(&mut self, action: D::Action) -> Option<()> {
        self.walker.act_right_subtree(action)
    }

    fn assert_correctness_locally(&self)
    where
        D::Summary: Eq,
    {
        self.walker.assert_correctness_locally();
        if let Some(node) = self.walker.node() {
            Treap::assert_priorities_locally_internal(&node);
        }
    }
}

impl<'a, D: Data> TreapWalker<'a, D> {
    /// Returns the priority of the current node. Lower numbers means
    /// The node is closer to the root.
    pub fn priority(&self) -> Option<T> {
        self.walker.inner().priority()
    }

    pub(super) fn inner_mut(&mut self) -> &mut BasicTree<D, T> {
        self.walker.inner_mut()
    }
}

impl<'a, D: Data> ModifiableWalker<D> for TreapWalker<'a, D> {
    /// Inserts the value into the tree at the current empty position.
    /// If the current position is not empty, return [`None`].
    /// When the function returns, the walker will be at the position the node
    /// was inserted.
    fn insert(&mut self, val: D::Value) -> Option<()> {
        if !self.is_empty() {
            return None;
        }

        let priority: T = rand::random();
        let mut temp = BasicTree::Empty;
        // in the first round, this value is irrelevent. choosing this will skip the first if.
        let mut prev_side = self.walker.is_left_son().unwrap_or(Side::Right);
        while let Ok(side) = self.go_up() {
            if self.priority().unwrap() > priority {
                // move to the position in which the node should be inserted
                // then break. insertion happens after the break outside the loop.
                match side {
                    Side::Left => self.walker.go_left().unwrap(),
                    Side::Right => self.walker.go_right().unwrap(),
                }
                break;
            }
            if self.priority().unwrap() == priority {
                eprintln!("found equal priorities")
            }
            if prev_side != side {
                let node = self.walker.node_mut().unwrap();
                let son = match side {
                    Side::Left => &mut node.left,
                    Side::Right => &mut node.right,
                };
                std::mem::swap(&mut temp, son);
            }
            self.walker.rebuild();
            prev_side = side;
        }

        // insert the new node, at the current position.
        let mut new: BasicNode<D, _> = BasicNode::new_alg(val, priority);

        match prev_side {
            Side::Left => {
                new.left = temp;
                new.right = std::mem::replace(self.walker.inner_mut(), BasicTree::Empty);
            }
            Side::Right => {
                new.right = temp;
                new.left = std::mem::replace(self.walker.inner_mut(), BasicTree::Empty);
            }
        }
        new.rebuild();
        *self.walker.inner_mut() = BasicTree::from_node(new);
        Some(())
    }

    /// Removes the current value from the tree, and returns it.
    /// If currently at an empty position, returns [`None`].
    /// The walker stays in the same position, and only the current node's subtree changes.
    fn delete(&mut self) -> Option<D::Value> {
        let tree = std::mem::replace(self.walker.inner_mut(), BasicTree::Empty);
        let node = tree.into_node()?;
        let left = Treap { tree: node.left };
        let right = Treap { tree: node.right };
        *self.walker.inner_mut() = ConcatenableTree::concatenate(left, right).tree;
        Some(node.node_value)
    }
}

/// Computes the union of two splay trees, ordered by keys.
/// We order the resulting tree based on the `D::Value: Keyed` instance, assuming that
/// the values in the existing trees are also in the correct order.
/// This is different from concatenate, because concatenate puts first all elements of the first tree,
/// and then all of the elements of the second tree.
///
/// If elements with equal keys are found, they are placed in an arbitrary order.
///
/// # Complexity
/// If the sizes of the two trees are `n,k`, with `n < k`, then the complexity is
/// `O(n*log(1+k/n))` in the average case. It is aolso equal to `O(log(n+k 'choose' n))`.
/// This has the effect that if you start with `n` different singletone trees,
/// and you united them together in any way whatsoever, the overall complexity would be
/// `O(n*log(n))`.
fn union_internal<D: Data>(tree1: &mut BasicTree<D, T>, mut tree2: Treap<D>)
where
    D::Value: Keyed,
{
    if tree2.is_empty() {
        return;
    }
    if tree1.is_empty() {
        *tree1 = tree2.tree;
        return;
    }
    if tree1.priority().unwrap() < tree2.priority().unwrap() {
        std::mem::swap(tree1, &mut tree2.tree);
    }
    let node = tree1.node_mut().unwrap();

    // formulation with less calls to panic, but less elegant
    /*
    let node = match (tree1.node_mut(), tree2.priority()) {
        (None, _) => { *tree1 = tree2.tree; return; },
        (_, None) => { return; },
        (Some(node), Some(priority)) => {
            if *node.alg_data() > priority {
                std::mem::swap(tree1, &mut tree2.tree);
                tree1.node_mut().unwrap()
            } else {
                node
            }
        },
    };
    */

    let key = node.node_value().get_key(); // this performs access()

    // TODO: replace by a locator that does the handling of the equality case by itself
    let mut split_walker = methods::search(&mut tree2, locators::ByKey((key,)));
    // if an element with the same key was found, arbitrarily decide to put it more to the right
    if split_walker.is_empty() == false {
        methods::previous_empty(&mut split_walker).unwrap();
    }
    // split
    let right = split_walker.split_right().unwrap();
    drop(split_walker);
    let left = tree2;

    // TODO: nice possible location for parrallelization
    union_internal(&mut node.left, left);
    union_internal(&mut node.right, right);
    node.rebuild();
}

#[cfg(feature = "async_union")]
use async_recursion::async_recursion;

#[cfg(feature = "async_union")]
/// Computes the union of two splay trees, ordered by keys.
/// We order the resulting tree based on the `D::Value: Keyed` instance, assuming that
/// the values in the existing trees are also in the correct order.
/// This is different from concatenate, because concatenate puts first all elements of the first tree,
/// and then all of the elements of the second tree.
///
/// If elements with equal keys are found, they are placed in an arbitrary order.
///
/// # Complexity
/// If the sizes of the two trees are `n,k`, with `n < k`, then the complexity is
/// `O(n*log(1+k/n))` in the average case. It is aolso equal to `O(log(n+k 'choose' n))`.
/// This has the effect that if you start with `n` different singletone trees,
/// and you united them together in any way whatsoever, the overall complexity would be
/// `O(n*log(n))`.
#[async_recursion]
pub async fn union_internal_concurrent<D: Data>(tree1: &mut BasicTree<D, T>, mut tree2: Treap<D>)
where
    D::Value: Keyed,
    <D::Value as Keyed>::Key: Sync,
    D::Action: Send,
    D::Summary: Send,
    D::Value: Send,
{
    if tree2.is_empty() {
        return;
    }
    if tree1.is_empty() {
        *tree1 = tree2.tree;
        return;
    }
    if tree1.priority().unwrap() < tree2.priority().unwrap() {
        std::mem::swap(tree1, &mut tree2.tree);
    }
    let node = tree1.node_mut().unwrap();

    // formulation with less calls to panic, but less elegant
    /*
    let node = match (tree1.node_mut(), tree2.priority()) {
        (None, _) => { *tree1 = tree2.tree; return; },
        (_, None) => { return; },
        (Some(node), Some(priority)) => {
            if *node.alg_data() > priority {
                std::mem::swap(tree1, &mut tree2.tree);
                tree1.node_mut().unwrap()
            } else {
                node
            }
        },
    };
    */

    let key = node.node_value().get_key(); // this performs access()

    // TODO: replace by a locator that does the handling of the equality case by itself
    let mut split_walker = methods::search(&mut tree2, locators::ByKey((key,)));
    // if an element with the same key was found, arbitrarily decide to put it more to the right
    if split_walker.is_empty() == false {
        methods::previous_empty(&mut split_walker).unwrap();
    }
    // split
    let right = split_walker.split_right().unwrap();
    drop(split_walker);
    let left = tree2;

    futures::join!(
        union_internal_concurrent(&mut node.left, left),
        union_internal_concurrent(&mut node.right, right)
    );
    node.rebuild();
}

#[cfg(feature = "async_union")]
/// Computes the union of two splay trees, ordered by keys.
/// We order the resulting tree based on the `D::Value : Keyed` instance, assuming that
/// the values in the existing trees are also in the correct order.
/// This is different from concatenate, because concatenate puts first all elements of the first tree,
/// and then all of the elements of the second tree.
///
/// If elements with equal keys are found, they are placed in an arbitrary order.
///
///```rust
///use orchard::*;
///use orchard::treap::*;
///use orchard::example_data::{NoAction, Ordered};
///
///type T = Treap<NoAction<Ordered<i32>>>;
///let tree1: T = (0..7).map(|x| Ordered(x)).collect();
///let tree2: T = (4..9).map(|x| Ordered(x)).collect();
///let tree = tokio_test::block_on(union_concurrent(tree1, tree2));
/// # tree.assert_correctness();
///assert_eq!(
///    tree.into_iter().collect::<Vec<_>>(),
///    [0,1,2,3,4,4,5,5,6,6,7,8].iter().map(|x| Ordered(*x)).collect::<Vec<_>>()
///);
///```
///
/// # Complexity
/// If the sizes of the two trees are `n,k`, with `n < k`, then the complexity is
/// `O(n*log(1+k/n))` in the average case. It is aolso equal to `O(log(n+k 'choose' n))`.
/// This has the effect that if you start with `n` different singletone trees,
/// and you united them together in any way whatsoever, the overall complexity would be
/// `O(n*log(n))`.
pub async fn union_concurrent<D: Data>(mut tree1: Treap<D>, tree2: Treap<D>) -> Treap<D>
where
    D::Value: Keyed,
    <D::Value as Keyed>::Key: Sync,
    D::Action: Send,
    D::Summary: Send,
    D::Value: Send,
{
    union_internal_concurrent(&mut tree1.tree, tree2).await;
    tree1
}

/// Computes the union of two splay trees, ordered by keys.
/// We order the resulting tree based on the `D::Value : Keyed` instance, assuming that
/// the values in the existing trees are also in the correct order.
/// This is different from concatenate, because concatenate puts first all elements of the first tree,
/// and then all of the elements of the second tree.
///
/// If elements with equal keys are found, they are placed in an arbitrary order.
///
///```rust
///use orchard::*;
///use orchard::treap::*;
///use orchard::example_data::{NoAction, Ordered};
///
///type T = Treap<NoAction<Ordered<i32>>>;
///let tree1: T = (0..7).map(|x| Ordered(x)).collect();
///let tree2: T = (4..9).map(|x| Ordered(x)).collect();
///let tree = union(tree1, tree2);
/// # tree.assert_correctness();
///assert_eq!(tree.into_iter().collect::<Vec<_>>(), [0,1,2,3,4,4,5,5,6,6,7,8].iter().map(|x| Ordered(*x)).collect::<Vec<_>>());
///```
///
/// # Complexity
/// If the sizes of the two trees are `n,k`, with `n < k`, then the complexity is
/// `O(n*log(1+k/n))` in the average case. It is aolso equal to `O(log(n+k 'choose' n))`.
/// This has the effect that if you start with `n` different singletone trees,
/// and you united them together in any way whatsoever, the overall complexity would be
/// `O(n*log(n))`.
pub fn union<D: Data>(mut tree1: Treap<D>, tree2: Treap<D>) -> Treap<D>
where
    D::Value: Keyed,
{
    tree1.union(tree2);
    tree1
}

impl<D: Data> ConcatenableTree<D> for Treap<D> {
    /// Concatenates the trees together, in place.
    ///```
    /// use orchard::trees::*;
    /// use orchard::treap::*;
    /// use orchard::example_data::StdNum;
    ///
    /// let mut tree: Treap<StdNum> = (17..=89).collect();
    /// let tree2: Treap<StdNum> = (13..=25).collect();
    /// tree.concatenate_right(tree2);
    ///
    /// assert_eq!(tree.iter().cloned().collect::<Vec<_>>(), (17..=89).chain(13..=25).collect::<Vec<_>>());
    /// # tree.assert_correctness();
    ///```
    fn concatenate_right(&mut self, tree2: Treap<D>) {
        let mut walker = self.walker();
        let mut tree_r = tree2.tree;

        // if we don't access here, then tree_r might be swapped into the walker
        // (in the first std::mem::swap) when it's not in a clean state, which is an assumed invariant.
        // this can mess up things, especially when reversals are present.
        tree_r.access();
        loop {
            match (walker.priority(), tree_r.priority()) {
                (None, _) => {
                    *walker.walker.inner_mut() = tree_r;
                    break;
                }
                (_, None) => break,
                (Some(a), Some(b)) if a > b => {
                    walker.go_right().unwrap();
                }
                _ => {
                    std::mem::swap(walker.walker.inner_mut(), &mut tree_r);
                    walker.go_left().unwrap();
                    std::mem::swap(walker.walker.inner_mut(), &mut tree_r);
                }
            }
        }
        // the walker is responsible for going up the tree
        // and rebuilding all the nodes
    }
}

impl<'a, D: Data> SplittableTreeRef<D> for &'a mut Treap<D> {
    type T = Treap<D>;
    type SplittableWalker = TreapWalker<'a, D>;
}

impl<'a, D: Data> SplittableWalker<D> for TreapWalker<'a, D> {
    type T = Treap<D>;

    /// Will only do anything if the current position is empty.
    /// If it is empty, it will split the tree: the elements
    /// to the left will remain, and the elements to the right
    /// will be put in the new output tree.
    /// The walker will be at the root after this operation, if it succeeds.
    ///
    ///```
    /// use orchard::trees::*;
    /// use orchard::treap::*;
    /// use orchard::example_data::StdNum;
    /// use orchard::methods::*;
    ///
    /// let mut tree: Treap<StdNum> = (17..88).collect();
    /// let mut tree2 = tree.slice(7..7).split_right().unwrap();
    ///
    /// assert_eq!(tree.iter().cloned().collect::<Vec<_>>(), (17..24).collect::<Vec<_>>());
    /// assert_eq!(tree2.iter().cloned().collect::<Vec<_>>(), (24..88).collect::<Vec<_>>());
    /// # tree.assert_correctness();
    ///```
    fn split_right(&mut self) -> Option<Treap<D>> {
        if !self.is_empty() {
            return None;
        }

        let mut temp = BasicTree::Empty;
        // in the first round, this value is irrelevent. choosing this will skip the first if.
        let mut prev_side = self.walker.is_left_son().unwrap_or(Side::Right);

        while let Ok(side) = self.go_up() {
            if prev_side != side {
                let node = self.walker.node_mut().unwrap();
                let son = match side {
                    Side::Left => &mut node.left,
                    Side::Right => &mut node.right,
                };
                std::mem::swap(&mut temp, son);
                node.rebuild();
            }
            prev_side = side;
        }

        if prev_side == Side::Left {
            std::mem::swap(self.walker.inner_mut(), &mut temp);
        }
        Some(Treap { tree: temp })
    }

    /// Will only do anything if the current position is empty.
    /// If it is empty, it will split the tree: the elements
    /// to the left will remain, and the elements to the right
    /// will be put in the new output tree.
    /// The walker will be at the root after this operation, if it succeeds.
    ///
    ///```
    /// use orchard::trees::*;
    /// use orchard::treap::*;
    /// use orchard::example_data::StdNum;
    /// use orchard::methods::*;
    ///
    /// let mut tree: Treap<StdNum> = (17..88).collect();
    /// let mut tree2 = tree.slice(7..7).split_left().unwrap();
    ///
    /// assert_eq!(tree2.iter().cloned().collect::<Vec<_>>(), (17..24).collect::<Vec<_>>());
    /// assert_eq!(tree.iter().cloned().collect::<Vec<_>>(), (24..88).collect::<Vec<_>>());
    /// # tree.assert_correctness();
    ///```
    fn split_left(&mut self) -> Option<Self::T> {
        let mut right = self.split_right()?;
        std::mem::swap(self.inner_mut(), &mut right.tree);
        Some(right)
    }
}
