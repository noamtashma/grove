// these two should not be public as they are merely separate files
// for some of the functions of this module
mod walker;
mod implementations;

pub use implementations::*;
pub use walker::*;
pub use crate::data::*; // because everyone will need to specify Data for the generic parameters

// use crate::trees::SomeEntry;

pub enum BasicTree<A : Action> {
	Empty, Root(Box<BasicNode<A>>) // TODO: rename Root
}
use BasicTree::*;

impl<A : Action> BasicTree<A> {

	pub fn rebuild(&mut self) {
		match self {
			Root(node) => node.rebuild(),
			_ => (),
		}
	}
	
	pub fn access(&mut self) {
		match self {
			Root(node) => node.access(),
			_ => (),
		}
	}

	pub fn segment_value(&self) -> A::Value {
		match self {
			Root(node) => node.segment_value(),
			_ => A::EMPTY,
		}
	}
}

impl<A : Action + Reverse> BasicTree<A> {
	pub fn reverse(&mut self) {
		if let Root(node) = self {
			node.reverse();
		}
	}
}

impl<A : Action + Reverse> BasicNode<A> {
	pub fn reverse(&mut self) {
		self.action.reverse();
		self.access();
	}
}

// TODO: decide if the fields should really be public
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
	
	pub fn segment_value(&self) -> A::Value {
		self.action.act(self.segment_value)
	}

	pub fn node_value(&self) -> A::Value {
		self.action.act(self.node_value)
	}

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

	/// Does not call access
	/// After calling this you might need to call access again
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
