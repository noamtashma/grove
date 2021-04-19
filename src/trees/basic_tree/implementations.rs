//! This module implements the tree traits for the BasicTree and BasicWalker
//! It is mostly a separate file from the main module file, since it's a private module, and its
//! contents are re-exported.

use super::*;
use super::super::*; // crate::trees::*
use crate::telescope::NO_VALUE_ERROR;

impl<'a, A : Action> SomeTree<A> for BasicTree<A> {
    fn into_inner(self) -> BasicTree<A> {
        self
    }

    fn new() -> Self {
        Empty
    }

    fn from_inner(tree : BasicTree<A>) -> Self {
        tree
    }
}

impl<'a, A : Action> SomeTreeRef<A> for &'a mut BasicTree<A> {
    type Walker = BasicWalker<'a, A>;

    fn walker(self) -> Self::Walker {
        BasicWalker::new(self)
    }
}

impl<'a, A : Action> SomeWalker<A> for BasicWalker<'a, A> {
    // returns Err if it's impossible to go left
	// otherwise returns Ok
	fn go_left(&mut self) -> Result<(), ()> {
		let mut frame = self.vals.last().expect(crate::telescope::NO_VALUE_ERROR).clone();
		let res = self.tel.extend_result( |tree| {
			match tree {
				Empty => Err(()),
				Root(node) => {
					// update values
					frame.right = A::compose_s(node.right.segment_summary(), frame.right);
					frame.right = A::compose_s(node.node_summary(), frame.right);
					node.left.access();
					Ok(&mut node.left)
				},
			}
		}
		);
		// push side information
		if res.is_ok() {
			self.is_left.push(true); // went left
			self.vals.push(frame);
		}
		return res;
	}
	
	// returns Err if it's impossible to go right
	// otherwise returns Ok
	fn go_right(&mut self) -> Result<(), ()> {
		let mut frame = self.vals.last().expect(crate::telescope::NO_VALUE_ERROR).clone();
		let res = self.tel.extend_result( |tree| {
			match tree {
				Empty => Err(()),
				Root(node) => {
					// update values
					frame.left = A::compose_s(frame.left, node.left.segment_summary());
					frame.left = A::compose_s(frame.left, node.node_summary());
					
					node.right.access();
					Ok(&mut node.right)
				},
			}
		}
		);
		// push side information
		if res.is_ok() {
			self.is_left.push(false); // went right
			self.vals.push(frame);
		}
		return res;
	}

	// if successful, returns whether or not the previous current value was the left son.
	fn go_up(&mut self) -> Result<bool, ()> {
		match self.is_left.pop() {
			None => Err(()),
			Some(b) => { 
				self.tel.pop().expect(NO_VALUE_ERROR);
				self.vals.pop().expect(NO_VALUE_ERROR);
				self.tel.rebuild();
				Ok(b)
			},
		}
	}

	fn depth(&self) -> usize {
		self.depth()
	}

	fn far_left_summary(&self) -> A::Summary {
		self.vals.last().expect(NO_VALUE_ERROR).left
	}
	fn far_right_summary(&self) -> A::Summary {
		self.vals.last().expect(NO_VALUE_ERROR).right
	}

	fn inner_mut(&mut self) -> &mut BasicTree<A> {
        &mut *self.tel
    }

	fn inner(&self) -> &BasicTree<A> {
        &*self.tel
    }
}


impl<A : Action> SomeEntry<A> for BasicTree<A> {
	fn value_mut(&mut self) -> Option<&mut A::Value> {
		match self {
			Empty => None,
			Root(node) => Some(&mut node.node_value),
		}
	}

	fn value(&self) -> Option<&A::Value> {
		match self {
			Empty => None,
			Root(node) => Some(&node.node_value),
		}
	}

	fn node_summary(&self) -> A::Summary {
		match self {
			Empty => A::EMPTY,
			Root(node) => node.node_summary()
		}
	}

	/*
	fn write(&mut self, data : A) -> Option<A> {
        match self {
			Empty => {
                *self = Root(Box::new(BasicNode::new(data, Empty, Empty)));
				self.access();
				self.rebuild();
				None
			},
			Root(node) => {
				let old_data = std::mem::replace(&mut node.data, data);
				node.access();
				node.rebuild();
				Some(old_data)
			},
		}
	}
	*/
	

    fn insert_new(&mut self, value : A::Value) -> Result<(), ()> {
        match self {
			Empty => {
				*self = Root(Box::new(BasicNode::new(value)));
				Ok(())
			},
			Root(_) => Err(()),
		}
    }
}

impl<'a, A : Action> SomeEntry<A> for BasicWalker<'a, A> {
    fn value_mut(&mut self) -> Option<&mut A::Value> {
        self.tel.value_mut()
    }

    fn value(&self) -> Option<&A::Value> {
        self.tel.value()
    }

	fn node_summary(&self) -> A::Summary {
		self.tel.node_summary()
	}

	/*
    fn write(&mut self, data : A) -> Option<A> {
        self.tel.write(data)
	}
	*/

    fn insert_new(&mut self, value : A::Value) -> Result<(), ()> {
        self.tel.insert_new(value)
    }
}