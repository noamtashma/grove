//! The basic tree module.
//! This module implements basic unbalanced trees.
//!
//! This module is meant to provide an inner tree type that other tree implementations
//! can wrap around. Therefore, it exposes more of its inner workings than the other trees,
//! and the walker can't rebalance the tree when modified.
//!
//! Also,  The type parameter `T` is supposed to be used to store the balancing algorithm's
//! bookeeping data (ranks, sizes and so on). Therefore, most of the functionality is
//! implemented for general `T`, even though by default `T = ()`.

// these two should not be public as they are merely separate files
// for some of the functions of this module

mod imm_down_walker;
pub(crate) use imm_down_walker::ImmDownBasicWalker;

mod walker;
pub use walker::*;

mod implementations;
pub use implementations::*;

/// Iterators for [`BasicTree`]
pub mod iterators;

mod iterative_deallocator;
pub use iterative_deallocator::deallocate_iteratively;

use crate::*;

/// A basic tree. might be empty.
/// The `T` parameter is for algorithm-specific bookeeping data.
/// For example, red-black trees store a color in each node.
pub enum BasicTree<D: ?Sized + Data, T = ()> {
    /// An empty tree
    Empty,
    /// A non empty tree, with a root node
    Root(Box<BasicNode<D, T>>), // TODO: rename Root
}
use BasicTree::*;

impl<D: Data, T> BasicTreeTrait<D, T> for BasicTree<D, T> {
    type Node = BasicNode<D, T>;

    /// Creates an empty tree
    fn new() -> Self {
        Empty
    }

    /// Returns the action that is currently stored at the root.
    /// This action is to be applied to all of the tree's values.
    /// Returns `default()` if the tree is empty, and the node's action otherwise.
    fn action(&self) -> D::Action {
        match self.node() {
            Some(node) => node.action,
            None => Default::default(),
        }
    }

    /// Constructs a new non-empty tree from a node.
    fn from_node(node: BasicNode<D, T>) -> Self {
        Root(Box::new(node))
    }

    

    /// Returns the algorithm-specific data
    fn alg_data(&self) -> Option<&T> {
        Some(self.node()?.alg_data())
    }

    /// Remakes the summary that is stored in this node, based on its sons.
    /// This is necessary when the sons might have changed.
    /// For example, after inserting a new node, all of the nodes from it to the root
    /// must be rebuilt, in order for the summaries accumulated over the whole
    /// subtree to be accurate.
    fn rebuild(&mut self) {
        if let Root(node) = self {
            node.rebuild()
        }
    }

    /// Pushes any actions stored in this node to its sons.
    /// Actions stored in nodes are supposed to be eventually applied to its
    /// whole subtree. Therefore, in order to access a node cleanly, without
    /// the still-unapplied-function complicating things, you must `access()` the node.
    fn access(&mut self) {
        if let Root(node) = self {
            node.access()
        }
    }

    /// Returns The inner node.
    fn node(&self) -> Option<&BasicNode<D, T>> {
        match self {
            Empty => None,
            Root(node) => Some(node),
        }
    }

    /// Returns The inner node.
    fn node_mut(&mut self) -> Option<&mut BasicNode<D, T>> {
        match self {
            Empty => None,
            Root(node) => Some(node),
        }
    }


    /// Returns The inner node.
    fn into_node(self) -> Option<BasicNode<D, T>> {
        match self {
            Empty => None,
            Root(node) => Some(*node),
        }
    }


    /// Checks that invariants remain correct. Invariants are checked by running
    /// the given function on every node in the current subtree.
    ///
    /// The function should perfoem checks (e.g, check that the current node's summary is indeed
    /// the sum of its children's summaries) and panic if they're violated.
    fn assert_correctness_with<F>(&self, func: F)
    where
        F: Fn(&BasicNode<D, T>) + Copy,
    {
        if let Some(node) = self.node() {
            func(node);
            node.left.assert_correctness_with(func);
            node.right.assert_correctness_with(func);
        }
    }
}

impl<D: Data, T> BasicTree<D, T> {
    /// Constructs a new non-empty tree from a boxed node.
    pub fn from_boxed_node(boxed: Box<BasicNode<D, T>>) -> Self {
        Root(boxed)
    }

    /// Returns The inner node with its box. This is exposed in order
    /// to allow easier coding while preventing from moving values of `BasicNode`,
    /// because `BasicNode` is bigger than a single pointer.
    pub fn into_node_boxed(self) -> Option<Box<BasicNode<D, T>>> {
        match self {
            Empty => None,
            Root(node) => Some(node),
        }
    }

    /// Returns The inner node with its box. This is exposed in order
    /// to allow easier coding while preventing from moving values of `BasicNode`,
    /// because `BasicNode` is bigger than a single pointer.
    pub fn node_boxed(&mut self) -> Option<&mut Box<BasicNode<D, T>>> {
        match self {
            Empty => None,
            Root(node) => Some(node),
        }
    }
}

// TODO: try to move the fields from pub(crate) to private
/// A basic node. can be viewed as a non-empty basic tree: it always has at least one value.
/// The `T` parameter is for algorithm-specific bookeeping data.
/// For example, red-black trees store a color in each node.
pub struct BasicNode<D: ?Sized + Data, T = ()> {
    action: D::Action,
    subtree_summary: D::Summary,
    pub(crate) node_value: D::Value,
    pub(crate) left: BasicTree<D, T>,
    pub(crate) right: BasicTree<D, T>,
    pub(crate) alg_data: T,
}

impl<D: Data> BasicNode<D> {
    /// Creates a node with a single value.
    pub fn new(value: D::Value) -> BasicNode<D> {
        let subtree_summary = value.to_summary();
        BasicNode {
            action: Default::default(),
            node_value: value,
            subtree_summary,
            left: Empty,
            right: Empty,
            alg_data: (),
        }
    }
}

impl<D: Data, T> BasicNodeTrait<D, T> for BasicNode<D, T> {
    /// Creates a node with a single value, and the algorithm specific data.
    fn new_alg(value: D::Value, alg_data: T) -> BasicNode<D, T> {
        let subtree_summary = value.to_summary();
        BasicNode {
            action: Default::default(),
            node_value: value,
            subtree_summary,
            left: Empty,
            right: Empty,
            alg_data,
        }
    }

    /// Returns the algorithm-specific data
    fn alg_data(&self) -> &T {
        &self.alg_data
    }

    fn action(&self) -> &D::Action {
        &self.action
    }

    /// Returns the summary of all values in this node's subtree.
    /// Same as [`BasicTree::subtree_summary`].
    fn subtree_summary(&self) -> D::Summary {
        self.action.act(self.subtree_summary)
    }

    /// Returns a summary for the value in this node specifically,
    /// and not the subtree.
    fn node_summary(&self) -> D::Summary {
        let summary = self.node_value.to_summary();
        self.action.act(summary)
    }

    /// Returns a reference to the value stored in this node specifically.
    /// Requires mutable access because it calls `BasicNode::access`, to ensure
    /// that the action applies.
    fn node_value(&mut self) -> &D::Value {
        self.access();
        &self.node_value
    }

    /// Returns a mutable reference to the value stored in this node specifically.
    fn node_value_mut(&mut self) -> &mut D::Value {
        self.access();
        &mut self.node_value
    }

    /// Returns the value stored in this node specifically.
    /// Assumes that the node has been accessed. Panics otherwise.
    fn node_value_clean(&self) -> &D::Value {
        assert!(self.action.is_identity());
        &self.node_value
    }

    /// Pushes any actions stored in this node to its sons.
    /// Actions stored in nodes are supposed to be eventually applied to its
    /// whole subtree. Therefore, in order to access a node cleanly, without
    /// the still-unapplied-function complicating things, you must `access()` the node.
    fn access(&mut self) {
        // reversing
        // for data that doesn't implement reversing, this becomes a no-op
        // and hopefully optimized away
        if self.action.to_reverse() {
            std::mem::swap(&mut self.left, &mut self.right);
        }

        self.left.act_subtree(self.action);
        self.right.act_subtree(self.action);
        self.action.act_inplace(&mut self.subtree_summary);
        self.action.act_inplace(&mut self.node_value);
        self.action = Default::default();
    }

    /// Remakes the data that is stored in this node, based on its sons.
    /// This is necessary when the data in the sons might have changed.
    /// For example, after inserting a new node, all of the nodes from it to the root
    /// must be rebuilt, in order for the segment values accumulated over the whole
    /// subtree to be accurate.
    fn rebuild(&mut self) {
        assert!(self.action.is_identity());
        let temp = self.node_value.to_summary();
        self.subtree_summary = self.left.subtree_summary() + temp + self.right.subtree_summary();
    }

    /// This function applies the given action to its whole subtree.
    /// Same as [`SomeEntry::act_subtree`], but for [`BasicNode<D>`].
    ///
    /// This function leaves the [`self.action`] field "dirty" - after calling
    /// this you might need to call `access`, to push the action to this node's sons.
    ///```
    /// use grove::{*, basic_tree::*};
    /// use grove::example_data::{StdNum, RevAffineAction};
    ///
    /// let mut tree: BasicTree<StdNum> = (1..=8).collect();
    /// let node: &mut BasicNode<StdNum> = tree.node_mut().unwrap();
    /// node.act(RevAffineAction {to_reverse: false, mul: -1, add: 5});
    /// # tree.assert_correctness();
    ///
    /// assert_eq!(tree.iter().cloned().collect::<Vec<_>>(), (-3..=4).rev().collect::<Vec<_>>());
    /// # tree.assert_correctness();
    ///```
    fn act(&mut self, action: D::Action) {
        self.action = action + self.action;
    }

    /// This function applies the given action only to the current value in this node.
    /// Same as [`SomeEntry::act_node`].
    fn act_value(&mut self, action: D::Action) {
        self.access();
        action.act_inplace(&mut self.node_value);
    }

    #[cfg(debug_assertions)]
    /// Used for debugging. Prints a representation of the tree, like so:
    /// `< < * * > * >`
    /// Each pair of triangle brackets is a node, and `*` denotes empty trees.
    /// The trees are printed in the layout they will have atfter all reversals have been
    /// finished, but nodes which are yet to be reversed (`node.action.to_reverse() == true`)
    /// are printed with an exclamation mark: `<! * * >`.
    /// You can provide a custom printer for the alg_data field.
    /// If the input `to_reverse` is true, it will print the tree in reverse.
    fn representation<F>(&self, alg_print: &F, to_reverse: bool) -> String
    where
        F: Fn(&Self) -> String,
    {
        let xor = self.action().to_reverse() ^ to_reverse;
        let shebang = if self.action().to_reverse() { "!" } else { "" };
        let mut left = self.left.representation(alg_print, xor);
        let mut right = self.right.representation(alg_print, xor);
        if xor {
            std::mem::swap(&mut left, &mut right);
        }

        format!("{} {} {} {}", shebang, alg_print(self), left, right)
    }

    /// Asserts that the summaries were calculated correctly at the current node.
    /// Otherwise, panics.
    fn assert_correctness_locally(&self)
    where
        D::Summary: Eq,
    {
        let ns = self.subtree_summary;
        let os: D::Summary = self.left.subtree_summary()
            + self.node_value.to_summary()
            + self.right.subtree_summary();
        assert!(ns == os, "Incorrect summaries found.");
    }
}


impl<D: Data, T> BasicNode<D, T> {
}