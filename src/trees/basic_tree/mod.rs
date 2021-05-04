//! The basic tree module
//! This module implements basic unbalanced trees.

// these two should not be public as they are merely separate files
// for some of the functions of this module
mod walker;
pub use walker::*;

mod implementations;
pub use implementations::*;

pub mod iterators;

use crate::*;
//pub use crate::data::*; // because everyone will need to specify Data for the generic parameters

/// A basic tree. might be empty.
/// The `T` parameter is for algorithm-specific bookeeping data.
/// For example, red-block trees store a color in each node.
pub enum BasicTree<A : ?Sized + Data, T=()> {
	Empty, Root(Box<BasicNode<A, T>>) // TODO: rename Root
}
use BasicTree::*;

impl<D : Data, T> BasicTree<D, T> {
	/// Constructs a new non-empty tree from a node.
	pub fn new(node : BasicNode<D, T>) -> BasicTree<D, T> {
		Root(Box::new(node))
	}

	/// Returns the algorithm-specific data
	pub fn alg_data(&self) -> Option<&T> {
		Some(&self.node()?.alg_data)
	}

	/// Remakes the data that is stored in this node, based on its sons.
	/// This is necessary when the data in the sons might have changed.
	/// For example, after inserting a new node, all of the nodes from it to the root
	/// must be rebuilt, in order for the segment values accumulated over the whole
	/// subtree to be accurate.
	pub(crate) fn rebuild(&mut self) {
		match self {
			Root(node) => node.rebuild(),
			_ => (),
		}
	}
	
	/// Pushes any actions stored in this node to its sons.
	/// Actions stored in nodes are supposed to be eventually applied to its
	/// whole subtree. Therefore, in order to access a node cleanly, without
	/// the still-unapplied-function complicating things, you must `access()` the node.
	pub(crate) fn access(&mut self) {
		match self {
			Root(node) => node.access(),
			_ => (),
		}
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
		iterators::ImmIterator::new(self, methods::locator::all::<D>)
	}

	/// Iterates over the given segment.
	///```
	/// use orchard::basic_tree::*;
	/// use orchard::example_data::StdNum;
	/// use orchard::methods;
	///
	/// let mut tree : BasicTree<StdNum> = (20..80).collect();
	/// let segment_iter = tree.iter_locator(methods::locate_by_index_range(3,13)); // should also try 3..5
	///
	/// assert_eq!(segment_iter.cloned().collect::<Vec<_>>(), (23..33).collect::<Vec<_>>());
	/// # tree.assert_correctness();
	///```
	pub fn iter_locator<L>(&mut self, loc : L) -> impl Iterator<Item=&D::Value> where
		L : methods::Locator<D>
	{
		iterators::ImmIterator::new(self, loc)
	}

	/// Checks that invariants remain correct. i.e., that every node's summary
	/// is the sum of the summaries of its children.
	/// If it is not, panics.
	pub fn assert_correctness(&self) where
		D::Summary : Eq,
	{
		if let Root(node) = self {
			let ns = node.subtree_summary;
			let os : D::Summary = node.left.subtree_summary() + D::to_summary(&node.node_value) + node.right.subtree_summary();
			assert!(ns == os);
				
			node.left.assert_correctness();
			node.right.assert_correctness();
		}
	}

	pub fn node(&self) -> Option<&BasicNode<D, T>> {
		match self {
			Empty => None,
			Root(node) => Some(node),
		}
	}

	pub fn node_mut(&mut self) -> Option<&mut BasicNode<D, T>> {
		match self {
			Empty => None,
			Root(node) => Some(node),
		}
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
		if let Root(node) = self {
			node.act(action);
		}
	}
}

// TODO: decide if the fields should really be public
/// A basic node. can be viewed as a non-empty basic tree: it always has at least one value.
/// The `T` parameter is for algorithm-specific bookeeping data.
/// For example, red-block trees store a color in each node.
pub struct BasicNode<D : ?Sized + Data, T=()> {
	action : D::Action,
	subtree_summary : D::Summary,
	pub (crate) node_value : D::Value,
	pub (crate)  left : BasicTree<D,T>,
	pub (crate) right : BasicTree<D,T>,
	pub (crate) alg_data : T,
}

impl<D : Data> BasicNode<D> {
	/// Creates a node with a single value.
	pub fn new(value : D::Value) -> BasicNode<D> {
		let subtree_summary = D::to_summary(&value);
		BasicNode {
			action : D::IDENTITY,
			node_value : value,
			subtree_summary,
			left : Empty,
			right : Empty,
			alg_data : (),
		}
	}
}
	
impl<D : Data, T> BasicNode<D, T> {
	/// Creates a node with a single value, and the algorithm specific data.
	pub fn new_alg(value : D::Value, alg_data : T) -> BasicNode<D, T> {
		let subtree_summary = D::to_summary(&value);
		BasicNode {
			action : D::IDENTITY,
			node_value : value,
			subtree_summary,
			left : Empty,
			right : Empty,
			alg_data,
		}
	}
	/// Returns the summary of all values in this node's subtree.
	/// Same as [`BasicTree::subtree_summary`].
	pub fn subtree_summary(&self) -> D::Summary {
		return D::act_summary(self.action, self.subtree_summary);
	}

	/// Returns a summary for the value in this node specifically,
	/// and not the subtree.
	pub fn node_summary(&self) -> D::Summary {
		let summary = D::to_summary(&self.node_value);
		D::act_summary(self.action, summary)
	}

	/// Returns a reference to the value stored in this node specifically.
	/// Requires mutable access because it calls [`BasicNode::access`], to ensure
	/// that the action applies.
	pub fn node_value(&mut self) -> &D::Value {
		self.access();
		&self.node_value
	}

	/// Returns a mutable reference to the value stored in this node specifically.
	pub fn node_value_mut(&mut self) -> &mut D::Value {
		self.access();
		&mut self.node_value
	}

	/// Returns the value stored in this node specifically.
	/// Assumes that the node has been accessed. Panics otherwise.
	pub(crate) fn node_value_clean(&self) -> &D::Value {
		assert!(self.action == D::IDENTITY);
		&self.node_value
	}

	/// Pushes any actions stored in this node to its sons.
	/// Actions stored in nodes are supposed to be eventually applied to its
	/// whole subtree. Therefore, in order to access a node cleanly, without
	/// the still-unapplied-function complicating things, you must `access()` the node.
	pub(crate) fn access(&mut self) {
		// reversing
		// for data that doesn't implement reversing, this becomes a no-op
		// and hopefully optimized away
		if D::to_reverse(self.action) {
			std::mem::swap(&mut self.left, &mut self.right);
		}

		if let Root(node) = &mut self.left {
			node.act(self.action);
		}
		if let Root(node) = &mut self.right {
			node.act(self.action);
		}
		self.subtree_summary = D::act_summary(self.action, self.subtree_summary);
		D::act_value(self.action, &mut self.node_value);
		self.action = D::IDENTITY;
	}

	/// Remakes the data that is stored in this node, based on its sons.
	/// This is necessary when the data in the sons might have changed.
	/// For example, after inserting a new node, all of the nodes from it to the root
	/// must be rebuilt, in order for the segment values accumulated over the whole
	/// subtree to be accurate.
	pub(crate) fn rebuild(&mut self) {
		assert!(self.action == D::IDENTITY);
		let temp = D::to_summary(&self.node_value);
		self.subtree_summary = self.left.subtree_summary() + temp + self.right.subtree_summary();
	}

	/// This function applies the given action to its whole subtree.
	/// Same as [`BasicTree::act`].
	pub fn act(&mut self, action : D::Action) {
		self.action = action + self.action;
	}

	/// This function applies the given action only to the current value in this node.
	/// Same as [`SomeEntry::act_node`].
	pub fn act_value(&mut self, action : D::Action) {
		self.access();
		D::act_value(action, &mut self.node_value);
	}
}