use super::*;
use super::super::telescope::*;
use std::ops::Deref;
use std::ops::DerefMut;

// a struct that takes a mutable reference of the tree, and allows you to walk on it.
// doesn't work for empty trees.
// TODO: make it work for empty trees? maybe?
// should automatically go back up the tree when dropped
struct TreeWalker<'a, D : Data> {
	tel : Telescope<'a, Node<D>>,
	// this array holds for every node, whether its left son is inside the walker
	// and not the right one.
	// this array is always one shorter than the telescope,
	// because the last node has no son in the structure.
	is_left : Vec<bool>,
	// TODO - deal with this array
}

impl<'a, D : Data> Deref for TreeWalker<'a, D> {
	type Target = Node<D>;
	fn deref(&self) -> &Node<D> {
		&*self.tel
	}
}

impl<'a, D : Data> DerefMut for TreeWalker<'a, D> {
	fn deref_mut(&mut self) -> &mut Node<D> {
		&mut *self.tel
	}
}

impl<'a, D : Data> TreeWalker<'a, D> {
	pub fn new(tree : &'a mut Node<D>) -> TreeWalker<'a, D> {
		TreeWalker{ tel : Telescope::new(tree),
					is_left : vec![] }
	}
	
	// returns Err if it's impossible to go left
	// otherwise returns Ok
	pub fn go_left(&mut self) -> Result<(), ()> {
		let res = self.tel.reborrow_result( &mut |node| {
				match &mut node.left {
					Empty => return Err(()),
					Root(left) => {
						left.access();
						return Ok(left)},
				}
			}
		);
		match res {
			Ok (()) => self.is_left.push(true), // went left
			Err(_) => (),
		};
		return res;
	}
	
	// returns Err if it's impossible to go right
	// otherwise returns Ok
	pub fn go_right(&mut self) -> Result<(), ()> {
		let res = self.tel.reborrow_result( &mut |node| {
			match &mut node.right {
				Empty => return Err(()),
				Root(right) => {
					right.access();
					return Ok(right)},
				}
			}
		);
		match res {
			Ok (()) => self.is_left.push(false), // went right
			Err(_) => (),
		};
		return res;
	}
	
	// if there is only the root, returns None
	// otherwise returns Some(true) is the current position is a left son
	// Some(falsE) if the current position is a right son
	pub fn is_left_son(&self) -> Option<bool> {
		if self.is_left.len() == 0 {
			None
		}
		else {
			Some(self.is_left[self.is_left.len() - 1])
		}
	}

	// if successful, returns whether or not the previous current value was the left son.
	pub fn go_up(&mut self) -> Result<bool, ()> {
		match self.is_left.pop() {
			None => return Err(()),
			Some(b) => { 
				self.tel.pop().expect("missing element from telescope");
				self.tel.rebuild();
				return Ok(b);
			},
		}
	}

	// returns Err(()) if this node has no right son.
	pub fn rot_left(&mut self) -> Result<(), ()> {
		let n1 : &mut Node<D> = &mut self.tel;
		n1.right.access();
		let right = std::mem::replace(&mut n1.right, Empty /* temporary */);
		let mut n2 : Box<Node<D>> = match right {
			Empty => return Err(()),
			Root(n2) => n2,
		};

		n1.right = n2.left;
		n2.left = Empty; // temporary

		 // this performs a potentially-heavy swap
		 // in the case that D is large
		 // TODO: figure out if there is a better way
		 // might require switching from Telescope<'a, Node>
		 // to Telescope<'a, Box<Node>>
		 // which is probably fine?
		std::mem::swap(n1, &mut n2);
		n2.rebuild();
		n1.left = Root(n2);
		n1.rebuild();
		n1.access(); // TODO: is this useless?
		return Ok(());
	}

	// returns Err(()) if this node has no left son.
	pub fn rot_right(&mut self) -> Result<(), ()> {
		let n1 : &mut Node<D> = &mut self.tel;
		n1.left.access();
		let left = std::mem::replace(&mut n1.left, Empty /* temporary */);
		let mut n2 : Box<Node<D>> = match left {
			Empty => return Err(()),
			Root(n2) => n2,
		};

		n1.left = n2.right;
		n2.right = Empty; // temporary

		 // this performs a potentially-heavy swap
		 // in the case that D is large
		 // TODO: figure out if there is a better way
		 // might require switching from Telescope<'a, Node>
		 // to Telescope<'a, Box<Node>>
		 // which is probably fine?
		std::mem::swap(n1, &mut n2);
		n2.rebuild();
		n1.right = Root(n2);
		n1.rebuild();
		n1.access(); // TODO: is this useless?
		return Ok(());
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
		return Ok(());
	}
}

impl<'a, D : Data> Drop for TreeWalker<'a, D> {
	fn drop(&mut self) {
		while let Ok(_) = self.go_up() {} // in order to rebuild the nodes
	}
}
