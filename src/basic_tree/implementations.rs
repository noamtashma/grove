use super::*;
use super::walker::*;
use crate::trees::*;
use crate::trees::SomeEntry;

impl<'a, D : Data> SomeTree<D> for Tree<D> {
    fn into_inner(self) -> Tree<D> {
        self
    }

    fn new() -> Self {
        Empty
    }

    fn from_inner(tree : Tree<D>) -> Self {
        tree
    }
}

impl<'a, D : Data> SomeTreeRef<D> for &'a mut Tree<D> {
    type Walker = BasicWalker<'a, D>;

    fn walker(self) -> Self::Walker {
        BasicWalker::new(self)
    }
}

impl<'a, D : Data> SomeWalker<D> for BasicWalker<'a, D> {
    // returns Err if it's impossible to go left
	// otherwise returns Ok
	fn go_left(&mut self) -> Result<(), ()> {
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
	fn go_right(&mut self) -> Result<(), ()> {
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
}

impl<'a, D : Data> SomeWalkerUp<D> for BasicWalker<'a, D> {
	
	// if successful, returns whether or not the previous current value was the left son.
	fn go_up(&mut self) -> Result<bool, ()> {
		match self.is_left.pop() {
			None => Err(()),
			Some(b) => { 
				self.tel.pop().expect("missing element from telescope");
				self.tel.rebuild();
				Ok(b)
			},
		}
	}
}

impl<D : Data> SomeEntry<D> for Tree<D> {
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

	fn write(&mut self, data : D) -> Option<D> {
        match self {
			Empty => {
                *self = Root(Box::new(Node::new(data, Empty, Empty)));
                self.rebuild();
				None
			},
			Root(node) => {
				let old_data = std::mem::replace(&mut node.data, data);
				node.rebuild();
				Some(old_data)
			},
		}
	}
	

    fn insert_new(&mut self, data : D) -> Result<(), ()> {
        match self {
			Empty => {
				*self = Root(Box::new(Node::new(data, Empty, Empty)));
				self.rebuild();
				Ok(())
			},
			Root(_) => Err(()),
		}
    }
}

impl<'a, D : Data> SomeEntry<D> for BasicWalker<'a, D> {
    fn data_mut(&mut self) -> Option<&mut D> {
        self.tel.data_mut()
    }

    fn data(&self) -> Option<&D> {
        self.tel.data()
    }

    fn write(&mut self, data : D) -> Option<D> {
        self.tel.write(data)
    }

    fn insert_new(&mut self, data : D) -> Result<(), ()> {
        self.tel.insert_new(data)
    }
}