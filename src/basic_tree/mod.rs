pub mod walker;
pub mod implementations;

pub use implementations::*;
pub use crate::data::Data; // because everyone will need to specify Data for the generic parameters

use crate::trees::SomeEntry;
pub use implementations::*;

pub enum Tree<D> {
	Empty, Root(Box<Node<D>>) // TODO: rename Root
}
use Tree::*;

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
	pub(crate) left : Tree<D>,
	pub(crate) right : Tree<D>
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
