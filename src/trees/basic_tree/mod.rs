// these two should not be public as they are merely separate files
// for some of the functions of this module
mod walker;
mod implementations;

pub use implementations::*;
pub use walker::*;
pub use crate::data::*; // because everyone will need to specify Data for the generic parameters

use crate::trees::SomeEntry;

pub enum BasicTree<D> {
	Empty, Root(Box<BasicNode<D>>) // TODO: rename Root
}
use BasicTree::*;

impl<D : Data> BasicTree<D> {

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

	// private function
	// outer callers should only be able to call this if D : Reverse
	fn internal_reverse(&mut self) {
		if let Root(node) = self {
			node.reverse();
		}
	}
}

impl<D : Reverse> BasicTree<D> {
	pub fn reverse(&mut self) {
		self.internal_reverse();
	}
}



pub struct BasicNode<D> {
	data : D,
	pub(crate) left : BasicTree<D>,
	pub(crate) right : BasicTree<D>
}

impl<D : Data> BasicNode<D> {
	pub fn rebuild(&mut self) {
		Data::rebuild_data(&mut self.data, self.left.data(), self.right.data());
	}
	
	pub fn access(&mut self) {
		Data::access(&mut self.data, self.left.data_mut(), self.right.data_mut());
		// reversing
		// for data that doesn't implement reversing, this becomes a no-op
		// and hopefully optimized away
		if self.data.to_reverse() {
			std::mem::swap(&mut self.left, &mut self.right);
			self.left.internal_reverse();
			self.right.internal_reverse();
		}
	}
	
	pub fn new(mut data : D, left : BasicTree<D>, right : BasicTree<D>) -> BasicNode<D> {
		// this must be written first because later the values are moved into the result
		data.rebuild_data(left.data(), right.data());
		BasicNode {
			data,
			left,
			right,
		}
	}
}


impl<D : Data> std::ops::Deref for BasicNode<D> {
	type Target = D;
	fn deref(&self) -> &D {
		&self.data
	}
}

impl<D : Data> std::ops::DerefMut for BasicNode<D> {
	fn deref_mut(&mut self) -> &mut D {
		&mut self.data
	}
}
