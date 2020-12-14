use super::*;
use super::super::telescope::*;
use std::ops::Deref;
use std::ops::DerefMut;

// a struct that takes a mutable reference of the tree, and allows you to walk on it.
// doesn't work for empty trees.
// TODO: make it work for empty trees? maybe?
// maybe switch to Telescope<'a, Box<Tree<D>>>
// should automatically go back up the tree when dropped


#[derive(destructure)]
pub struct TreeWalker<'a, D : Data> {
	tel : Telescope<'a, Tree<D>>,
	// this array holds for every node, whether its left son is inside the walker
	// and not the right one.
	// this array is always one shorter than the telescope,
	// because the last node has no son in the structure.
	is_left : Vec<bool>,
	// TODO - deal with this array
}

impl<'a, D : Data> Deref for TreeWalker<'a, D> {
	type Target = Tree<D>;
	fn deref(&self) -> &Tree<D> {
		&*self.tel
	}
}

impl<'a, D : Data> DerefMut for TreeWalker<'a, D> {
	fn deref_mut(&mut self) -> &mut Tree<D> {
		&mut *self.tel
	}
}

impl<'a, D : Data> TreeWalker<'a, D> {
	pub fn new(tree : &'a mut Tree<D>) -> TreeWalker<'a, D> {
		TreeWalker{ tel : Telescope::new(tree),
					is_left : vec![] }
	}

	pub fn is_empty(&self) -> bool {
		matches!(&*self.tel, Tree::Empty)
	}
	
	// returns Err if it's impossible to go left
	// otherwise returns Ok
	pub fn go_left(&mut self) -> Result<(), ()> {
		let res = self.tel.extend_result( &mut |tree| {
				match tree {
					Empty => Err(()),
					Root(node) => {
						node.access();
						Ok(&mut node.left)},
				}
			}
		);
		if res.is_ok() {
			self.is_left.push(true); // went left
		}
		return res;
	}
	
	// returns Err if it's impossible to go right
	// otherwise returns Ok
	pub fn go_right(&mut self) -> Result<(), ()> {
		let res = self.tel.extend_result( &mut |tree| {
			match tree {
				Empty => Err(()),
				Root(node) => {
					node.access();
					Ok(&mut node.right)},
				}
			}
		);
		if res.is_ok() {
			self.is_left.push(false); // went right
		}
		return res;
	}
	
	// if there is only the root, returns None
	// otherwise returns Some(true) is the current position is a left son
	// Some(falsE) if the current position is a right son
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

	// if successful, returns whether or not the previous current value was the left son.
	pub fn go_up(&mut self) -> Result<bool, ()> {
		match self.is_left.pop() {
			None => Err(()),
			Some(b) => { 
				self.tel.pop().expect("missing element from telescope");
				self.tel.rebuild();
				Ok(b)
			},
		}
	}

	// returns Err(()) if this is an empty tree or if it has no right son.
	pub fn rot_left(&mut self) -> Result<(), ()> {
		let owned_tree = std::mem::replace(&mut *self.tel, Tree::Empty);

		let mut bn1 : Box<Node<D>> = match owned_tree {
			Tree::Empty => return Err(()),
			Root(bn) => bn,
		};

		bn1.right.access();

		let mut bn2 : Box<Node<D>> = match bn1.right {
			Tree::Empty => return Err(()),
			Root(bn) => bn,
		};

		bn1.right = bn2.left;
		bn1.rebuild();
		bn2.left = Root(bn1);
		bn2.rebuild();
		//bn2.access(); // is this necessary? this seems useless

		*self.tel = Root(bn2); // restore the node back
		Ok(())
	}

	// returns Err(()) if this node has no left son.
	pub fn rot_right(&mut self) -> Result<(), ()> {
		let owned_tree = std::mem::replace(&mut *self.tel, Tree::Empty);

		let mut bn1 : Box<Node<D>> = match owned_tree {
			Tree::Empty => return Err(()),
			Root(bn) => bn,
		};

		bn1.left.access();

		let mut bn2 : Box<Node<D>> = match bn1.left {
			Tree::Empty => return Err(()),
			Root(bn) => bn,
		};

		bn1.left = bn2.right;
		bn1.rebuild();
		bn2.right = Root(bn1);
		bn2.rebuild();
		//bn2.access(); // is this necessary? this seems useless

		*self.tel = Root(bn2); // restore the node back
		Ok(())
	}

	// performs rot_left if b is true
	// rot_right otherwise
	pub fn rot_side(&mut self, b : bool) -> Result<(), ()> {
		if b {
			self.rot_left()
		} else {
			self.rot_right()
		}
	}

	// rotates so that the current node moves up.
	// basically moves up and then calls rot_side.
	// fails if the current node is the root.
	pub fn rot_up(&mut self) -> Result<(), ()> {
		let b = self.go_up()?;
		self.rot_side(!b).expect("original node went missing?");
		Ok(())
	}

	// this takes the walker and turns it into a reference to the root
	pub fn root_into_ref(mut self) -> &'a mut Tree<D> {
		// go to the root
		while !self.is_root() {
			self.go_up();
		}
		let (tel, _) = self.destructure();
		tel.into_ref()
	}
}

// this implementation exists in order to rebuild the nodes
// when the walker goes out of scope
impl<'a, D : Data> Drop for TreeWalker<'a, D> {
	fn drop(&mut self) {
		while !self.is_root() {
			self.go_up();
		}
	}
}