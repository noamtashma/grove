//! This module representes the values that can be put inside a segment tree,
//! the summaries that you can receive from a query about a segment in the segment tree,
//! and the actions that you can perform on the segment tree.
//!
//! In order for a choice of types for `Value`, `Summary` and `Action`, to work
//! in a segment tree, they must be an instance of the [`Data`] trait.
//!
//! In addition, this module provides the [`SizedSummary`] and [`Keyed`] traits,
//! and some common possible instantiations in the [`example_data`] module.

pub mod example_data;
pub use example_data::{Keyed, SizedSummary};

use std::ops::Add;

/// This trait represents the data that will be stored inside the tree.
///
/// Every node in the tree will contain an action to be performed on the node's subtree,
/// a summary of the node's subtree, and a value.
/// The action will be of type [`Self::Action`], the summary of type [`Self::Summary`], and the value will be of type [`Self::Value`].
///
/// * [`Self::Value`] is the type of values in your tree, which can be anything at all. The tree
/// representes a sequence of values of type [`Self::Value`].
///
/// * [`Self::Summary`] can include: indices, sizes, sums, maximums
/// and minimums of subtrees, and more. It is the type of information you can gather about a subsegment
/// of your tree. This is the result of querying for information about a segment.
///
/// * [`Self::Action`] is the type of actions that can be performed on subsegments. for example,
/// reverse a subsegment, add a constant to all values in a subsegment, apply `max` with a
/// constant on all values in a subsegment, and so on.
///
/// All of the trait's requirements are modeled as separated traits in order to allow for easier
/// mixing and matching of different values, summaries and actions.
///
/// # Requirements
///
/// In order for everything to work, we must know how to:
/// * Compose actions together:
///    >  Action composition is done by an [`Add`] instance.
///    >  i.e., applying `a + b` should be equivalent to applying `b` and then applying `a`.
///    >  Composition is right to left. What chronologically happens first, is on the right.
///
/// * Compute the summary of a single value, and add up summaries of two subsegments together:
///  > Summaries of segments are created by converting single values into their singletone
///  > summaries using `to_summary()`, and summaries can be added together to obtain bigger summaries, using an
///  > [`Add`] instance. i.e.,
///  > if the summary for a list `vals_list1` is `summary1`, and the summary for a list `vals_list2`
///  > is `summary2`, the summary of the concatenation `vals_list1 + vals_list2` should be `summary1 + summary2`.
///
/// * Apply an action on a single value, and apply an action on a summary of a segment
///  > Applying actions on values and on summaries is required by the bounds
///  > [`Self::Action`]`: `[`Acts`]`<`[`Self::Value`]`> + `[`Acts`]`<`[`Self::Summary`]`>`.
///  > This means that in order to update a segment summary `summary: `[`Self::Summary`] after
///  > action `action: `[`Self::Action`] has been applied to it,
///  > you could execute `summary = action.act(summary);` or `action.act_inplace(&mut summary);`,
///  > and similarly as well for [`Self::Value`].
///
/// Additional requirements:
/// * Decide whether to reverse the subsegment it is acted upon. This is done by implementing the
/// [`Action::to_reverse()`] function. If you do not want to reverse segments, you can use the default implementation,
/// which always returns false.
/// * Have an identity action and empty summary: These are represented by the bounds [`Self::Action`]`: `[`Default`],
/// [`Self::Summary`]`: `[`Default`].
/// * Test actions for being the identity. This is represented by [`Action::is_identity()`].
///
/// # Rules
/// In order for the segment trees to work correctly, all of these operations must play nicely with each other.
/// * composition of actions: action composition must be associative, and the
///   identity action should actually be the identity.
///   ```notrust
///   (action3 + action2) + action1 === action3 + (action2 + action1)
///   default() + summary === action + default() === action
///   ```
///
/// * Action composition must actually be action composition, and the identity must actually be the identity.
///   ```notrust
///   (action2 + action1).act(value) == action2.act(action1.act(value))
///   (action2 + action1).act(summary) == action2.act(action1.act(summary))
///   default().act(value) === value
///   default().act(summary) === summary
///   ```
///
/// * Summary addition must be associative, so that we get the same summary regardless of the tree structure,
///   and the empty summary must be the identity.
///   ```notrust
///   (summary1 + summary2) + summary3 === summary1 + (summary2 + summary3)
///   default() + action === summary + default() === summary
///   ```
///
/// * Adding up summaries and then applying an action must be equal to applying the
///   action separately and then summing the parts:
///   ```notrust
///   action.act(summary1 + summary2) == action.act(summary1) + action.act(summary2)
///   ```
///   This property is used so that the summary values can be updated without
///   updating the whole tree.
///   This means that the action respects the monoid structure of the summaries.
///   If the action reverses segments, (i.e, if `D::to_reverse(action) == true`), then it has to satisfy a 'cross'
///   version instead:
///   ```notrust
///   action.act(summary1 + summary2) == action.act(summary2) + action.act(summary1)
///   ```
///
/// * The action should respect [`Self::to_summary()`]:
///   ```notrust
///   D::to_summary(&action.act(value)) === action.act(D::to_summary(&value))
///   ```
///
/// * If the action can reverse segments, it should also satisfy that composing two actions
///   xor's their [`Action::to_reverse()`] results:
///   ```notrust
///   D::to_reverse(action2 + action1) === D::to_reverse(action2) ^ D::to_reverse(action1)
///   ```
pub trait Data {
    /// The values that reside in trees.
    type Value: ToSummary<Self::Summary>;
    /// The summaries of values over segments. When querying a segment,
    /// you get a summary of the segment, represented by a value of type `Self::Summary`.
    type Summary: Copy + Default + Add<Output = Self::Summary>;
    /// The actions you can perform on the values
    type Action: Action + Acts<Self::Value> + Acts<Self::Summary>;

    /// Creates the summary of a single value.
    fn to_summary(val: &Self::Value) -> Self::Summary {
        val.to_summary()
    }
}

/// A [`Data`] implementation for a generic triplet of value, summary and action types,
/// so you don't have to make an `impl` yourself.
impl<V, S, A> Data for (V, S, A)
where
    V: ToSummary<S>,
    S: Copy + Default + Add<Output = S>,
    A: Action + Acts<V> + Acts<S>,
{
    type Value = V;
    type Summary = S;
    type Action = A;
}

/// Trait representing actions. this entailes having an identity action ([`Default`]), being able to compose actions
/// ([`Add`]`<Output=Self>`), checking whether an action is the identity action, and checking whether this action
/// reverses subsegments.
pub trait Action: Copy + Default + Add<Output = Self> {
    /// Test whether this action is the identity action.
    fn is_identity(self) -> bool;

    /// This function should be implemented if you want to be able to reverse subsegments of your tree.
    /// The default implementation always returns `false`.
    ///
    /// Note that if the action reverses a segment, it shouldn't be used with [`crate::methods::act_segment`].
    /// Instead, use a tree type that supports reversals (e.g, SplayTree, Treap) and use its native
    /// [`crate::SomeTree::act_segment`] function.
    ///
    /// This function should return whether this action reverses the segment it is applied to.
    fn to_reverse(self) -> bool {
        false
    }
}

/// Trait representation actions on a type `V`. If `A: Acts<V>` that means that given any `action: A`,
/// we can apply it to any `val: V`. This trait is used to represent the actions on
/// values and summaries used by segment trees.
pub trait Acts<V> {
    /// Act on a value in-place.
    fn act_inplace(&self, object: &mut V);
    /// Act on a value and return the result.
    fn act(&self, mut object: V) -> V {
        self.act_inplace(&mut object);
        object
    }
}

/// This trait is implemented by Values,
/// and provides a conversion from a value to the summary of that single value.
pub trait ToSummary<S> {
    /// Creates the summary of a single value.
    fn to_summary(&self) -> S;
}
