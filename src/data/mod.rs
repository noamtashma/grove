pub mod example_data;

/// this trait represents the data that will be stored inside the tree.
/// the data can include: keys, values, indices, heights, sizes, sums maximums and minimums of subtrees, actions to be performed on the subtrees,
/// and whatever your heart desires for your data structure needs.
pub trait Data {
	/// rebuild the associated data from the previous data and the sons.
	fn rebuild_data<'a>(&'a mut self, left : Option<&'a Self>, right : Option<&'a Self>);
	/// clear the current actions in order for the user to access the node safely
	fn access<'a>(&'a mut self, left : Option<&'a mut Self>, right : Option<&'a mut Self>);

	/// these two functions should be implemented if you want to be able to reverse subtrees of your tree.
	/// this function should return whether you would like to reverse your subtree
	/// and zero it back - calling to_reverse() twice should always return false the second time.

	/// it doesn't matter in which function the actual effect of the reverse happens,
	/// however, you can only pick one
	fn to_reverse(&mut self) -> bool {
		false
	}

	/// this function should flip the bit of whether you'll want to reverse your data
	fn reverse(&mut self) {
		panic!("didn't implement reverse for a D : Reverse");
	}
}

// TODO: remove Eq
pub trait Action : Copy + Eq {
	// compose right to left. i.e., what chronologically happens first, is on the right.
	fn compose_a(self, other : Self) -> Self;
	const IDENTITY : Self;

	type Value : Copy;
	fn compose_v(left : Self::Value, right : Self::Value) -> Self::Value;
	const EMPTY : Self::Value;
	// default implementation that does nothing:
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
pub trait Reverse {
	/// This function should flip the bit of whether this action will reverse the data.
	fn reverse(&mut self);

	// alternative: have a constant action which is just a reversal,
	// instead of a method which adds a reversal to an existing action.
	// this is equivalent because we can already combine actions.
	// const REVERSE : Self;
}