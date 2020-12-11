

enum Tree<K, V, D> {
	Empty, Root(Box<Node<K, V, D>>)
}

use Tree::*;

impl<K, V, D> Tree<K, V, D> {
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

struct Node<K, V, D> {
	key : K,
	value : V,
	data : D,
	left : Tree<K, V, D>,
	right : Tree<K, V, D>
}

pub trait Data<K, V> {
	// create the appropriate data given the node and its two sons
	fn rebuild_data<'a>(key : &'a K, value : &'a V, left : Option<&'a Self>, right : Option<&'a Self>) -> Self;
	// the same, and then put the result into current
	fn rebuild_data_inplace<'a>(current : &'a mut Self, key : &'a K, value : &'a V, left : Option<&'a Self>, right : Option<&'a Self>);
	// clear the current actions in order for the user to access the node safely
	fn access<'a>(current : &'a mut Self, key : &'a mut K, value : &'a mut V, left : Option<&'a mut Self>, right : Option<&'a mut Self>); 
}

// TODO - trait RevData
// need to consider the design

impl<K, V, D : Data<K, V>> Node<K, V, D> {
	fn rebuild(&mut self) {
		Data::rebuild_data_inplace(&mut self.data, &self.key, &self.value, self.left.data(), self.right.data()); 
	}
	
	fn access(&mut self) {
		Data::access(&mut self.data, &mut self.key, &mut self.value, self.left.data_mut(), self.right.data_mut());
		// TODO - reversing
	}
	
	fn new(key : K, value : V, left : Tree<K, V, D>, right : Tree<K, V, D>) -> Node<K, V, D> {
		// this must be written first because later the values are moved into the result
		let data = Data::rebuild_data(&key, &value, left.data(), right.data());
		Node {
			key,
			value,
			data,
			left,
			right,
		}
	}
}


