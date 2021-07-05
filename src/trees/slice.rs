//! A convenience struct that represents a specific segment of a tree.

use super::*;
use crate::*;
use std::marker::PhantomData;

/// Returns a value representing a specific subsegment of the tree. This gives a nicer
/// Interface for tree operations: `tree.slice(3..50).act(action)` instead of
/// `tree.act_segment(3..50, action)`.
///
/// It allows to query the tree for a summary or to apply an action,
/// and other useful operations, with a nicer interface.
///
/// This struct essentially just forwards calls, mostly to the methods in the traits in [`crate::trees`].
pub struct Slice<'a, D, T, L> {
    phantom: PhantomData<D>,
    tree: &'a mut T,
    locator: L,
}

impl<'a, D: Data, T: SomeTree<D>, L: Locator<D>> Slice<'a, D, T, L>
where
    for<'b> &'b mut T: SomeTreeRef<D>,
{
    /// Creates a new slice that represents the locator's segment in the tree.
    pub fn new(tree: &'a mut T, locator: L) -> Self {
        Slice {
            phantom: PhantomData,
            tree,
            locator,
        }
    }

    /// Compute the summary of this subsegment.
    pub fn summary(&mut self) -> D::Summary {
        self.tree.segment_summary(self.locator.clone())
    }

    /// Apply an action on this subsegment.
    pub fn act(&mut self, action: D::Action) {
        self.tree.act_segment(action, self.locator.clone());
    }

    /// Finds any node in the current subsegment.
    /// If there isn't any, it finds the empty location where that node would be instead.
    /// Returns a walker at the wanted position.
    pub fn search(self) -> <&'a mut T as SomeTreeRef<D>>::Walker {
        methods::search(self.tree, self.locator)
    }

    /// Iterating on values.
    /// This iterator assumes you won't change the values using interior mutability. If you change the values,
    /// The tree summaries will behave incorrectly.
    ///
    /// The iterator receives a `&mut self` argument instead of a `&self` argument.
    /// Because of the way the trees work, immutable iterators can't be written without either mutable access
    /// to the tree, or assuming that the values are `Clone`.
    ///
    /// On the other hand, mutable iterators can't be written because the values of the nodes must be rebuilt,
    /// but they can only be rebuilt after the iterator exits. (This is because rust iterators can't be streaming iterators).
    /// If you want a mutable iterator, use a walker instead.
    pub fn iter(self) -> basic_tree::iterators::IterLocator<'a, D, L, T::TreeData> {
        self.tree.iter_locator(self.locator)
    }
}

impl<'a, D: Data, T: SomeTree<D>, L: Locator<D>> Slice<'a, D, T, L>
where
    for<'b> &'b mut T: ModifiableTreeRef<D>,
{
    /// Assumes that the this subsegment is empty.
    /// Inserts the value into the tree into the position of this empty subsegment.
    /// If the current subsegment is not empty, returns [`None`].
    pub fn insert(&mut self, value: D::Value) -> Option<()> {
        let mut walker = methods::search(&mut *self.tree, self.locator.clone());
        walker.insert(value)
    }

    /// Removes any value from this subsegment from tree, and returns it.
    /// If this subsegment is empty, returns [`None`].
    pub fn delete(&mut self) -> Option<D::Value> {
        let mut walker = methods::search(&mut *self.tree, self.locator.clone());
        walker.delete()
    }
}

impl<'a, D: Data, T: SomeTree<D>, L: Locator<D>> Slice<'a, D, T, L>
where
    for<'b> &'b mut T: SplittableTreeRef<D>,
{
    /// Assumes that the this subsegment is empty.
    /// Split out everything to the right of this subsegment, if it is an empty subsegment.
    /// Otherwise returns [`None`].
    pub fn split_right(
        &mut self,
    ) -> Option<<<&mut T as SplittableTreeRef<D>>::SplittableWalker as SplittableWalker<D>>::T>
    {
        let mut walker = methods::search(&mut *self.tree, self.locator.clone());
        walker.split_right()
    }

    /// Assumes that the this subsegment is empty.
    /// Split out everything to the left of the this subsegment, if it is an empty subsegment.
    /// Otherwise returns [`None`].
    pub fn split_left(
        &mut self,
    ) -> Option<<<&mut T as SplittableTreeRef<D>>::SplittableWalker as SplittableWalker<D>>::T>
    {
        let mut walker = methods::search(&mut *self.tree, self.locator.clone());
        walker.split_left()
    }
}
