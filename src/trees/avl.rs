//! Implementation of AVL trees.
//! Balanced by keeping track of node ranks, this is a worst-case balancing
//! Algorithm that has a small memory overhead per node.

use crate::locators;

use super::basic_tree::*;
use super::*;

/// The type that is used for rank bookkeeping.
/// `u8` is definitely enough, since the rank of the tree is logarithmic in the tree size.
type T = u8;
/// Used for rank differences
type TD = i8;

/// An AVL tree. Balanced by keeping track of node ranks, this is a worst-case balancing
/// Algorithm that has a small memory overhead per node.
pub struct AVLTree<D: Data> {
    tree: BasicTree<D, T>,
}

/// For implementing `rank`, `rank_diff` and `rebuild_ranks` for
/// trees, nodes and walkers alike.
trait Rankable {
    fn rank(&self) -> T;

    /// Returns `true` if the rank of the current node had to be updated,
    /// `false` if it was correct.
    fn rebuild_ranks(&mut self) -> bool;

    /// Returns `right.rank() - left.rank()`
    fn rank_diff(&self) -> TD;
}

impl<D: Data> Rankable for BasicTree<D, T> {
    fn rank(&self) -> T {
        match self.node() {
            None => 0,
            Some(node) => node.rank(),
        }
    }

    fn rebuild_ranks(&mut self) -> bool {
        if let Some(node) = self.node_mut() {
            node.rebuild_ranks()
        } else {
            true
        }
    }

    /// Returns `right.rank() - left.rank()`
    fn rank_diff(&self) -> TD {
        match self.node() {
            None => 0,
            Some(node) => node.rank_diff(),
        }
    }
}

impl<D: Data> Rankable for BasicNode<D, T> {
    fn rank(&self) -> T {
        *self.alg_data()
    }

    /// Returns `right.rank() - left.rank()`
    fn rank_diff(&self) -> TD {
        let diff = self.right.rank() as TD - self.left.rank() as TD;
        if self.action().to_reverse() {
            -diff
        } else {
            diff
        }
    }

    fn rebuild_ranks(&mut self) -> bool {
        let new_rank = std::cmp::max(self.left.rank(), self.right.rank()) + 1;
        let changed = self.rank() != new_rank;
        self.alg_data = new_rank;
        changed
    }
}

impl<D: Data> AVLTree<D> {
    /// Creates an empty [`AVLTree`].
    pub fn new() -> Self {
        AVLTree {
            tree: BasicTree::Empty,
        }
    }

    /// Asserts that the ranks at the current node are correct.
    /// Otherwise, panics.
    pub fn assert_ranks_locally(&self) {
        if let Some(node) = self.tree.node() {
            Self::assert_ranks_locally_internal(node);
        }
    }

    fn assert_ranks_locally_internal(node: &BasicNode<D, T>) {
        assert!(node.rank() == node.left.rank() + 1 || node.rank() == node.right.rank() + 1);
        assert!(node.left.rank() == node.rank() - 1 || node.left.rank() == node.rank() - 2);
        assert!(node.right.rank() == node.rank() - 1 || node.right.rank() == node.rank() - 2);
    }

    /// Asserts that the tree's ranks are correct.
    /// Otherwise, panics.
    pub fn assert_ranks(&self) {
        self.tree
            .assert_correctness_with(Self::assert_ranks_locally_internal);
    }
}

impl<D: Data> Rankable for AVLTree<D> {
    fn rank(&self) -> T {
        self.tree.rank()
    }

    /// Returns `right.rank() - left.rank()`
    fn rank_diff(&self) -> TD {
        self.tree.rank_diff()
    }

    fn rebuild_ranks(&mut self) -> bool {
        self.tree.rebuild_ranks()
    }
}

impl<D: Data> Default for AVLTree<D> {
    fn default() -> Self {
        AVLTree::new()
    }
}

impl<D: Data> SomeTree<D> for AVLTree<D> {
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
        if !action.to_reverse() {
            methods::act_segment(self, action, locator)
        } else {
            // split out the middle
            let mut mid: AVLTree<D> = self
                .slice(locators::LeftEdgeOf(locator.clone()))
                .split_right()
                .unwrap();

            let mut walker2 = AVLWalker {
                walker: BasicWalker::new_with_context(
                    &mut mid.tree,
                    self.subtree_summary(),
                    Default::default(),
                ),
            };
            walker2.search_subtree(locators::RightEdgeOf(locator));
            let right = walker2.split_right().unwrap();
            drop(walker2);

            // apply action
            mid.act_subtree(action);

            // glue back together
            mid.concatenate_right(right);
            self.concatenate_right(mid);
        }
    }

    type TreeData = u8;
    fn iter_locator<'a, L: locators::Locator<D>>(
        &'a mut self,
        locator: L,
    ) -> basic_tree::iterators::IterLocator<'a, D, L, u8> {
        iterators::IterLocator::new(&mut self.tree, locator)
    }

    fn assert_correctness(&self)
    where
        D::Summary: Eq,
    {
        self.tree.assert_correctness_with(|node| {
            node.assert_correctness_locally();
            Self::assert_ranks_locally_internal(node);
        });
    }
}

impl<'a, D: Data> SomeTreeRef<D> for &'a mut AVLTree<D> {
    type Walker = AVLWalker<'a, D>;

    fn walker(self) -> Self::Walker {
        AVLWalker {
            walker: self.tree.walker(),
        }
    }
}

impl<'a, D: Data> ModifiableTreeRef<D> for &'a mut AVLTree<D> {
    type ModifiableWalker = AVLWalker<'a, D>;
}

impl<'a, D: Data> SplittableTreeRef<D> for &'a mut AVLTree<D> {
    type T = AVLTree<D>;

    type SplittableWalker = AVLWalker<'a, D>;
}

derive_SomeEntry! {tree, 
    impl<D: Data> SomeEntry<D> for AVLTree<D> {
        fn assert_correctness_locally(&self)
        where
            D::Summary: Eq,
        {
            if let Some(node) = self.tree.node() {
                Self::assert_ranks_locally_internal(node);
                node.assert_correctness_locally();
            }
        }
    }
}

impl<D: Data> std::iter::FromIterator<D::Value> for AVLTree<D> {
    /// This takes `O(n)` worst-case time.
    fn from_iter<T: IntoIterator<Item = D::Value>>(iter: T) -> Self {
        // TODO: check if inserting is O(1) amortized. if it is, we can do this by
        // just calling insert.
        // if not, than this is `O(n log n)` worst-case time.

        let mut tree: AVLTree<D> = Default::default();
        let mut walker = tree.walker();
        for val in iter.into_iter() {
            // note: this relies on the assumption, that after we insert a node, the new position of the locator
            // will be an ancestor of the location where the value was inserted.
            // TODO: check.
            while walker.go_right().is_ok() {}
            walker.insert(val);
        }
        drop(walker);
        tree
    }
}

impl<D: Data> IntoIterator for AVLTree<D> {
    type Item = D::Value;
    type IntoIter = iterators::IntoIter<D, std::ops::RangeFull, T>;

    fn into_iter(self) -> Self::IntoIter {
        iterators::IntoIter::new(self.tree, ..)
    }
}

/// A walker struct for [`AVLTree`].
pub struct AVLWalker<'a, D: Data> {
    walker: BasicWalker<'a, D, T>,
}

impl<'a, D: Data> std::ops::Drop for AVLWalker<'a, D> {
    fn drop(&mut self) {
        self.go_to_root()
    }
}

derive_SomeWalker!{walker,
    impl<'a, D: Data> SomeWalker<D> for AVLWalker<'a, D> {
        fn go_up(&mut self) -> Result<Side, ()> {
            let res = self.walker.go_up()?;
            let changed = self.inner_mut().rebuild_ranks();
            assert!(!changed); // it shouldn't have changed without being rebalanced already
            Ok(res)
        }
    }
}

derive_SomeEntry!{walker,
    impl<'a, D: Data> SomeEntry<D> for AVLWalker<'a, D> {
        fn assert_correctness_locally(&self)
        where
            D::Summary: Eq,
        {
            self.walker.assert_correctness_locally();
            if let Some(node) = self.walker.node() {
                AVLTree::assert_ranks_locally_internal(node);
            }
        }
    }
}

impl<'a, D: Data> Rankable for AVLWalker<'a, D> {
    /// Returns the priority of the current node. Lower numbers means
    /// The node is closer to the root.
    fn rank(&self) -> T {
        match self.walker.node() {
            None => 0,
            Some(node) => *node.alg_data(),
        }
    }

    /// Returns `right.rank() - left.rank()`
    fn rank_diff(&self) -> TD {
        self.walker.inner().rank_diff()
    }

    fn rebuild_ranks(&mut self) -> bool {
        self.inner_mut().rebuild_ranks()
    }
}

impl<'a, D: Data> AVLWalker<'a, D> {
    fn inner(&self) -> &BasicTree<D, T> {
        self.walker.inner()
    }

    fn inner_mut(&mut self) -> &mut BasicTree<D, T> {
        self.walker.inner_mut()
    }

    fn rot_left(&mut self) -> Option<()> {
        let rebuilder = |node: &mut BasicNode<D, T>| {
            node.rebuild_ranks();
        };
        self.walker.rot_left_with_custom_rebuilder(rebuilder)
    }

    fn rot_right(&mut self) -> Option<()> {
        let rebuilder = |node: &mut BasicNode<D, T>| {
            node.rebuild_ranks();
        };
        self.walker.rot_right_with_custom_rebuilder(rebuilder)
    }

    fn rot_up(&mut self) -> Result<Side, ()> {
        let rebuilder = |node: &mut BasicNode<D, T>| {
            node.rebuild_ranks();
        };
        self.walker.rot_up_with_custom_rebuilder(rebuilder)
    }

    // For completeness this function is still here. It might be used in future versions.
    #[allow(dead_code)]
    fn rot_side(&mut self, side: Side) -> Option<()> {
        let rebuilder = |node: &mut BasicNode<D, T>| {
            node.rebuild_ranks();
        };
        self.walker.rot_side_with_custom_rebuilder(side, rebuilder)
    }

    /// This function gets called when a node is deleted or inserted,
    /// at the current position.
    fn rebalance(&mut self) {
        if self.is_empty() {
            let res = self.walker.go_up(); // ranks may be incorrect, so go up with the inner walker
            if res.is_err() {
                return;
            }
        }

        self.rebuild_ranks();

        loop {
            let node = self.inner().node().unwrap();
            match self.rank_diff() {
                -2 => {
                    // -2, left is deeper
                    if node.left.rank_diff() <= 0 {
                        // left left case
                        self.rot_right().unwrap();
                    } else {
                        // left.rank() = 1, left right case
                        self.go_left().unwrap();
                        self.rot_left().unwrap(); // TODO
                        let res = self.rot_up();
                        assert!(res == Ok(Side::Left));
                    }
                }

                -1..=1 => {} // do nothing, the current node is now balanced.

                2 => {
                    // 2, left is shallower
                    if node.right.rank_diff() >= 0 {
                        // right right case
                        self.rot_left().unwrap();
                    } else {
                        // right.rank() = -1, right left case
                        self.go_right().unwrap();
                        self.rot_right().unwrap();
                        let res = self.rot_up();
                        assert!(res == Ok(Side::Right));
                    }
                }

                rd => panic!("illegal rank difference: {}", rd),
            }

            // current node has been balanced. now go up a node,
            // and check if we need to continue rebalancing.
            let res = self.walker.go_up(); // ranks may be incorrect, so go up with the inner walker
            let changed = self.inner_mut().rebuild_ranks();
            let rd = self.inner().rank_diff();
            if !changed && -1 <= rd && rd <= 1 {
                // tree is now balanced correctly
                break;
            }
            if res.is_err() {
                // reached root
                break;
            }
        }
    }

    // TODO: specify where the walker will be.
    /// Deletes a node and returns it with the box.
    fn delete_boxed(&mut self) -> Option<Box<BasicNode<D, T>>> {
        // the delete implementation is copied from `BasicTree`,
        // in order for rebalancing to be done properly.
        let mut node = self.walker.take_subtree().into_node_boxed()?;
        if node.right.is_empty() {
            self.walker.put_subtree(node.left).unwrap();
            node.left = BasicTree::Empty;
            self.rebalance();
        } else {
            // find the next node and move it to the current position
            let mut walker = node.right.walker();
            while walker.go_left().is_ok() {}
            let res = walker.go_up();
            assert_eq!(res, Ok(Side::Left));

            let mut boxed_replacement_node = walker.take_subtree().into_node_boxed().unwrap();
            assert!(boxed_replacement_node.left.is_empty());
            walker.put_subtree(boxed_replacement_node.right).unwrap();
            AVLWalker { walker }.rebalance(); // rebalance here

            boxed_replacement_node.left = node.left;
            node.left = BasicTree::Empty;
            boxed_replacement_node.right = node.right;
            node.right = BasicTree::Empty;
            boxed_replacement_node.rebuild();
            self.walker
                .put_subtree(BasicTree::Root(boxed_replacement_node))
                .unwrap();
            self.rebalance(); // rebalance here
        }
        Some(node)
    }
}

impl<'a, D: Data> ModifiableWalker<D> for AVLWalker<'a, D> {
    /// Inserts the value into the tree at the current empty position.
    /// If the current position is not empty, return [`None`].
    /// When the function returns, the walker will be at a position which is an ancestor of the
    /// newly inserted node.
    fn insert(&mut self, val: D::Value) -> Option<()> {
        self.walker
            .insert_with_alg_data(val, 1 /* rank of a node with no sons */)?;
        self.rebalance();
        Some(())
    }

    // TODO: specify where the walker will be.
    fn delete(&mut self) -> Option<D::Value> {
        Some(self.delete_boxed()?.node_value)
    }
}

impl<'a, D: Data> SplittableWalker<D> for AVLWalker<'a, D> {
    type T = AVLTree<D>;

    /// Will only do anything if the current position is empty.
    /// If it is empty, it will split the tree: the elements
    /// to the left will remain, and the elements to the right
    /// will be put in the new output tree.
    /// The walker will be at the root after this operation, if it succeeds.
    ///
    ///```
    /// use grove::{SomeTree, avl::AVLTree};
    /// use grove::example_data::StdNum;
    ///
    /// let mut tree: AVLTree<StdNum> = (17..88).collect();
    /// let mut tree2 = tree.slice(7..7).split_right().unwrap();
    ///
    /// assert_eq!(tree.iter().cloned().collect::<Vec<_>>(), (17..24).collect::<Vec<_>>());
    /// assert_eq!(tree2.iter().cloned().collect::<Vec<_>>(), (24..88).collect::<Vec<_>>());
    /// # tree.assert_correctness();
    ///```
    fn split_right(&mut self) -> Option<Self::T> {
        if !self.is_empty() {
            return None;
        }
        let mut left = AVLTree::new();
        let mut right = AVLTree::new();

        // ranks may be incorrect, so go up with the inner walker
        while let Ok(side) = self.walker.go_up() {
            let mut node = self.walker.take_subtree().into_node_boxed().unwrap();
            match side {
                Side::Left => {
                    assert!(node.left.is_empty());
                    let auxiliary_right = AVLTree { tree: node.right };
                    node.right = BasicTree::Empty;
                    right.concatenate_boxed_middle_right(node, auxiliary_right);
                }
                Side::Right => {
                    assert!(node.right.is_empty());
                    let auxiliary_left = AVLTree { tree: node.left };
                    node.left = BasicTree::Empty;
                    left.concatenate_boxed_middle_left(auxiliary_left, node);
                }
            }
        }

        // the `self` tree is empty by this point.
        self.walker.put_subtree(left.tree).unwrap();
        Some(right)
    }

    /// Will only do anything if the current position is empty.
    /// If it is empty, it will split the tree: the elements
    /// to the left will remain, and the elements to the right
    /// will be put in the new output tree.
    /// The walker will be at the root after this operation, if it succeeds.
    ///
    ///```
    /// use grove::{SomeTree, avl::AVLTree};
    /// use grove::example_data::StdNum;
    ///
    /// let mut tree: AVLTree<StdNum> = (17..88).collect();
    /// let mut tree2 = tree.slice(7..7).split_left().unwrap();
    ///
    /// assert_eq!(tree2.iter().cloned().collect::<Vec<_>>(), (17..24).collect::<Vec<_>>());
    /// assert_eq!(tree.iter().cloned().collect::<Vec<_>>(), (24..88).collect::<Vec<_>>());
    /// # tree.assert_correctness();
    ///```
    fn split_left(&mut self) -> Option<Self::T> {
        let mut right = self.split_right()?;
        std::mem::swap(&mut right.tree, self.inner_mut());
        Some(right)
    }
}

impl<D: Data> AVLTree<D> {
    /// Concatenates the trees together, in place, with a given value for the middle.
    /// Complexity: `O(log n)`. More precisely, `O(dr)` where `dr` is the difference of ranks between the two trees.
    ///```
    /// use grove::{SomeTree, avl::AVLTree};
    /// use grove::example_data::StdNum;
    ///
    /// let mut tree: AVLTree<StdNum> = (17..=89).collect();
    /// let tree2: AVLTree<StdNum> = (13..=25).collect();
    /// tree.concatenate_middle_right(5, tree2);
    ///
    /// assert_eq!(tree.iter().cloned().collect::<Vec<_>>(), (17..=89).chain(5..=5).chain(13..=25).collect::<Vec<_>>());
    /// # tree.assert_correctness();
    ///```
    pub fn concatenate_middle_right(&mut self, mid: D::Value, right: AVLTree<D>) {
        let node = BasicNode::new_alg(mid, 0 /* dummy value */);
        self.concatenate_boxed_middle_right(Box::new(node), right);
    }

    fn concatenate_boxed_middle_right(
        &mut self,
        mut mid: Box<BasicNode<D, T>>,
        mut right: AVLTree<D>,
    ) {
        if self.rank() < right.rank() {
            std::mem::swap(self, &mut right);
            self.concatenate_boxed_middle_left(right, mid);
            return;
        }
        let mut walker = self.walker();
        while walker.rank() > right.rank() {
            walker.go_right().unwrap();
        }
        mid.alg_data = 0;
        mid.left = walker.walker.take_subtree();
        mid.right = right.tree;
        mid.rebuild();
        walker.walker.put_subtree(BasicTree::Root(mid)).unwrap();
        walker.rebalance();
    }

    /// Concatenates the trees together, in place, with a given value for the middle.
    /// Complexity: `O(log n)`. More precisely, `O(dr)` where `dr` is the difference of ranks between the two trees.
    ///```
    /// use grove::{SomeTree, avl::AVLTree};
    /// use grove::example_data::StdNum;
    ///
    /// let tree1: AVLTree<StdNum> = (17..=89).collect();
    /// let mut tree2: AVLTree<StdNum> = (13..=25).collect();
    /// tree2.concatenate_middle_left(tree1, 5);
    ///
    /// assert_eq!(tree2.iter().cloned().collect::<Vec<_>>(), (17..=89).chain(5..=5).chain(13..=25).collect::<Vec<_>>());
    /// # tree2.assert_correctness();
    ///```
    pub fn concatenate_middle_left(&mut self, left: AVLTree<D>, mid: D::Value) {
        let node = BasicNode::new_alg(mid, 0 /* dummy value */);
        self.concatenate_boxed_middle_left(left, Box::new(node));
    }

    fn concatenate_boxed_middle_left(
        &mut self,
        mut left: AVLTree<D>,
        mut mid: Box<BasicNode<D, T>>,
    ) {
        if self.rank() < left.rank() {
            std::mem::swap(self, &mut left);
            self.concatenate_boxed_middle_right(mid, left);
            return;
        }
        let mut walker = self.walker();
        while walker.rank() > left.rank() {
            walker.go_left().unwrap();
        }
        mid.alg_data = 0;
        mid.right = walker.walker.take_subtree();
        mid.left = left.tree;
        mid.rebuild();
        walker.walker.put_subtree(BasicTree::Root(mid)).unwrap();
        walker.rebalance();
    }
}

impl<D: Data> ConcatenableTree<D> for AVLTree<D> {
    /// Concatenates the trees together, in place.
    /// Complexity: `O(log n)`.
    ///```
    /// use grove::{SomeTree, ConcatenableTree, avl::AVLTree};
    /// use grove::example_data::StdNum;
    ///
    /// let mut tree: AVLTree<StdNum> = (17..=89).collect();
    /// let tree2: AVLTree<StdNum> = (13..=25).collect();
    /// tree.concatenate_right(tree2);
    ///
    /// assert_eq!(tree.iter().cloned().collect::<Vec<_>>(), (17..=89).chain(13..=25).collect::<Vec<_>>());
    /// # tree.assert_correctness();
    ///```
    fn concatenate_right(&mut self, mut right: Self) {
        if !right.is_empty() {
            let mut walker = right.search(locators::LeftEdgeOf(..));
            walker.go_up().unwrap();
            let mid = walker.delete_boxed().unwrap(); // TODO: deallocated node only to reallocate it later. fix.
            drop(walker);
            self.concatenate_boxed_middle_right(mid, right);
        }
    }
}

/// Concatenates the trees together, in place, with a given value for the middle.
/// Complexity: `O(log n)`. More precisely, `O(dr)` where `dr` is the difference of ranks between the two trees.
///```
    /// use grove::{SomeTree, avl::AVLTree, avl::concatenate_with_middle};
    /// use grove::example_data::StdNum;
///
/// let tree1: AVLTree<StdNum> = (17..=89).collect();
/// let tree2: AVLTree<StdNum> = (13..=25).collect();
/// let mut tree3 = concatenate_with_middle(tree1, 5, tree2);
///
/// assert_eq!(tree3.iter().cloned().collect::<Vec<_>>(), (17..=89).chain(5..=5).chain(13..=25).collect::<Vec<_>>());
/// # tree3.assert_correctness();
///```
pub fn concatenate_with_middle<D: Data>(
    mut left: AVLTree<D>,
    mid: D::Value,
    right: AVLTree<D>,
) -> AVLTree<D> {
    left.concatenate_middle_right(mid, right);
    left
}
