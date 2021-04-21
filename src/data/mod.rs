pub mod example_data;

use std::ops::Add;

/// Used for cases where no action or no summary is needed.
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Default, PartialOrd, Ord)]
pub struct Unit {}

impl Add for Unit {
	type Output = Unit;
	fn add(self, _b : Unit) -> Unit {
		Unit {}
	}
}

// TODO: remove Eq requirement from Self::Action
/// This trait represents the data that will be stored inside the tree.
///
/// Every node in the tree will contain an action to be performed on the node's subtree,
/// a summary of the node's subtree, and a value.
/// The action will be of type [`Self::Action`], the summary of type `Self::Summary`, and the value will be of type `Self::Value`.
///
/// `Self::Summary` can include: indices, heights, sizes, sums, maximums
/// and minimums of subtrees, and more. It is the type of summaries of values
/// you can have in your tree. This is the result of querying for information about a segment.
///
/// `Self::Action` is the type of actions that can be performed on segments. for example,
/// reverse a subtree, add a constant to a subtree, apply `max` with a constant on a subtree,
/// and so on.
///
/// Action composition is done by requiring that the action implement `Add<Output=Self::Action>`.
/// i.e., applying `a + b` should be equivalent to applying `b` and then applying `a`.
/// This composition must be associative. 
/// Composition is right to left. i.e., what chronologically happens first, is on the right.
///
/// Summary composition is done by requiring that the summary implement `Add<Output=Self::Summary>`.
/// Since the structure of the tree may be any structure,
/// and the summary value should depend on the values in the subtree and
/// not on the tree structure, this composition must be associative.
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
	/// ```
	///    action.act(Self::compose_v(summary_1, summary_2))
	///    == Self::compose_v(action.act(summary_1), action.act(summary_2))
	/// ```
	/// Indeed, this property is used so that the summary values can be updated without
	/// updating the whole tree.
	///
	/// Therefore, This is essentially a monoid action by the monoid of actions
	/// on the monoid of values.
	fn act(_act : Self::Action, other : Self::Summary) -> Self::Summary {
		other
	}
	
	/// The action, but on values instead of summaries.
	/// Must commute with `to_summary`.
	fn act_value(_act : Self::Action, _other : &mut Self::Value) {}

	/// This function should be implemented if you want to be able to reverse subtrees of your tree,
	/// i.e., if you also implement Reverse.
	///
	/// This function should return whether this action reverses the segment it is applied to.
	fn to_reverse(_action : Self::Action) -> bool {
		false
	}
}

/// Marker trait for Data that implement reverse.
/// If you want your data structure to be able to reverse subtrees,
/// implement this marker trait, and the `Action::to_reverse` function.

/// Note that if the action reverses a segment, it shouldn't be used on the regular functions
/// that apply an action to a segment, because that would reverse different parts of the segment
/// separately. Instead, it should work with the split-then-apply variants. (TODO: implement)

/// The `to_reverse` function is part of the `Action` trait and not this trait,
/// in order that the `access` function can work for both reversible and non reversible
/// actions uniformly.
pub trait Reverse : Data {
	/// Mark the action in the node that it should be reversed.
	fn internal_reverse(node : &mut crate::trees::basic_tree::BasicNode<Self>);
}