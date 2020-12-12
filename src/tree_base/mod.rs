mod walker;

enum Tree<D> {
	Empty, Root(Box<Node<D>>)
}

use Tree::*;

impl<D> Tree<D> {
	fn data_mut(&mut self) -> Option<&mut D> {
		match self {
			Empty => None,
			Root(node) => Some(&mut node.data),
		}
	}
	fn data(&self) -> Option<&D> {
		match self {
			Empty => None,
			Root(node) => Some(&node.data),
		}
	}
}
impl<D : Data> Tree<D> {

	fn rebuild(&mut self) {
		match self {
			Root(node) => node.rebuild(),
			_ => (),
		}
	}
	
	fn access(&mut self) {
		match self {
			Root(node) => node.access(),
			_ => (),
		}
	}
}

struct Node<D> {
	data : D,
	left : Tree<D>,
	right : Tree<D>
}

// this trait represents the data that will be stored inside the tree.
// the data can include: keys, values, indices, heights, sizes, sums maximums and minimums of subtrees, actions to be performed on the subtrees,
// and whatever your heart desires for your data structure needs.
pub trait Data {
	// rebuild the associated data from the previous data and the sons.
	fn rebuild_data<'a>(&'a mut self, left : Option<&'a Self>, right : Option<&'a Self>);
	// clear the current actions in order for the user to access the node safely
	fn access<'a>(&'a mut self, left : Option<&'a mut Self>, right : Option<&'a mut Self>);
}

// TODO - trait RevData
// need to consider the design

impl<D : Data> Node<D> {
	fn rebuild(&mut self) {
		Data::rebuild_data(&mut self.data, self.left.data(), self.right.data()); 
	}
	
	fn access(&mut self) {
		Data::access(&mut self.data, self.left.data_mut(), self.right.data_mut());
		// TODO - reversing
	}
	
	fn new(mut data : D, left : Tree<D>, right : Tree<D>) -> Node<D> {
		// this must be written first because later the values are moved into the result
		data.rebuild_data(left.data(), right.data());
		Node {
			data,
			left,
			right,
		}
	}
}


