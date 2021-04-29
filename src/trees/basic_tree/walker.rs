// This is a private module, so no documentation for it directly.
// instead look for documentation of the `BasicWalker` struct.

use super::*;
use crate::telescope::*;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::trees::SomeWalker; // in order to be able to use our own go_up method

pub (super) struct Frame<A : ?Sized + Data> {
	pub left : A::Summary,
	pub right : A::Summary,
}

// the default clone implementation requires that A : Clone, which is uneccessary
impl<A : ?Sized + Data> Clone for Frame<A> where A::Summary : Clone {
	fn clone(&self) -> Self {
		Frame { left : self.left.clone(), right : self.right.clone() }
	}
}

impl<A : Data> Frame<A> {
	pub fn empty() -> Frame<A> {
		Frame {left : A::EMPTY, right : A::EMPTY}
	}
}

// Invariant: the current node is always already accessed,
// and only nodes on the path from the root to the current node (exclusive) may have
// incorrect values.

/// This struct implements a walker for the [`BasicTree`] type.
/// It is struct that has a mutable reference of the tree, and allows you to walk up and down on it.
/// The walker may also be in a position which is the son of a node, but doesn't contain
/// a node by itself, and then it is said to be in an empty position.
///
/// The walker implements [`Deref`] and [`DerefMut`] with [`BasicTree`] as the target type,
/// as the walker acts as a smart pointer to a subtree of the original tree.
///
/// Walkers for other kinds of trees may be built by wrapping around the [`BasicWalker`] type,
/// as tree types can be built by wrapping around the [`BasicTree`] type.
/// 
/// The walker will automatically go back up the tree to the root when dropped,
/// in order to [`BasicNode::rebuild`] all the nodes.
///
/// Internally, a [`Telescope`] type is used, in order to be able to dynamically
/// go up and down the tree without upsetting the borrow checker.
#[derive(destructure)]
pub struct BasicWalker<'a, A : Data, T=()> {
	/// The telescope, holding references to all the subtrees from the root to the
	/// current position.
	pub(super) tel : Telescope<'a, BasicTree<A, T>>,

	/// This array holds the accumulation of all the values left of the subtree, and
	/// all of the values right of the subtree, for every subtree from the root to
	/// the current subtree.
	pub(super) vals : Vec<Frame<A>>,

	/// This array holds for every node, whether the next subtree in the walker
	/// is its left son or the right son. (true corresponds to the left son).
	/// This array is always one shorter than [`BasicWalker::tel`] and [`BasicWalker::vals`],
	/// because the last node has no son in the walker.
	pub(super) is_left : Vec<bool>,
}

impl<'a, A : Data, T> Deref for BasicWalker<'a, A, T> {
	type Target = BasicTree<A, T>;
	fn deref(&self) -> &BasicTree<A, T> {
		&*self.tel
	}
}

impl<'a, A : Data, T> DerefMut for BasicWalker<'a, A, T> {
	fn deref_mut(&mut self) -> &mut BasicTree<A, T> {
		&mut *self.tel
	}
}

impl<'a, D : Data, T> BasicWalker<'a, D, T> {
	pub fn new(tree : &'a mut BasicTree<D, T>) -> BasicWalker<'a, D, T> {
		tree.access();
		BasicWalker{ tel : Telescope::new(tree),
			        vals : vec![Frame::empty()],
					is_left : vec![] }
	}

	pub fn is_empty(&self) -> bool {
		matches!(&*self.tel, BasicTree::Empty)
	}
	
	/// If there is only the root, returns [`None`]
	/// Otherwise returns [`Some(true)`] if the current position is a left son
	/// [`Some(false)`] if the current position is a right son
	pub fn is_left_son(&self) -> Option<bool> {
		if self.is_left.is_empty() {
			None
		}
		else {
			Some(self.is_left[self.is_left.len() - 1])
		}
	}

	/// The convention is, the root is at depth zero
	pub fn depth(&self) -> usize {
		self.is_left.len()
	}

	/// Returns if this is the empty tree
	/// Note: even if you are the root, the root might still be empty.
	pub fn is_root(&self) -> bool {
		self.is_left.is_empty()
	}

	/// Performs a left rotation
	/// Returns [`Err(())`] if this is an empty tree or if it has no right son.
	pub fn rot_left(&mut self) -> Result<(), ()> {
		let owned_tree = std::mem::replace(&mut *self.tel, BasicTree::Empty);

		let mut bn1 : Box<BasicNode<D, T>> = match owned_tree {
			BasicTree::Empty => return Err(()),
			Root(bn) => bn,
		};
		assert!(bn1.action == D::IDENTITY);
		bn1.right.access();

		let mut bn2 : Box<BasicNode<D, T>> = match bn1.right {
			BasicTree::Empty => return Err(()),
			Root(bn) => bn,
		};

		bn1.right = bn2.left;
		bn2.subtree_summary = bn1.subtree_summary; // this is insetad of bn2.rebuild(), since we already know the result
		bn1.rebuild();
		bn2.left = Root(bn1);
		//bn2.rebuild();

		*self.tel = Root(bn2); // restore the node back
		Ok(())
	}

	/// Performs a right rotation
	/// Returns [`Err(())`] if this node has no left son.
	pub fn rot_right(&mut self) -> Result<(), ()> {
		let owned_tree = std::mem::replace(&mut *self.tel, BasicTree::Empty);

		let mut bn1 : Box<BasicNode<D, T>> = match owned_tree {
			BasicTree::Empty => return Err(()),
			Root(bn) => bn,
		};
		assert!(bn1.action == D::IDENTITY);
		bn1.left.access();

		let mut bn2 : Box<BasicNode<D, T>> = match bn1.left {
			BasicTree::Empty => return Err(()),
			Root(bn) => bn,
		};

		bn1.left = bn2.right;
		bn2.subtree_summary = bn1.subtree_summary; // this is insetad of bn2.rebuild(), since we already know the result
		bn1.rebuild();
		bn2.right = Root(bn1);
		//bn2.rebuild();

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
	pub fn root_into_ref(mut self) -> &'a mut BasicTree<D, T> {
		// go to the root
		self.go_to_root();
		let (tel, _, _) = self.destructure();
		tel.into_ref()
	}
}

/// This implementation exists in order to rebuild the nodes
/// when the walker gets dropped
impl<'a, A : Data, T> Drop for BasicWalker<'a, A, T> {
	fn drop(&mut self) {
		self.go_to_root();
	}
}


