//! The basic tree module
//! This module implements basic unbalanced trees.

// these two should not be public as they are merely separate files
// for some of the functions of this module
mod walker;
mod implementations;

pub use implementations::*;
pub use walker::*;
pub use crate::data::*; // because everyone will need to specify Data for the generic parameters

// use crate::trees::SomeEntry;

/// A bisc tree. might be empty.
pub enum BasicTree<A : Action> {
	Empty, Root(Box<BasicNode<A>>) // TODO: rename Root
}
use BasicTree::*;

impl<A : Action> BasicTree<A> {

	/// Remakes the data that is stored in this node, based on its sons.
	/// This is necessary when the data in the sons might have changed.
	/// For example, after inserting a new node, all of the nodes from it to the root
	/// must be rebuilt, in order for the segment values accumulated over the whole
	/// subtree to be accurate.
	pub fn rebuild(&mut self) {
		match self {
			Root(node) => node.rebuild(),
			_ => (),
		}
	}
	
	/// Pushes any actions stored in this node to its sons.
	/// Actions stored in nodes are supposed to be eventually applied to its
	/// whole subtree. Therefore, in order to access a node cleanly, without
	/// the still-unapplied-function complicating things, you must `access()` the node.
	pub fn access(&mut self) {
		match self {
			Root(node) => node.access(),
			_ => (),
		}
	}

	/// Returns the summary of all values in this node's subtree.
	pub fn segment_value(&self) -> A::Value {
		match self {
			Root(node) => node.segment_value(),
			_ => A::EMPTY,
		}
	}
}

impl<A : Action + Reverse> BasicTree<A> {
	/// Reverses the whole tree
	pub fn reverse(&mut self) {
		if let Root(node) = self {
			node.reverse();
		}
	}
}

impl<A : Action + Reverse> BasicNode<A> {
	/// calls access after calling the reverse action.
	pub fn reverse(&mut self) {
		Reverse::internal_reverse(self);
		self.access();
	}
}

// TODO: decide if the fields should really be public
/// A basic node. can be viewed as a non-empty basic tree: it always has at least one value.
pub struct BasicNode<A : Action> {
	action : A,
	segment_value : A::Value,
	node_value : A::Value,
	pub left : BasicTree<A>,
	pub right : BasicTree<A>
}

impl<A : Action> BasicNode<A> {

	pub fn new(value : A::Value) -> BasicNode<A> {
		BasicNode {
			action : A::IDENTITY,
			node_value : value,
			segment_value : value.clone(),
			left : Empty,
			right : Empty,
		}
	}
	
	/// Returns the summary of all values in this node's subtree.
	pub fn segment_value(&self) -> A::Value {
		self.action.act(self.segment_value)
	}

	/// Returns the value stored in this node specifically.
	pub fn node_value(&self) -> A::Value {
		self.action.act(self.node_value)
	}

	/// Pushes any actions stored in this node to its sons.
	/// Actions stored in nodes are supposed to be eventually applied to its
	/// whole subtree. Therefore, in order to access a node cleanly, without
	/// the still-unapplied-function complicating things, you must `access()` the node.
	pub fn access(&mut self) {
		// reversing
		// for data that doesn't implement reversing, this becomes a no-op
		// and hopefully optimized away
		if self.action.to_reverse() {
			std::mem::swap(&mut self.left, &mut self.right);
		}

		if let Root(node) = &mut self.left {
			node.act(self.action);
		}
		if let Root(node) = &mut self.right {
			node.act(self.action);
		}
		self.segment_value = self.action.act(self.segment_value);
		self.node_value = self.action.act(self.node_value);
		self.action = A::IDENTITY;
	}

	/// Remakes the data that is stored in this node, based on its sons.
	/// This is necessary when the data in the sons might have changed.
	/// For example, after inserting a new node, all of the nodes from it to the root
	/// must be rebuilt, in order for the segment values accumulated over the whole
	/// subtree to be accurate.
	pub fn rebuild(&mut self) {
		assert!(self.action == A::IDENTITY);
		self.segment_value = self.node_value.clone();
		if let Root(node) = &self.left {
			self.segment_value = A::compose_v(node.segment_value(), self.segment_value);
		}
		if let Root(node) = &self.right {
			self.segment_value = A::compose_v(self.segment_value, node.segment_value());
		}

		//Data::rebuild_data(&mut self.data, self.left.data(), self.right.data());
	}

	/// This function applies the given action to its whole subtree.
	///
	/// This function leaves the `self.action` field "dirty" - after calling
	/// this you might need to call access, to push the action to this node's sons.
	pub fn act(&mut self, action : A) {
		self.action = A::compose_a(action, self.action);
	}

	/*
	pub fn create(mut data : D, left : BasicTree<D>, right : BasicTree<D>) -> BasicNode<D> {
		// this must be written first because later the values are moved into the result
		data.rebuild_data(left.data(), right.data());
		BasicNode {
			data,
			left,
			right,
		}
	}
	*/
}

/*
impl<D : Action> std::ops::Deref for BasicNode<D> {
	type Target = D;
	fn deref(&self) -> &D {
		&self.data
	}
}

impl<D : Action> std::ops::DerefMut for BasicNode<D> {
	fn deref_mut(&mut self) -> &mut D {
		&mut self.data
	}
}
*/
