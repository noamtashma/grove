use super::*;
use crate::telescope::*;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::trees::SomeWalker; // in order to be able to use our own go_up method

#[derive(Clone)]
pub (super) struct Frame<A : Action> {
	pub left : A::Value,
	pub right : A::Value,
}

impl<A : Action> Frame<A> {
	pub fn empty() -> Frame<A> {
		Frame {left : A::EMPTY, right : A::EMPTY}
	}
}

// Invariant: the current node is always already accessed,
// and only nodes on the path from the root to the current node (exclusive) may have
// incorrect values.
/// A struct that has a mutable reference of the tree, and allows you to walk on it.
/// will automatically go back up the tree when dropped, in order to rebuild() all the nodes.
#[derive(destructure)]
pub struct BasicWalker<'a, A : Action> {
	pub(super) tel : Telescope<'a, BasicTree<A>>,

	// this array holds the accumulation of all the values left of the node, and
	// all of the values right of the node, for every node.
	pub(super) vals : Vec<Frame<A>>,

	// this array holds for every node, whether its left son is inside the walker
	// and not the right one.
	// this array is always one shorter than the telescope,
	// because the last node has no son in the structure.
	pub(super) is_left : Vec<bool>,
}

impl<'a, A : Action> Deref for BasicWalker<'a, A> {
	type Target = BasicTree<A>;
	fn deref(&self) -> &BasicTree<A> {
		&*self.tel
	}
}

impl<'a, A : Action> DerefMut for BasicWalker<'a, A> {
	fn deref_mut(&mut self) -> &mut BasicTree<A> {
		&mut *self.tel
	}
}

impl<'a, A : Action> BasicWalker<'a, A> {
	pub fn new(tree : &'a mut BasicTree<A>) -> BasicWalker<'a, A> {
		tree.access();
		BasicWalker{ tel : Telescope::new(tree),
			        vals : vec![Frame::empty()],
					is_left : vec![] }
	}

	pub fn is_empty(&self) -> bool {
		matches!(&*self.tel, BasicTree::Empty)
	}
	
	// if there is only the root, returns None
	// otherwise returns Some(true) if the current position is a left son
	// Some(false) if the current position is a right son
	pub fn is_left_son(&self) -> Option<bool> {
		if self.is_left.is_empty() {
			None
		}
		else {
			Some(self.is_left[self.is_left.len() - 1])
		}
	}

	// the convention is, the root is at depth zero
	pub fn depth(&self) -> usize {
		self.is_left.len()
	}

	// note: even if you are the root, the root might still be empty,
	// if this is the empty tree
	pub fn is_root(&self) -> bool {
		self.is_left.is_empty()
	}

	// returns Err(()) if this is an empty tree or if it has no right son.
	pub fn rot_left(&mut self) -> Result<(), ()> {
		let owned_tree = std::mem::replace(&mut *self.tel, BasicTree::Empty);

		let mut bn1 : Box<BasicNode<A>> = match owned_tree {
			BasicTree::Empty => return Err(()),
			Root(bn) => bn,
		};

		bn1.right.access();

		let mut bn2 : Box<BasicNode<A>> = match bn1.right {
			BasicTree::Empty => return Err(()),
			Root(bn) => bn,
		};

		bn1.right = bn2.left;
		bn1.rebuild();
		bn2.left = Root(bn1);
		bn2.rebuild();

		*self.tel = Root(bn2); // restore the node back
		Ok(())
	}

	// returns Err(()) if this node has no left son.
	pub fn rot_right(&mut self) -> Result<(), ()> {
		let owned_tree = std::mem::replace(&mut *self.tel, BasicTree::Empty);

		let mut bn1 : Box<BasicNode<A>> = match owned_tree {
			BasicTree::Empty => return Err(()),
			Root(bn) => bn,
		};

		bn1.left.access();

		let mut bn2 : Box<BasicNode<A>> = match bn1.left {
			BasicTree::Empty => return Err(()),
			Root(bn) => bn,
		};

		bn1.left = bn2.right;
		bn1.rebuild();
		bn2.right = Root(bn1);
		bn2.rebuild();

		*self.tel = Root(bn2); // restore the node back
		Ok(())
	}

	/// Performs rot_left if b is true
	/// rot_right otherwise
	pub fn rot_side(&mut self, b : bool) -> Result<(), ()> {
		if b {
			self.rot_left()
		} else {
			self.rot_right()
		}
	}

	/// Rotates so that the current node moves up.
	/// Basically moves up and then calls rot_side.
	/// Fails if the current node is the root.
	pub fn rot_up(&mut self) -> Result<bool, ()> {
		let b = self.go_up()?;
		self.rot_side(!b).expect("original node went missing?");
		Ok(b)
	}

	pub fn go_to_root(&mut self) {
		while !self.is_root() {
			self.go_up().unwrap();
		}
	}

	/// This takes the walker and turns it into a reference to the root
	pub fn root_into_ref(mut self) -> &'a mut BasicTree<A> {
		// go to the root
		self.go_to_root();
		let (tel, _, _) = self.destructure();
		tel.into_ref()
	}
}

/// This implementation exists in order to rebuild the nodes
/// when the walker gets dropped
impl<'a, A : Action> Drop for BasicWalker<'a, A> {
	fn drop(&mut self) {
		self.go_to_root();
	}
}


