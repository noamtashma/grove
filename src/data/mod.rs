//! This module representes the values that can be put inside a segment tree,
//! the summaries that you can receive from a query about a segment in the segment tree,
//! and the actions that you can perform on the segment tree.
//!
//! In order for a choice of types for `Value`, `Summary` and `Action`, to work
//! in a segment tree, they must be an instance of the [`Data`] trait.
//!
//! In addition, this module provides the [`SizedData`] and [`Keyed`] traits,
//! and some common possible instantiations in the [`example_data`] module.


pub mod example_data;
pub use example_data::{SizedData, Keyed};

use std::ops::Add;

// TODO: remove Eq requirement from Self::Action
/// This trait represents the data that will be stored inside the tree.
///
/// Every node in the tree will contain an action to be performed on the node's subtree,
/// a summary of the node's subtree, and a value.
/// The action will be of type [`Self::Action`], the summary of type [`Self::Summary`], and the value will be of type [`Self::Value`].
///
/// [`Self::Summary`] can include: indices, heights, sizes, sums, maximums
/// and minimums of subtrees, and more. It is the type of summaries of values
/// you can have in your tree. This is the result of querying for information about a segment.
///
/// [`Self::Action`] is the type of actions that can be performed on segments. for example,
/// reverse a subtree, add a constant to a subtree, apply `max` with a constant on a subtree,
/// and so on.
///
/// Action composition is done by requiring that the action implement [`Add`].
/// i.e., applying `a + b` should be equivalent to applying `b` and then applying `a`.
/// This composition must be associative. 
/// Composition is right to left. i.e., what chronologically happens first, is on the right.
///
/// Summary composition is done by requiring that the summary implement [`Add`]. i.e.,
/// if the summary for a list `vals1` is `summary1`, and the summary for a list `vals2`
/// is `summary2`, the summary of the concatenation `vals1 + vals2` should be `summary1 + summary2`.
/// This composition must be associative.
///
/// In addition for the compositions to be associative,
/// the instance should obey the following rules:
///```rust,compile
/// # use orchard::*;
/// # use orchard::Data;
/// # fn test<D : Data>(
///	# action : D::Action, action1 : D::Action, action2 : D::Action,
/// # summary : D::Summary, summary1 : D::Summary, summary2 : D::Summary,
/// # mut value : D::Value)  where D::Summary : Eq {
/// // composition of actions
/// D::act_summary(action2 + action1, summary) == D::act_summary(action2, D::act_summary(action1, summary));
/// // composition of actions
/// D::act_value(action2 + action1, &mut value) == {
/// 	D::act_value(action1, &mut value);
/// 	D::act_value(action2, &mut value);
/// };
/// // the action respects the monoid structure
/// D::act_summary(action, summary1 + summary2) == D::act_summary(action, summary1) + D::act_summary(action, summary2);
/// // the action respects `to_summary`
/// # let _ = 
/// {
/// 	D::act_value(action, &mut value); // first act on value
/// 	D::to_summary(&value)             // take summary
/// } == {                               // vs
/// 	let sum = D::to_summary(&value);  // first take summary
/// 	D::act_value(action, &mut value); // act on value
/// 	D::act_summary(action, sum)               // act on summary to reflect acting on the value
/// };
/// # }
///```
/// If the action also implements [`Reverse`], it should also satisfy that composing two actions
/// xor's their [`Data::to_reverse()`] results.
pub trait Data {
	/// The values that reside in trees.
	type Value;
	/// The actions you can perform on the values
	type Action : Eq + Copy + Add<Output=Self::Action>;
	/// The summaries of values over segments. When querying a segment,
	/// you get a "summary" of the segment.
	type Summary : Copy + Add<Output=Self::Summary>;

	/// The identity action.
	const IDENTITY : Self::Action;
	/// The empty summary
	const EMPTY : Self::Summary;

	/// creates the summary of a single value.
	fn to_summary(val : &Self::Value) -> Self::Summary;

	/// Applying an action to a value.
	/// The default implementation just returns the value, since
	/// this is always a legal implementation.
	///
	/// Applying an action on a summary value must be equal to applying the
	/// action separately and then summing the values:
	///```rust,ignore
	///action.act(summary_1 + summary_2)
	///== action.act(summary_1) + action.act(summary_2)
	///```
	/// Indeed, this property is used so that the summary values can be updated without
	/// updating the whole tree.
	///
	/// Therefore, This is essentially a monoid action by the monoid of actions
	/// on the monoid of values.
	fn act_summary(_act : Self::Action, other : Self::Summary) -> Self::Summary {
		other
	}
	
	/// The action, but on values instead of summaries.
	/// Must commute with [`Data::to_summary`].
	fn act_value(_act : Self::Action, _other : &mut Self::Value) {}

	/// This function should be implemented if you want to be able to reverse subtrees of your tree,
	/// i.e., if you also implement [`Reverse`].
	///
	/// Note that if the action reverses a segment, it shouldn't be used with [`methods::act_segment`].
	/// Instead, use a tree type that supports reversals (e.g, SplayTree, Treap) and use its native
	/// [`SomeTree::act_segment`] function.
	///
	/// This function should return whether this action reverses the segment it is applied to.
	fn to_reverse(_action : Self::Action) -> bool {
		false
	}
}