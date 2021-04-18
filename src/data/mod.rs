pub mod example_data;


// TODO: remove Eq
/// This trait represents the data that will be stored inside the tree.
///
/// Every node in the tree will contain an action, and a value.
/// The action will be of type `Self`, and the value will be of type `Self::Value`.
///
/// `Self::Value` can include: keys, values, indices, heights, sizes, sums maximums
/// and minimums of subtrees, and more. It is the type of values and summaries of values
/// you can have in your tree.
///
/// `Self` is the type of actions that can be performed on the subtrees. for example,
/// reverse a subtree, add a constant to a subtree, apply `max` with a constant on a subtree,
/// and so on.
pub trait Action : Copy + Eq {
	/// Action composition. i.e., applying the resulting action should be equivalent
	/// to applying the `other` function and then the `self` action.
	/// Therefore, this composition must be associative. 
	/// Compose right to left. i.e., what chronologically happens first, is on the right.
	fn compose_a(self, other : Self) -> Self;
	/// The identity action.
	const IDENTITY : Self;

	type Value : Copy;
	/// Value composition. This is used to create the summary values
	/// That give information about whole subtrees.
	/// Since the structure of the tree may be any structure,
	/// but the summary value should depend on the values in the subtree and
	/// not on the tree structure, this composition must be associative.
	fn compose_v(left : Self::Value, right : Self::Value) -> Self::Value;
	const EMPTY : Self::Value;

	// default implementation that does nothing:
	/// Applying an action to a value.
	/// The default implementation just returns the value, since
	/// this is always a legal implementation.
	///
	/// Applying an action on a summary value must be equal to applying the
	/// action separately and then summing the values:
	/// ```
	///    action.act(Self::compose_v(value_1, value_2))
	///    == Self::compose_v(action.act(value_1), action.act(value_2))
	/// ```
	/// Indeed, this property is used so that the summary values can be updated without
	/// updating the whole tree.
	///
	/// Therefore, This is essentially a monoid action by the monoid of actions
	/// on the monoid of values.
	fn act(self, other : Self::Value) -> Self::Value {
		other
	}
	/*
	fn act_inplace(&self, other : &mut Self::Value) -> () {
		let res = self.act(other);
		*other = res;
	}
	*/

	/// This function should be implemented if you want to be able to reverse subtrees of your tree,
	/// i.e., if you also implement Reverse.
	
	/// This function should return whether this action reverses the segment it is applied to.
	fn to_reverse(&self) -> bool {
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
pub trait Reverse : Action {
	/// Mark the action in the node that it should be reversed.
	fn internal_reverse(node : &mut crate::trees::basic_tree::BasicNode<Self>);
}