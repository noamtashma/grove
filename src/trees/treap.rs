//! Implementation of treaps
//!
//! It is a balancced tree algorithm that supports reversals, splitting and concatenation.
//!
//! Its operations take `O(log n)` expected time, probabilistically.
//! Each operation may take up to linear time, but the probability of any operation
//! taking more than `O(log n)` time  is extremely low.



use super::*;
use super::basic_tree::*;
use rand;

// The type that is used for bookkeeping.
// convention: a smaller number should go higher up the tree.
type T = usize;

pub struct Treap<D : Data> {
    tree : BasicTree<D, T>,
}

impl<D : Data> SomeTree<D> for Treap<D> {
    fn segment_summary<L>(&mut self, locator : L) -> D::Summary where
        L : crate::Locator<D> {
            methods::segment_summary(self, &locator)
    }

    fn act_segment<L>(&mut self, action : D::Action, locator : L) where
        L : crate::Locator<D> {
            methods::act_segment(self, action, &locator)
    }
}

impl<D : Data> Default for Treap<D> {
    fn default() -> Self {
        Treap::new()
    }
}

impl<'a, D : Data> SomeTreeRef<D> for &'a mut Treap<D> {
    type Walker = TreapWalker<'a, D>;

    fn walker(self) -> Self::Walker {
        TreapWalker { walker : self.tree.walker() }
    }
}

impl<D : Data> SomeEntry<D> for Treap<D> {
    fn value_mut(&mut self) -> Option<&mut D::Value> {
        self.tree.value_mut()
    }

    fn with_value<F, R>(&mut self, f : F) -> Option<R> where 
        F : FnOnce(&mut D::Value) -> R {
        self.tree.with_value(f)
    }

    fn node_summary(&self) -> D::Summary {
        self.tree.node_summary()
    }

    fn subtree_summary(&self) -> D::Summary {
        self.tree.subtree_summary()
    }

    fn left_subtree_summary(&self) -> Option<D::Summary> {
        self.tree.left_subtree_summary()
    }

    fn right_subtree_summary(&self) -> Option<D::Summary> {
        self.tree.right_subtree_summary()
    }

    fn act_subtree(&mut self, action : D::Action) {
        self.tree.act_subtree(action);
    }

    fn act_node(&mut self, action : D::Action) -> Option<()> {
        self.tree.act_node(action)
    }

    fn act_left_subtree(&mut self, action : D::Action) -> Option<()> {
        self.tree.act_left_subtree(action)
    }

    fn act_right_subtree(&mut self, action : D::Action) -> Option<()> {
        self.tree.act_right_subtree(action)
    }
}

impl<D : Data> Treap<D> {
    pub fn new() -> Treap<D> {
        Treap { tree : BasicTree::Empty }
    }

    pub fn priority(&self) -> Option<T> {
        Some(self.tree.node()?.alg_data)
    }

    /// Iterates over the whole tree.
	///```
	/// use orchard::basic_tree::*;
	/// use orchard::example_data::StdNum;
	///
	/// let mut tree : BasicTree<StdNum> = (17..=89).collect();
	///
	/// assert_eq!(tree.iter().cloned().collect::<Vec<_>>(), (17..=89).collect::<Vec<_>>());
	/// # tree.assert_correctness();
	///```
	pub fn iter(&mut self) -> impl Iterator<Item=&D::Value> {
		self.tree.iter()
	}

    /// Iterates over the given segment.
	///```
	/// use orchard::basic_tree::*;
	/// use orchard::example_data::StdNum;
	/// use orchard::locator;
	///
	/// let mut tree : BasicTree<StdNum> = (20..80).collect();
	/// let segment_iter = tree.iter_locator(locator::locate_by_index_range(3,13)); // should also try 3..5
	///
	/// assert_eq!(segment_iter.cloned().collect::<Vec<_>>(), (23..33).collect::<Vec<_>>());
	/// # tree.assert_correctness();
	///```
	pub fn iter_locator<L>(&mut self, loc : L) -> impl Iterator<Item=&D::Value> where
        L : methods::locator::Locator<D>
    {
        self.tree.iter_locator(loc)
    }

	/// This function applies the given action to its whole subtree.
	///
	/// This function leaves the [`self.action`] field "dirty" - after calling
	/// this you might need to call access, to push the action to this node's sons.
	///```
	/// use orchard::basic_tree::*;
	/// use orchard::example_data::{StdNum, RevAddAction};
	///
	/// let mut tree : BasicTree<StdNum> = (1..=8).collect();
	/// tree.act(RevAddAction {to_reverse : false, add : 5});
	/// # tree.assert_correctness();
	///
	/// assert_eq!(tree.iter().cloned().collect::<Vec<_>>(), (6..=13).collect::<Vec<_>>());
	/// # tree.assert_correctness();
	///```
	pub fn act(&mut self, action : D::Action) {
		self.tree.act(action);
	}

    /// Checks that invariants remain correct. i.e., that every node's summary
	/// is the sum of the summaries of its children.
	/// If it is not, panics.
	pub fn assert_correctness(&self) where
        D::Summary : Eq,
    {
        self.tree.assert_correctness()
    }
}

impl<D : Data + Reverse> Treap<D> {
    /// Reverses the whole tree.
	/// Complexity: O(log n).
	///```
	/// use orchard::basic_tree::*;
	/// use orchard::example_data::StdNum;
	///
	/// let mut tree : BasicTree<StdNum> = (17..=89).collect();
	/// tree.reverse();
	///
	/// assert_eq!(tree.iter().cloned().collect::<Vec<_>>(), (17..=89).rev().collect::<Vec<_>>());
	/// # tree.assert_correctness();
	///```
	pub fn reverse(&mut self) {
		self.tree.reverse();
	}
}

impl<D : Data> std::iter::FromIterator<D::Value> for Treap<D> {
    /// This takes [`O(n)`] worst-case time.
    fn from_iter<T: IntoIterator<Item = D::Value>>(iter: T) -> Self {
        // TODO: write a specific instantiation instead of calling insert,
        // because we know that we're not using all of insert's generality.
        let mut tree = Treap { tree : BasicTree::Empty };
        let mut walker = tree.walker();
        for val in iter {
            walker.insert(val).unwrap();
            // note that it can only go right once
            while let Ok(()) = walker.go_right() {}
        }
        drop(walker);
        tree
    }
}

impl<D : Data> IntoIterator for Treap<D> {
    type Item = D::Value;
    type IntoIter = iterators::OwningIterator<D, fn(D::Summary, &D::Value, D::Summary) -> methods::LocResult, T>;

    fn into_iter(self) -> Self::IntoIter {
        iterators::OwningIterator::new(self.tree, methods::all::<D>)
    }
}

pub struct TreapWalker<'a, D : Data> {
    walker : BasicWalker<'a, D, T>,
}

impl<'a, D : Data> SomeWalker<D> for TreapWalker<'a, D> {
    fn go_left(&mut self) -> Result<(), ()> {
        self.walker.go_left()
    }

    fn go_right(&mut self) -> Result<(), ()> {
        self.walker.go_right()
    }

    fn go_up(&mut self) -> Result<bool, ()> {
        self.walker.go_up()
    }

    fn depth(&self) -> usize {
        self.walker.depth()
    }

    fn far_left_summary(&self) -> D::Summary {
        self.walker.far_left_summary()
    }

    fn far_right_summary(&self) -> D::Summary {
        self.walker.far_left_summary()
    }

    fn value(&self) -> Option<&D::Value> {
        self.walker.value()
    }
}

impl<'a, D : Data> SomeEntry<D> for TreapWalker<'a, D> {
    fn value_mut(&mut self) -> Option<&mut D::Value> {
        self.walker.value_mut()
    }

    fn with_value<F, R>(&mut self, f : F) -> Option<R> where 
        F : FnOnce(&mut D::Value) -> R {
        self.walker.with_value(f)
    }

    fn node_summary(&self) -> D::Summary {
        self.walker.node_summary()
    }

    fn subtree_summary(&self) -> D::Summary {
        self.walker.subtree_summary()
    }

    fn left_subtree_summary(&self) -> Option<D::Summary> {
        self.walker.left_subtree_summary()
    }

    fn right_subtree_summary(&self) -> Option<D::Summary> {
        self.walker.right_subtree_summary()
    }

    fn act_subtree(&mut self, action : D::Action) {
        self.walker.act_subtree(action);
    }

    fn act_node(&mut self, action : D::Action) -> Option<()> {
        self.walker.act_node(action)
    }

    fn act_left_subtree(&mut self, action : D::Action) -> Option<()> {
        self.walker.act_left_subtree(action)
    }

    fn act_right_subtree(&mut self, action : D::Action) -> Option<()> {
        self.walker.act_right_subtree(action)
    }
}

impl<'a, D : Data> TreapWalker<'a, D> {
    
    /// Returns the priority of the current node. Lower numbers means 
    /// The node is closer to the root.
    pub fn priority(&self) -> Option<T> {
        Some(self.walker.node()?.alg_data)
    }

     // TODO: make a trait for splittable trees
    /// Will only do anything if the current position is empty.
    /// If it is empty, it will split the tree: the elements
    /// to the left will remain, and the elements to the right
    /// will be put in the new output tree.
    /// The walker will be at the root after this operation, if it succeeds.
    ///
    ///```
    /// use orchard::treap::*;
    /// use orchard::example_data::StdNum;
    /// use orchard::methods::*; 
    ///
    /// let mut tree : Treap<StdNum> = (17..88).collect();
    /// let mut walker = search_by_locator(&mut tree, &IndexLocator{low : 7, high : 7});
    /// let mut tree2 = walker.split().unwrap();
    /// drop(walker);
    ///
    /// assert_eq!(tree.iter().cloned().collect::<Vec<_>>(), (17..24).collect::<Vec<_>>());
    /// assert_eq!(tree2.iter().cloned().collect::<Vec<_>>(), (24..88).collect::<Vec<_>>());
    /// # tree.assert_correctness();
    ///```
    pub fn split(&mut self) -> Option<Treap<D>> {
        if !self.is_empty() { return None }
        
        let mut temp = BasicTree::Empty;
        // in the first round, this value is irrelevent. choosing this will skip the first if.
        let mut prev_side = self.walker.is_left_son().unwrap_or(false);
        
        while let Ok(side) = self.go_up() {
            if prev_side != side {
                let node = self.walker.node_mut().unwrap();
                let son = if side == true {
                    &mut node.left
                } else {
                    &mut node.right
                };
                std::mem::swap(&mut temp, son);
                node.rebuild();
            }
            prev_side = side;
        }

        if prev_side == true {
            std::mem::swap(&mut *self.walker, &mut temp);
        }
        Some(Treap {tree : temp})
    }
}


impl<'a, D : Data> ModifiableWalker<D> for TreapWalker<'a, D> {
    /// Inserts the value into the tree at the current empty position.
    /// If the current position is not empty, return [`None`].
    /// When the function returns, the walker will be at the position the node
    /// was inserted.
    fn insert(&mut self, val : D::Value) -> Option<()> {
        if !self.is_empty() { return None }

        let priority : T = rand::random();
        let mut temp = BasicTree::Empty;
        // in the first round, this value is irrelevent. choosing this will skip the first if.
        let mut prev_side = self.walker.is_left_son().unwrap_or(false);
        while let Ok(side) = self.go_up() {
            if self.priority().unwrap() > priority {
                // move to the position in which the node should be inserted
                // then break. insertion happens after the break outside the loop.
                if side == true {
                    self.walker.go_left().unwrap();
                } else {
                    self.walker.go_right().unwrap();
                };
                break;
            }
            if self.priority().unwrap() == priority { eprintln!("found equal priorities") }
            if prev_side != side {
                let node = self.walker.node_mut().unwrap();
                let son = if side == true {
                    &mut node.left
                } else {
                    &mut node.right
                };
                std::mem::swap(&mut temp, son);
            }
            self.walker.rebuild();
            prev_side = side;
        }

        // insert the new node, at the current position.
        let mut new : BasicNode<D,_> = BasicNode::new_alg(val, priority);
        
        if prev_side == true {
            new.left = temp;
            new.right = std::mem::replace(&mut *self.walker, BasicTree::Empty);
        } else {
            new.right = temp;
            new.left = std::mem::replace(&mut *self.walker, BasicTree::Empty);
        }
        new.rebuild();
        *self.walker = BasicTree::new(new);
        Some(())
    }

    /// Removes the current value from the tree, and returns it.
    /// If currently at an empty position, returns [`None`].
    fn delete(&mut self) -> Option<D::Value> {
        let tree = std::mem::replace(&mut *self.walker, BasicTree::Empty);
        let boxed_node = match tree {
            BasicTree::Root(boxed_node) => boxed_node,
            BasicTree::Empty => return None,
        };
        let left = Treap { tree : boxed_node.left };
        let right = Treap { tree : boxed_node.right };
        *self.walker = concatenate(left, right).tree;
        Some(boxed_node.node_value)
    }
} 

/// Concatenates the trees together.
///```
/// use orchard::treap::*;
/// use orchard::example_data::StdNum;
///
/// let tree1 : Treap<StdNum> = (17..=89).collect();
/// let tree2 : Treap<StdNum> = (13..=25).collect();
/// let mut tree = concatenate(tree1, tree2);
///
/// assert_eq!(tree.iter().cloned().collect::<Vec<_>>(), (17..=89).chain(13..=25).collect::<Vec<_>>());
/// # tree.assert_correctness();
///```
pub fn concatenate<D : Data>(mut tree1 : Treap<D>, tree2 : Treap<D>) -> Treap<D> {
    let mut walker = tree1.walker();
    let mut tree_r = tree2.tree;
    loop {
        match (walker.priority(), tree_r.alg_data().cloned()) {
            (None, _) => { *walker.walker = tree_r; break },
            (_, None) => break,
            (Some(a), Some(b)) if a < b => {
                walker.go_right().unwrap();
            },
            _ => { 
                std::mem::swap(&mut *walker.walker, &mut tree_r);
                walker.go_left().unwrap();
                std::mem::swap(&mut *walker.walker, &mut tree_r);
            },
        }
    }
    drop(walker); // the walker is responsible for going up the tree
    // and rebuilding all the nodes
    tree1
}