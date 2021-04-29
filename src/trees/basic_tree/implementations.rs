//! This module implements the tree traits for the [`BasicTree`] and [`BasicWalker`]
//! It is mostly a separate file from the main module file, since it's a private module, and its
//! contents are re-exported.

use super::*;
use super::super::*; // crate::trees::*
use crate::telescope::NO_VALUE_ERROR;

impl<'a, A : Data> SomeTree<A> for BasicTree<A> {
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

impl<'a, A : Data> SomeTreeRef<A> for &'a mut BasicTree<A> {
    type Walker = BasicWalker<'a, A>;

    fn walker(self) -> Self::Walker {
        BasicWalker::new(self)
    }
}

impl<'a, A : Data> SomeWalker<A> for BasicWalker<'a, A> {
    /// Returns Err if it's impossible to go left
	/// otherwise returns Ok
	fn go_left(&mut self) -> Result<(), ()> {
		let mut frame = self.vals.last().expect(crate::telescope::NO_VALUE_ERROR).clone();
		let res = self.tel.extend_result( |tree| {
			if let Some(node) = tree.node_mut() {
				// update values
				frame.right = node.node_summary() + node.right.subtree_summary() + frame.right;
				node.left.access();
				Ok(&mut node.left)
			} else { Err(()) }
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
			if let Some(node) = tree.node_mut() {
				// update values
				frame.left = frame.left + node.left.subtree_summary() + node.node_summary();
				
				node.right.access();
				Ok(&mut node.right)
			} else { Err(()) }
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

	// fn inner(&self) -> &BasicTree<A> {
    //     &*self.tel
    // }

	fn value(&self) -> Option<&A::Value> {
		let value = self.node()?.node_value_clean();
		Some(value)
	}
}


impl<D : Data> SomeEntry<D> for BasicTree<D> {
	fn value_mut(&mut self) -> Option<&mut D::Value> {
		let value = &mut self.node_mut()?.node_value;
		Some(value)
	}

	fn node_summary(&self) -> D::Summary {
		match self.node() {
			None => D::EMPTY,
			Some(node) => node.node_summary()
		}
	}

    fn insert_new(&mut self, value : D::Value) -> Result<(), ()> {
        match self {
			Empty => {
				*self = BasicTree::new(BasicNode::new(value));
				Ok(())
			},
			_ => Err(()),
		}
    }

	fn subtree_summary(&self) -> D::Summary {
		if let Some(node) = self.node() {
			node.subtree_summary()
		} else { D::EMPTY }
	}

    fn left_subtree_summary(&self) -> Option<D::Summary> {
        let res = self.node()?.left.subtree_summary();
		Some(res)
    }

    fn right_subtree_summary(&self) -> Option<D::Summary> {
        let res = self.node()?.right.subtree_summary();
		Some(res)
    }

    fn with_value<F, R>(&mut self, f : F) -> Option<R> where 
        F : FnOnce(&mut D::Value) -> R {
        let res = f(self.value_mut()?);
    	self.access();
    	Some(res)
    }

    fn act_subtree(&mut self, action : D::Action) {
        if let Some(node) = self.node_mut() {
			node.act(action);
		}
    }
}

impl<'a, D : Data> SomeEntry<D> for BasicWalker<'a, D> {
    fn value_mut(&mut self) -> Option<&mut D::Value> {
        self.tel.value_mut()
    }

	fn node_summary(&self) -> D::Summary {
		self.tel.node_summary()
	}

    fn insert_new(&mut self, value : D::Value) -> Result<(), ()> {
        self.tel.insert_new(value)
    }

    fn subtree_summary(&self) -> D::Summary {
        self.tel.subtree_summary()
    }

    fn left_subtree_summary(&self) -> Option<D::Summary> {
        self.tel.left_subtree_summary()
    }

    fn right_subtree_summary(&self) -> Option<D::Summary> {
        self.tel.right_subtree_summary()
    }

    fn with_value<F, R>(&mut self, f : F) -> Option<R> where 
        F : FnOnce(&mut D::Value) -> R {
        let res = f(self.value_mut()?);
		self.access();
		Some(res)
    }

    fn act_subtree(&mut self, action : D::Action) {
        self.act_subtree(action);
    }
}