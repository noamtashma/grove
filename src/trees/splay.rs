//! An implementation of a splay tree
//!
//! It is a balanced tree algorithm that supports reversals, splitting and concatenation,
//! and has remarkable properties.
//!
//! Its operations take `O(log n)` amortized time
//! (deterministically). Therefore, individual operations may take up to linear time,
//! but it never takes more than `O(log n)` time per operation when the operations are
//! counted together.
//!
//! The tree implements the [`splay`] operation, that should be used after searching for a node.
//!
//! The Splay tree's complexity guarantees follow from splaying nodes
//! after accessing them. This is so that when you go deep down the tree,
//! when you come back up you make the deep part less deep.
//!
//! Therefore, going up the tree without splaying can undermine the splay tree's complexity
//! guarantees, and operations that should only take `O(log n)` amortized time
//! can take linear time.
//!
//! It is recommended that if you want to reuse a [`SplayWalker`], use the
//! [`splay`] function.
//!
//! When a [`SplayWalker`] is dropped, the walker automatically splays up the tree,
//! ensuring that nodes that need to be rebuilt are rebuilt, but also that
//! the splaytree's complexity properties remain.

use super::basic_tree::*;
use super::*;
use crate::locators;

#[derive(destructure)]
/// A Splay tree.
///
/// It is a balanced tree algorithm that supports reversals, splitting and concatenation,
/// and has remarkable properties.
///
/// Its operations take `O(log n)` amortized time
/// (deterministically). Therefore, individual operations may take up to linear time,
/// but it never takes more than `O(log n)` time per operation when the operations are
/// counted together.
///
/// The tree implements the [`splay`] operation, that should be used after searching for a node.
///
/// The Splay tree's complexity guarantees follow from splaying nodes
/// after accessing them. This is so that when you go deep down the tree,
/// when you come back up you make the deep part less deep.
///
/// Therefore, going up the tree without splaying can undermine the splay tree's complexity
/// guarantees, and operations that should only take `O(log n)` amortized time
/// can take linear time.
///
/// It is recommended that if you want to reuse a [`SplayWalker`], use the
/// [`splay`] function.
///
/// When a [`SplayWalker`] is dropped, the walker automatically splays up the tree,
/// ensuring that nodes that need to be rebuilt are rebuilt, but also that
/// the splaytree's complexity properties remain.
pub struct SplayTree<D: Data> {
    tree: BasicTree<D>,
}

impl<D: Data> SplayTree<D> {
    /// Note: using this directly may cause the tree to lose its properties as a splay tree
    pub fn basic_walker(&mut self) -> BasicWalker<D> {
        BasicWalker::new(&mut self.tree)
    }

    /// Creates a new empty [`SplayTree`].
    pub fn new() -> SplayTree<D> {
        SplayTree {
            tree: BasicTree::Empty,
        }
    }

    /// Checks that invariants remain correct. i.e., that every node's summary
    /// is the sum of the summaries of its children.
    /// If it is not, panics.
    pub fn assert_correctness(&self)
    where
        D::Summary: Eq,
    {
        self.tree.assert_correctness()
    }

    /// Gets the tree into a state in which the locator's segment
    /// is a single subtree, and returns a walker at that subtree.
    pub fn isolate_segment<'a, L>(&'a mut self, locator: L) -> SplayWalker<'a, D>
    where
        L: crate::Locator<D>,
    {
        if self.is_empty() {
            return self.walker();
        }

        let left_edge = locators::LeftEdgeOf(locator.clone());
        // reborrows the tree for a shorter time
        let mut walker = self.slice(left_edge).search();
        let b1 = walker.previous_filled().is_ok();
        walker.splay(); // must drop here so that the next call to search can happen

        // if we previously splayed a node, work only below it, in order to not move it
        // when splaying
        if b1 {
            walker.go_right().unwrap();
        }

        let mut walker2 = SplayWalker {
            walker: walker.walker.detached_walker(),
        };

        let right_edge = locators::RightEdgeOf(locator);
        walker2.go_to_root();
        walker2.search_subtree(right_edge);
        let b2 = walker2.next_filled().is_ok();
        walker2.splay();
        drop(walker2);

        if b2 {
            walker.go_left().unwrap();
        }

        walker
    }

    /// Converts the tree into its internal representation as a [`BasicTree`].
    pub fn into_inner(self) -> BasicTree<D> {
        self.destructure().0
    }
}

impl<D: Data> std::default::Default for SplayTree<D> {
    fn default() -> Self {
        SplayTree::new()
    }
}

/// Deallocating a large splay tree can cause a stack overflow, since the tree might be unbalanced.
/// Therefore we have an iterative deallocator.
impl<D: Data> Drop for SplayTree<D> {
    fn drop(&mut self) {
        basic_tree::deallocate_iteratively(&mut self.tree);
    }
}

/// A walker for a [`SplayTree`].
#[derive(destructure)]
pub struct SplayWalker<'a, D: Data> {
    walker: BasicWalker<'a, D>,
}

impl<'a, D: Data> SplayWalker<'a, D> {
    /// Creates a new walker for the given tree.
    pub fn new(walker: BasicWalker<'a, D>) -> Self {
        SplayWalker { walker }
    }

    /// Returns the internal [`BasicWalker`].
    pub fn inner(&self) -> &BasicTree<D> {
        self.walker.inner()
    }

    // using this function can really mess up the structure
    // use wisely
    // this function shouldn't really be public
    // TODO: should this function exist?
    pub(super) fn inner_mut(&mut self) -> &mut BasicTree<D> {
        self.walker.inner_mut()
    }

    /// Converts into the internal [`BasicWalker`].
    pub fn into_inner(self) -> BasicWalker<'a, D> {
        // this is a workaround for the problem that,
        // we can't move out of a type implementing Drop

        let (walker,) = self.destructure();
        walker
    }

    /// If at the root, do nothing.
    /// otherwise, do a single splay step upwards.
    /// If empty, go up once and return.
    ///
    /// Amortized complexity of splay steps:
    /// The amortized cost of any splay step, except the zig step near the root, is at most
    /// `log(new_node.size) - log(old_node.size) - 1`
    /// The -1 covers the complexity of going down the tree in the first place.
    pub fn splay_step(&mut self) {
        // if the walker points to an empty position,
        // we can't splay it, just go upwards once.
        if self.walker.is_empty() {
            let _ = self.walker.go_up();
            return;
        }

        let b1 = match self.walker.go_up() {
            Err(()) => return, // already the root
            Ok(b1) => b1,
        };

        let b2 = match self.walker.is_left_son() {
            None => {
                self.walker.rot_side(b1.flip()).unwrap();
                return;
            } // became the root - zig step
            Some(b2) => b2,
        };

        if b1 == b2 {
            // zig-zig case
            self.walker.rot_up().unwrap();
            self.walker.rot_side(b1.flip()).unwrap();
        } else {
            // zig-zag case
            self.walker.rot_side(b1.flip()).unwrap();
            self.walker.rot_up().unwrap();
        }
    }

    /// Same as [`SplayWalker::splay_step`], but splays up to the specified depth.
    pub fn splay_step_depth(&mut self, depth: usize) {
        if self.depth() <= depth {
            return;
        }

        // if the walker points to an empty position,
        // we can't splay it, just go upwards once.
        if self.walker.is_empty() {
            if let Err(()) = self.walker.go_up() {
                // if already the root, exit. otherwise, go up
                panic!(); // shouldn't happen, because if we are at the root, the previous condition would have caught it.
            };
            return;
        }

        let b1 = match self.walker.go_up() {
            Ok(b1) => b1,
            Err(()) => panic!(), // shouldn't happen, the previous condition would have caught this
        };

        if self.depth() <= depth {
            // zig case
            self.walker.rot_side(b1.flip()).unwrap();
        } else {
            let b2 = match self.walker.is_left_son() {
                None => panic!(), // we couldn't have gone into this branch
                Some(b2) => b2,
            };

            if b1 == b2 {
                // zig-zig case
                self.walker.rot_up().unwrap();
                self.walker.rot_side(b1.flip()).unwrap();
            } else {
                // zig-zag case
                self.walker.rot_side(b1.flip()).unwrap();
                self.walker.rot_up().unwrap();
            }
        }
    }

    /// Splay the current node to the top of the tree.
    ///
    /// If the walker is on an empty spot, it will splay some nearby node
    /// to the top of the tree instead. (current behavior is the father of the empty spot).
    ///
    /// Going down the tree, and then splaying,
    /// has an amortized cost of `O(log n)`.
    pub fn splay(&mut self) {
        self.splay_to_depth(0);
    }

    /// Splays a node into a given depth. Doesn't make any changes to any nodes closer to the root.
    /// If the node is at a shallower depth already, the function panics.
    /// See the [`splay`] function.
    pub fn splay_to_depth(&mut self, depth: usize) {
        assert!(self.depth() >= depth);
        while self.walker.depth() != depth {
            self.splay_step_depth(depth);
        }
    }
}

impl<'a, D: Data> Drop for SplayWalker<'a, D> {
    fn drop(&mut self) {
        self.splay();
    }
}

impl<D: Data> SomeTree<D> for SplayTree<D> {
    /// Note: calling this is inefficient
    /// and panicks if debug assertions are on.
    ///
    /// This is because splay trees rely on changing the tree's structure to ensure
    /// its complexity properties.
    /// Using this might take `O(n)` time per operation in the worst case.
    fn segment_summary_imm<L>(&self, locator: L) -> D::Summary
    where
        L: locators::Locator<D>,
        D::Value: Clone,
    {
        if cfg!(debug_assertions) {
            panic!(".segment_summary_imm() method is inefficient for splay trees")
        } else {
            self.tree.segment_summary_imm(locator)
        }
    }

    /// This is the same as `self.segment_summary_unclonable(locator)`
    /// because splay trees rely on changing the tree's structure to ensure
    /// its complexity properties.
    fn segment_summary<L>(&mut self, locator: L) -> D::Summary
    where
        L: locators::Locator<D>,
    {
        self.segment_summary_unclonable(locator)
    }

    fn segment_summary_unclonable<L>(&mut self, locator: L) -> D::Summary
    where
        L: locators::Locator<D>,
    {
        let walker = self.isolate_segment(locator);
        walker.subtree_summary()
    }

    fn act_segment<L>(&mut self, action: D::Action, locator: L)
    where
        L: crate::Locator<D>,
    {
        let mut walker = self.isolate_segment(locator);
        walker.act_subtree(action);
    }

    type TreeData = ();
    fn iter_locator<'a, L: locators::Locator<D>>(
        &'a mut self,
        locator: L,
    ) -> basic_tree::iterators::IterLocator<'a, D, L> {
        self.isolate_segment(locator.clone());
        iterators::IterLocator::new(&mut self.tree, locator)
    }

    fn assert_correctness(&self)
    where
        D::Summary: Eq,
    {
        self.tree.assert_correctness();
    }
}

derive_SomeEntry! {tree, (),
    impl<D: Data> SomeEntry<D> for SplayTree<D> {
        fn assert_correctness_locally(&self)
        where
            D::Summary: Eq,
        {
            self.tree.assert_correctness_locally();
        }
    }
}

impl<'a, D: Data> SomeTreeRef<D> for &'a mut SplayTree<D> {
    type Walker = SplayWalker<'a, D>;
    fn walker(self: &'a mut SplayTree<D>) -> SplayWalker<'a, D> {
        SplayWalker {
            walker: self.basic_walker(),
        }
    }
}

impl<'a, D: Data> ModifiableTreeRef<D> for &'a mut SplayTree<D> {
    type ModifiableWalker = Self::Walker;
}

impl<D: Data> std::iter::FromIterator<D::Value> for SplayTree<D> {
    fn from_iter<T: IntoIterator<Item = D::Value>>(iter: T) -> Self {
        SplayTree {
            tree: iter.into_iter().collect(),
        }
    }
}

impl<D: Data> IntoIterator for SplayTree<D> {
    type Item = D::Value;
    type IntoIter = <BasicTree<D> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}

derive_SomeWalker! {walker,
    impl<'a, D: Data> SomeWalker<D> for SplayWalker<'a, D> {
        /// If successful, returns whether or not the previous current value was the left son.
        /// If already at the root of the tree, returns `Err(())`.
        /// You shouldn't use this method too much, or you might lose the
        /// SplayTree's complexity properties - see documentation aboud splay tree.
        fn go_up(&mut self) -> Result<Side, ()> {
            self.walker.go_up()
        }

        // overrides the default implementations for these methods:

        /// Finds the previous filled node.
        /// If there isn't any, moves to root and return Err(()).
        ///
        /// Restructures the tree in order to satisfy the splay tree's complexity properties.
        /// Complexity: amortized `O(log n)` time.
        fn previous_filled(&mut self) -> Result<(), ()> {
            match self.walker.node() {
                None => {}
                Some(node) => {
                    if !node.left.is_empty() {
                        // the previous node is in this node's left subtree case
                        self.go_left().unwrap();
                        while self.go_right().is_ok() {}
                        let r = self.go_up();
                        assert_eq!(r, Ok(Side::Right));
                        return Ok(());
                    }
                }
            }

            // the next filled node is this node's first left ancestor
            let count = match self.walker.steps_until_sided_ancestor(Side::Right) {
                None => {
                    self.splay();
                    return Err(());
                }
                Some(count) => count,
            };

            let depth = self.depth();
            // splay to just below the previous filled value
            self.splay_to_depth(depth - count + 1);
            let r = self.go_up();
            assert_eq!(r, Ok(Side::Right));
            Ok(())
        }

        /// Finds the next filled node.
        /// If there isn't any, moves to root and return Err(()).
        ///
        /// Restructures the tree in order to satisfy the splay tree's complexity properties.
        /// Complexity: amortized `O(log n)` time.
        fn next_filled(&mut self) -> Result<(), ()> {
            match self.walker.node() {
                None => {}
                Some(node) => {
                    if !node.right.is_empty() {
                        // the previous node is in this node's right subtree case
                        self.go_right().unwrap();
                        while self.go_left().is_ok() {}
                        let r = self.go_up();
                        assert_eq!(r, Ok(Side::Left));
                        return Ok(());
                    }
                }
            }
            // return methods::next_filled(self);
            // the next filled node is this node's first right ancestor
            let count = match self.walker.steps_until_sided_ancestor(Side::Left) {
                None => {
                    self.splay();
                    return Err(());
                }
                Some(count) => count,
            };

            let depth = self.depth();
            // splay to just below the previous filled value
            self.splay_to_depth(depth - count + 1);
            let r = self.go_up();
            assert_eq!(r, Ok(Side::Left));
            Ok(())
        }
    }
}

derive_SomeEntry! {walker, (),
    impl<'a, D: Data> SomeEntry<D> for SplayWalker<'a, D> {
        fn assert_correctness_locally(&self)
        where
            D::Summary: Eq,
        {
            self.walker.assert_correctness_locally();
        }
    }
}

impl<'a, D: Data> ModifiableWalker<D> for SplayWalker<'a, D> {
    /// Inserts the value into the tree at the current empty position.
    /// If the current position is not empty, return [`None`].
    /// When the function returns, the walker will be at the position the node
    /// was inserted.
    fn insert(&mut self, value: D::Value) -> Option<()> {
        self.walker.insert(value)
    }

    /// Removes the current value from the tree, and returns it.
    /// If currently at an empty position, returns [`None`].
    /// After deletion, the walker may move to a son of the current node or to an adjacent empty position.
    fn delete(&mut self) -> Option<D::Value> {
        // the delete implementation is copied from `BasicTree`,
        // in order that splaying could be done on the second part of the path,
        // to preserve the splay tree's complexity properties.
        let mut node = self.walker.take_subtree().into_node()?;
        if node.right.is_empty() {
            self.walker.put_subtree(node.left).unwrap();
        } else {
            // find the next node and move it to the current position
            let mut walker = node.right.walker();
            while walker.go_left().is_ok() {}
            let res = walker.go_up();
            assert_eq!(res, Ok(Side::Left));

            let mut boxed_replacement_node = walker.take_subtree().into_node_boxed().unwrap();
            assert!(boxed_replacement_node.left.is_empty());
            walker.put_subtree(boxed_replacement_node.right).unwrap();
            drop(SplayWalker { walker }); // splay to preserve the tree's complexity

            boxed_replacement_node.left = node.left;
            boxed_replacement_node.right = node.right;
            boxed_replacement_node.rebuild();
            self.walker
                .put_subtree(BasicTree::Root(boxed_replacement_node))
                .unwrap();
        }
        Some(node.node_value)
    }
}

impl<D: Data> ConcatenableTree<D> for SplayTree<D> {
    /// Concatenates the other tree into this tree.
    ///```
    /// use grove::{SomeTree, ConcatenableTree, splay::SplayTree};
    /// use grove::example_data::StdNum;
    ///
    /// let tree1: SplayTree<StdNum> = (17..=89).collect();
    /// let tree2: SplayTree<StdNum> = (13..=25).collect();
    /// let mut tree3 = ConcatenableTree::concatenate(tree1, tree2);
    ///
    /// assert_eq!(tree3.iter().cloned().collect::<Vec<_>>(), (17..=89).chain(13..=25).collect::<Vec<_>>());
    /// # tree3.assert_correctness();
    ///```
    fn concatenate_right(&mut self, other: Self) {
        let mut walker = self.walker();
        while walker.go_right().is_ok() {}
        match walker.go_up() {
            Err(()) => {
                // the tree is empty; just substitute the other tree.
                drop(walker);
                *self = other;
                return;
            }
            Ok(Side::Right) => (),
            Ok(Side::Left) => unreachable!(),
        };
        walker.splay();
        let node = walker.inner_mut().node_mut().unwrap();
        assert!(node.right.is_empty());
        node.right = other.into_inner();
        node.rebuild();
    }
}

impl<'a, D: Data> SplittableTreeRef<D> for &'a mut SplayTree<D> {
    type T = SplayTree<D>;

    type SplittableWalker = SplayWalker<'a, D>;
}

impl<'a, D: Data> SplittableWalker<D> for SplayWalker<'a, D> {
    type T = SplayTree<D>;

    /// Will only do anything if the current position is empty.
    /// If it is empty, it will split the tree: the elements
    /// to the left will remain, and the elements to the right
    /// will be put in the new output tree.
    /// The walker will be at the root after this operation, if it succeeds.
    ///
    ///```
    /// use grove::{SomeTree, SplittableTreeRef, splay::SplayTree};
    /// use grove::example_data::StdNum;
    ///
    /// let mut tree: SplayTree<StdNum> = (17..88).collect();
    /// let mut tree2 = tree.slice(7..7).split_right().unwrap();
    ///
    /// assert_eq!(tree.iter().cloned().collect::<Vec<_>>(), (17..24).collect::<Vec<_>>());
    /// assert_eq!(tree2.iter().cloned().collect::<Vec<_>>(), (24..88).collect::<Vec<_>>());
    /// # tree.assert_correctness();
    ///```
    fn split_right(&mut self) -> Option<SplayTree<D>> {
        if !self.is_empty() {
            return None;
        }

        // to know which side we should cut
        let side = match self.go_up() {
            Err(()) => return Some(SplayTree::new()), // this is the empty tree
            Ok(b) => b,
        };
        self.splay();
        let node = match self.inner_mut().node_mut() {
            Some(node) => node,
            _ => panic!(),
        };
        match side {
            Side::Left => {
                let mut tree = std::mem::replace(&mut node.left, BasicTree::Empty);
                node.rebuild();
                std::mem::swap(self.inner_mut(), &mut tree);
                Some(SplayTree { tree })
            }
            Side::Right => {
                let tree = std::mem::replace(&mut node.right, BasicTree::Empty);
                node.rebuild();
                Some(SplayTree { tree })
            }
        }
    }

    fn split_left(&mut self) -> Option<Self::T> {
        let mut right = self.split_right()?;
        std::mem::swap(self.inner_mut(), &mut right.tree);
        Some(right)
    }
}
