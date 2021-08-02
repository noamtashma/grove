//! The trees module.
//! This module contains:
//! * Traits that all implementations of trees should implement
//! * Specific implementations of trees
//!
//! The [`SomeWalker`] trait implements traversing a tree. This includes dealing with the borrow
//! checking problems of recursive structures (using [`recursive_reference`]), and rebalancing the tree.
//! Therefore, walkers can't guarantee that the tree won't change as you walk through them.

#[macro_use]
mod macros;
mod segment_algorithms;

pub mod avl;
pub mod basic_tree;
pub mod slice;
pub mod splay;
pub mod treap;

use crate::data::*;
use crate::locators;

/// Used to specify sidedness
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[allow(missing_docs)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    /// Flips the side
    pub fn flip(self) -> Self {
        match self {
            Side::Left => Self::Right,
            Side::Right => Self::Left,
        }
    }
}

/// This trait is the top-level trait that the different trees implement.
/// Every tree that implements this trait can be used directly by the functions
/// immediately in this trait.
/// More advanced use can be achieved by using walkers, which must be implemented.
pub trait SomeTree<D: Data>:
    SomeEntry<D> + std::iter::FromIterator<D::Value> + IntoIterator<Item = D::Value> + Default
where
    for<'a> &'a mut Self: SomeTreeRef<D>,
{
    // TODO: make immutable
    /// Compute the summary of a subsegment.
    /// Requires `D::Value: Clone`.
    ///
    /// Note: calling this on splay trees is inefficient
    /// and panicks in debug builds.
    /// (see documentation on splay tree's version of this function)
    fn segment_summary_imm<L>(&self, locator: L) -> D::Summary
    where
        L: locators::Locator<D>,
        D::Value: Clone;

    /// Compute the summary of a subsegment.
    /// Requires `D::Value: Clone` and mutable access.
    fn segment_summary<L>(&mut self, locator: L) -> D::Summary
    where
        L: locators::Locator<D>,
        D::Value: Clone,
    {
        self.segment_summary_imm(locator)
    }

    /// Compute the summary of a subsegment.
    /// Doesn't require `D::Value: Clone`, and works well in splay trees,
    /// but requires mutable access to `self`.
    fn segment_summary_unclonable<L>(&mut self, locator: L) -> D::Summary
    where
        L: locators::Locator<D>;

    /// Apply an action on a subsegment.
    fn act_segment<L>(&mut self, action: D::Action, locator: L)
    where
        L: locators::Locator<D>;

    /// Returns a value representing a specific subsegment of the tree. This gives a nicer
    /// Interface for tree operations: `tree.slice(3..50).act(action)` instead of
    /// `tree.act_segment(3..50, action)`. see [`slice::Slice`].
    fn slice<L: locators::Locator<D>>(&mut self, locator: L) -> slice::Slice<D, Self, L> {
        slice::Slice::new(self, locator)
    }

    /// This is here just so that the signature for iter_locator can be written out. Don't use this.
    type TreeData;

    /// Iterating on values.
    /// This iterator assumes you won't change the values using interior mutability. If you change the values,
    /// The tree summaries will behave incorrectly.
    ///
    /// See documentation in [`basic_tree::iterators`] as to why this function receives a `&mut self`
    /// instead of `&Self` input, and why there isn't a mutable iterator.
    ///```
    /// use grove::{SomeTree, basic_tree::BasicTree};
    /// use grove::example_data::StdNum;
    ///
    /// let mut tree: BasicTree<StdNum> = (20..80).collect();
    /// let segment_iter = tree.iter_locator(3..13);
    ///
    /// assert_eq!(segment_iter.cloned().collect::<Vec<_>>(), (23..33).collect::<Vec<_>>());
    /// # tree.assert_correctness();
    ///```
    fn iter_locator<'a, L: locators::Locator<D>>(
        &'a mut self,
        locator: L,
    ) -> basic_tree::iterators::IterLocator<'a, D, L, Self::TreeData>;

    /// Iterates over the whole tree.
    ///```
    /// use grove::{SomeTree, basic_tree::BasicTree};
    /// use grove::example_data::StdNum;
    ///
    /// let mut tree: BasicTree<StdNum> = (17..=89).collect();
    ///
    /// assert_eq!(tree.iter().cloned().collect::<Vec<_>>(), (17..=89).collect::<Vec<_>>());
    /// # tree.assert_correctness();
    ///```
    fn iter(
        &mut self,
    ) -> basic_tree::iterators::IterLocator<'_, D, std::ops::RangeFull, Self::TreeData> {
        self.iter_locator(..)
    }

    /// Used for testing purposes.
    /// Should panic if the invariants aren't satisfied.
    fn assert_correctness(&self)
    where
        D::Summary: Eq;
}

/// This is a workaround for not having Generic Associated Types in Rust yet.
/// Really, the type [`Self::Walker`] should have been defined in [`SomeTree`] and
/// should have been generic in a lifetime parameter.
pub trait SomeTreeRef<D: Data> {
    /// The walker type associated with this tree.
    /// for example, if `Self = &'a AVLTreee<D>` then `Self::Walker = AVLWalker<'a>`.
    /// The walker's lifetime comes from `Self`'s lifetime.
    type Walker: SomeWalker<D>;
    /// Creates a walker for the given tree.
    fn walker(self) -> Self::Walker;

    /// Finds any node that the locator `Accept`s.
    /// If there isn't any, it finds the empty location where that node would be instead.
    /// Returns a walker at the wanted position.
    fn search<L>(self, locator: L) -> Self::Walker
    where
        L: locators::Locator<D>,
        Self: Sized,
    {
        let mut walker = self.walker();
        walker.search_subtree(locator);
        walker
    }
}

/// The Walker trait implements walking through a tree.
/// This includes dealing with the borrow checking problems of recursive structures (using Telescope),
/// and rebalancing the tree.
/// Therefore, walkers can't guarantee that the tree won't change as you walk through them.
///
/// The walker should be able to walk on any of the existing nodes, or any empty position just near them.
/// i.e., The walker can also be in the position of a son of an existing node, where there isn't
/// a node yet.
/// The method [`SomeEntry::is_empty()`] can tell whether you are at an empty position. Trying to move downward from an
/// empty position produces an error value.
pub trait SomeWalker<D: Data>: SomeEntry<D> {
    /// Returns the current depth in the tree.
    /// The convention is, the root is at depth zero
    fn depth(&self) -> usize;

    /// This function is here since only walkers can guarantee that the current value
    /// is clean.
    fn value(&self) -> Option<&D::Value>;

    /// return `Err(())` if it is in an empty spot.
    fn go_left(&mut self) -> Result<(), ()>;
    /// returns `Err(())` if it is in an empty spot.
    fn go_right(&mut self) -> Result<(), ()>;
    /// If successful, returns whether or not the previous current value was the left son.
    /// If already at the root of the tree, returns `Err(())`.
    fn go_up(&mut self) -> Result<Side, ()>;
    /// Goes to the root.
    /// May restructure the tree while doing so. For example, in splay trees,
    /// this splays the current node.
    fn go_to_root(&mut self) {
        while self.go_up().is_ok() {}
    }

    /// If the walker is at an empty position, return an error.
    /// Goes to the next empty position.
    ///
    /// May restructure the tree while doing so.
    fn next_empty(&mut self) -> Result<(), ()> {
        if self.is_empty() {
            self.next_filled()?; // if already at the last empty node, returns error here.
        }
        // can't panic - already checked that position is nonempty
        self.go_right().expect("Expected nonempty position");
        while !self.is_empty() {
            self.go_left().unwrap();
        }
        Ok(())
    }

    /// If the walker is at an empty position, return an error.
    /// Goes to the previous empty position.
    ///
    /// May restructure the tree while doing so.
    fn previous_empty(&mut self) -> Result<(), ()> {
        if self.is_empty() {
            self.previous_filled()?; // if already at the first empty node, returns error here.
        }
        // can't panic - already checked that position is nonempty
        self.go_left().expect("Expected nonempty position");
        while !self.is_empty() {
            self.go_right().unwrap();
        }
        Ok(())
    }

    /// Finds the next filled node.
    /// If there isn't any, moves to root and return Err(()).
    ///
    /// May restructure the tree while doing so.
    fn next_filled(&mut self) -> Result<(), ()> {
        if !self.is_empty() {
            self.next_empty().unwrap();
        }
        loop {
            match self.go_up() {
                Ok(Side::Left) => break,
                Ok(Side::Right) => (),
                Err(_) => return Err(()), // there was no next node
            }
        }
        Ok(())
    }

    /// Finds the previous filled node.
    /// If there isn't any, moves to root and return Err(()).
    ///
    /// May restructure the tree while doing so.
    fn previous_filled(&mut self) -> Result<(), ()> {
        if !self.is_empty() {
            self.previous_empty().unwrap();
        }
        loop {
            match self.go_up() {
                Ok(Side::Right) => break,
                Ok(Side::Left) => (),
                Err(_) => return Err(()), // there was no next node
            }
        }
        Ok(())
    }

    /// Finds any node that the locator `Accept`s. Looks only inside the current subtree.
    /// If there isn't any, it finds the empty location where that node would be instead.
    /// Returns a walker at the wanted position.
    fn search_subtree<L: crate::Locator<D>>(&mut self, locator: L) {
        use locators::LocResult;
        while let Some(res) = locators::query_locator(self, &locator) {
            match res {
                LocResult::Accept => break,
                LocResult::GoRight => self.go_right().unwrap(),
                LocResult::GoLeft => self.go_left().unwrap(),
            };
        }
    }

    /// Returns a summary of all the values to the left of this point,
    /// That are not children of this point.
    fn far_left_summary(&self) -> D::Summary;
    /// Returns a summary of all the values to the right of this point,
    /// That are not children of this point.
    fn far_right_summary(&self) -> D::Summary;

    /// Returns a summary of all the values to the left of this point.
    /// If the walker is in a non empty spot, this does not include the current node.
    fn left_summary(&self) -> D::Summary {
        let left = self.far_left_summary();
        match self.left_subtree_summary() {
            Some(subtree) => left + subtree,
            None => left,
        }
    }

    /// Returns a summary of all the values to the right of this point.
    /// If the walker is in a non empty spot, this does not include the current node.
    fn right_summary(&self) -> D::Summary {
        let right = self.far_right_summary();
        match self.right_subtree_summary() {
            Some(subtree) => subtree + right,
            None => right,
        }
    }
}

/// Methods that ask to read the contents of the current tree/subtree.
/// These methods are common to the trees themselves and to the walkers.
pub trait SomeEntry<D: Data> {
    /// Lets you modify the value, and after you modified it, rebuilds the node.
    /// If the current position is empty, returns [`None`].
    fn with_value<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut D::Value) -> R;

    /// Returns [`true`] if the current tree/subtree is empty.
    fn is_empty(&self) -> bool {
        self.left_subtree_summary().is_none()
    }

    /// Returns a summary of just the current node.
    /// Returns the empty summary if at an empty position.
    fn node_summary(&self) -> D::Summary;

    /// Returns the summary of all values in this node's subtree.
    ///```
    /// use grove::{SomeTree, SomeEntry, basic_tree::BasicTree};
    /// use grove::example_data::StdNum;
    ///
    /// let tree: BasicTree<StdNum> = (1..=8).collect();
    /// let summary = tree.subtree_summary();
    ///
    /// assert_eq!(summary.size, 8);
    /// assert_eq!(summary.sum, 36);
    /// assert_eq!(summary.max, Some(8));
    /// assert_eq!(summary.min, Some(1));
    /// # tree.assert_correctness();
    ///```
    fn subtree_summary(&self) -> D::Summary;

    /// Returns the summary of the subtree of this node's left son.
    fn left_subtree_summary(&self) -> Option<D::Summary>;
    /// Returns the summary of the subtree of this node's right son.
    fn right_subtree_summary(&self) -> Option<D::Summary>;

    /// Applies the action on the current node.
    fn act_node(&mut self, action: D::Action) -> Option<()>;

    /// Applies the given action on this subtree.
    fn act_subtree(&mut self, action: D::Action);

    /// Applies the given action on this node's left son.
    fn act_left_subtree(&mut self, action: D::Action) -> Option<()>;
    /// Applies the given action on this node's right son.
    fn act_right_subtree(&mut self, action: D::Action) -> Option<()>;

    /// Used for testing purposes.
    /// Should panic if the local invariants aren't satisfied.
    fn assert_correctness_locally(&self)
    where
        D::Summary: Eq;

    #[cfg(debug_assertions)]
    /// This has to be specified in order for the `representation` method
    /// to work
    type EntryTreeData;

    #[cfg(debug_assertions)]
    /// Used for debugging. Prints a representation of the tree, like so:
    /// `< < * * > * >`
    /// Each pair of triangle brackets is a node, and `*` denotes empty trees.
    /// The trees are printed in the layout they will have atfter all reversals have been
    /// finished, but nodes which are yet to be reversed (`node.action.to_reverse() == true`)
    /// are printed with an exclamation mark: `<! * * >`.
    /// You can provide a custom printer for the alg_data field.
    /// If the input `to_reverse` is true, it will print the tree in reverse.
    fn representation<F>(&self, alg_print: &F, to_reverse: bool) -> String
    where
        F: Fn(&basic_tree::BasicNode<D, Self::EntryTreeData>) -> String;
}

/// Trait for trees that can be modified, i.e., values can be inserted and deleted.
///
/// This trait is a workaround for current rust type inference limitations.
/// It allows to write generic code for a tree type that has a modifiable walker.
/// Intuitively it should've been enough to require
/// `T: SomeTree<D>, for<'a> &'a mut T: SomeTreeRef<D>, for<'a> <&'a mut T as SomeTreeRef<D>>::Walker: ModifiableWalker`.
/// However, that doesn't work. Instead, use `for<'a> &'a mut T: ModifiableTreeRef<D>`.
pub trait ModifiableTreeRef<D: Data>: SomeTreeRef<D, Walker = Self::ModifiableWalker> {
    /// Inner type that ideally shouldn't be used - just use `Self::Walker`.
    type ModifiableWalker: ModifiableWalker<D>;
}

/// This is a trait for walkers that allow inserting and deleting values.
pub trait ModifiableWalker<D: Data>: SomeWalker<D> {
    /// Inserts the value into the tree at the current empty position.
    /// If the current position is not empty, returns [`None`].
    /// May end up at any possible location, depending on the tree type.
    fn insert(&mut self, value: D::Value) -> Option<()>;

    /// Removes the current value from the tree, and returns it.
    /// If currently at an empty position, returns [`None`].
    /// May end up at any possible location, depending on the tree type.
    fn delete(&mut self) -> Option<D::Value>;
}

/// Trait for trees that can concatenate.
/// I wanted this to be the same trait family as SplittableWalker, but the current rustc type solver didn't let me.
/// It's enough to only implement any one of the three methods - they're all implemented in terms of each other.
pub trait ConcatenableTree<D: Data>: SomeTree<D>
where
    for<'a> &'a mut Self: SomeTreeRef<D>,
{
    /// Concatenates the two inputs into one tree.
    fn concatenate(mut left: Self, right: Self) -> Self {
        left.concatenate_right(right);
        left
    }

    /// Concatenates the other tree to the right of this tree.
    fn concatenate_right(&mut self, mut other: Self) {
        let left = std::mem::take(self);
        other.concatenate_left(left);
        *self = other;
    }

    /// Concatenates the other tree to the left of this tree.
    fn concatenate_left(&mut self, other: Self) {
        let right = std::mem::take(self);
        *self = Self::concatenate(other, right);
    }
}
/// Trait for trees that can be split and concatenated.
/// Require this kind of tree if you want to use reversal actions on segments of your tree.
pub trait SplittableTreeRef<D: Data>:
    SomeTreeRef<D, Walker = Self::SplittableWalker> + Sized
{
    /// Inner type that ideally shouldn't be used - just use the original tree type.
    type T;
    /// Inner type that ideally shouldn't be used - just use `Self::Walker`.
    type SplittableWalker: SplittableWalker<D, T = Self::T>;
}

/// Walkers that can split a tree into two.
pub trait SplittableWalker<D: Data>: ModifiableWalker<D> {
    /// Inner type that ideally shouldn't be used - just use the original tree type.
    type T;

    /// Split out everything to the right of the current position, if it is an empty position.
    /// Otherwise returns [`None`].
    fn split_right(&mut self) -> Option<Self::T>;

    /// Split out everything to the left of the current position, if it is an empty position.
    /// Otherwise returns [`None`].
    fn split_left(&mut self) -> Option<Self::T>;
}
