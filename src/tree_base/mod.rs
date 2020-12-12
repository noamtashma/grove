pub mod walker;
pub use super::data::*;

pub enum Tree<D> {
	Empty, Root(Box<Node<D>>)
}

use Tree::*;

impl<D> Tree<D> {
	pub fn data_mut(&mut self) -> Option<&mut D> {
		match self {
			Empty => None,
			Root(node) => Some(&mut node.data),
		}
	}
	pub fn data(&self) -> Option<&D> {
		match self {
			Empty => None,
			Root(node) => Some(&node.data),
		}
	}
}

impl<D : Data> Tree<D> {

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
}

pub struct Node<D> {
	data : D,
	left : Tree<D>,
	right : Tree<D>
}

impl<D : Data> Node<D> {
	pub fn rebuild(&mut self) {
		Data::rebuild_data(&mut self.data, self.left.data(), self.right.data()); 
	}
	
	pub fn access(&mut self) {
		Data::access(&mut self.data, self.left.data_mut(), self.right.data_mut());
		// TODO - reversing
	}
	
	pub fn new(mut data : D, left : Tree<D>, right : Tree<D>) -> Node<D> {
		// this must be written first because later the values are moved into the result
		data.rebuild_data(left.data(), right.data());
		Node {
			data,
			left,
			right,
		}
	}
}


impl<D : Data> std::ops::Deref for Node<D> {
	type Target = D;
	fn deref(&self) -> &D {
		&self.data
	}
}

impl<D : Data> std::ops::DerefMut for Node<D> {
	fn deref_mut(&mut self) -> &mut D {
		&mut self.data
	}
}
