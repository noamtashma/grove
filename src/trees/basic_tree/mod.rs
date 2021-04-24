//! The basic tree module
//! This module implements basic unbalanced trees.

// these two should not be public as they are merely separate files
// for some of the functions of this module
mod walker;
pub use walker::*;

mod implementations;
pub use implementations::*;

pub mod iterators;

use crate::*;
//pub use crate::data::*; // because everyone will need to specify Data for the generic parameters

// use crate::trees::SomeEntry;

/// A basic tree. might be empty.
pub enum BasicTree<A : ?Sized + Data> {
	Empty, Root(Box<BasicNode<A>>) // TODO: rename Root
}
use BasicTree::*;

impl<D : Data> BasicTree<D> {
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
	pub fn subtree_summary(&self) -> D::Summary {
		match self {
			Root(node) => node.subtree_summary(),
			_ => D::EMPTY,
		}
	}

	/// Iterates over the whole tree.
	pub fn iter(&mut self) -> impl Iterator<Item=&D::Value> {
		iterators::ImmIterator::new(self, methods::locator::all::<D>)
	}

	/// Iterates over the given segment.
	pub fn iter_locator<L>(&mut self, loc : L) -> impl Iterator<Item=&D::Value> where
		L : methods::locator::Locator<D>
	{
		iterators::ImmIterator::new(self, loc)
	}
}

impl<A : Data + Reverse> BasicTree<A> {
	/// Reverses the whole tree
	pub fn reverse(&mut self) {
		if let Root(node) = self {
			node.reverse();
		}
	}
}

impl<A : Data + Reverse> BasicNode<A> {
	/// calls access after calling the reverse action.
	pub fn reverse(&mut self) {
		Reverse::internal_reverse(self);
		self.access();
	}
}

// TODO: decide if the fields should really be public
/// A basic node. can be viewed as a non-empty basic tree: it always has at least one value.
pub struct BasicNode<A : ?Sized + Data> {
	action : A::Action,
	subtree_summary : A::Summary,
	pub (crate) node_value : A::Value,
	pub left : BasicTree<A>,
	pub right : BasicTree<A>
}

impl<A : Data> BasicNode<A> {

	pub fn new(value : A::Value) -> BasicNode<A> {
		let subtree_summary = A::to_summary(&value);
		BasicNode {
			action : A::IDENTITY,
			node_value : value,
			subtree_summary,
			left : Empty,
			right : Empty,
		}
	}
	
	/// Returns the summary of all values in this node's subtree.
	pub fn subtree_summary(&self) -> A::Summary {
		return A::act(self.action, self.subtree_summary);
	}

	/// Returns a summary for the value in this node specifically,
	/// and not the subtree.
	pub fn node_summary(&self) -> A::Summary {
		let summary = A::to_summary(&self.node_value);
		A::act(self.action, summary)
	}

	/// Returns the value stored in this node specifically.
	/// Requires mutable access because it calls `access`, to ensure
	/// that the action applies.
	pub fn node_value(&mut self) -> &A::Value {
		self.access();
		&self.node_value
	}

	/// Returns the value stored in this node specifically.
	pub fn node_value_mut(&mut self) -> &mut A::Value {
		self.access();
		&mut self.node_value
	}

	/// Returns the value stored in this node specifically.
	/// Assumes that the node has been accessed. Panics otherwise.
	pub fn node_value_clean(&self) -> &A::Value {
		assert!(self.action == A::IDENTITY);
		&self.node_value
	}

	/// Pushes any actions stored in this node to its sons.
	/// Actions stored in nodes are supposed to be eventually applied to its
	/// whole subtree. Therefore, in order to access a node cleanly, without
	/// the still-unapplied-function complicating things, you must `access()` the node.
	pub fn access(&mut self) {
		// reversing
		// for data that doesn't implement reversing, this becomes a no-op
		// and hopefully optimized away
		if A::to_reverse(self.action) {
			std::mem::swap(&mut self.left, &mut self.right);
		}

		if let Root(node) = &mut self.left {
			node.act(self.action);
		}
		if let Root(node) = &mut self.right {
			node.act(self.action);
		}
		self.subtree_summary = A::act(self.action, self.subtree_summary);
		A::act_value(self.action, &mut self.node_value);
		self.action = A::IDENTITY;
	}

	/// Remakes the data that is stored in this node, based on its sons.
	/// This is necessary when the data in the sons might have changed.
	/// For example, after inserting a new node, all of the nodes from it to the root
	/// must be rebuilt, in order for the segment values accumulated over the whole
	/// subtree to be accurate.
	pub fn rebuild(&mut self) {
		assert!(self.action == A::IDENTITY);
		self.subtree_summary = A::to_summary(&self.node_value);
		if let Root(node) = &self.left {
			self.subtree_summary = node.subtree_summary() + self.subtree_summary;
		}
		if let Root(node) = &self.right {
			self.subtree_summary = self.subtree_summary + node.subtree_summary();
		}

		//Data::rebuild_data(&mut self.data, self.left.data(), self.right.data());
	}

	/// This function applies the given action to its whole subtree.
	///
	/// This function leaves the `self.action` field "dirty" - after calling
	/// this you might need to call access, to push the action to this node's sons.
	pub fn act(&mut self, action : A::Action) {
		self.action = action + self.action;
	}

	/*
	pub fn create(mut data : A, left : BasicTree<A>, right : BasicTree<A>) -> BasicNode<A> {
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
impl<A : Action> std::ops::Deref for BasicNode<A> {
	type Target = A;
	fn deref(&self) -> &A {
		&self.data
	}
}

impl<A : Action> std::ops::DerefMut for BasicNode<A> {
	fn deref_mut(&mut self) -> &mut A {
		&mut self.data
	}
}
*/
