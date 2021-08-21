#![allow(missing_docs)]

//! The basic tree trait.
//!
//! This trait represents unbalanced trees implemented in different ways.
//! They specify the implementation details in how the tree itself is built.
//! For example, whether your tree has parent pointers, whether it has next-neighbor pointers,
//! whether it is implemented using safe rust, and so on.
//!
//! Then different balancing algorithms (Treap, AVL, ...) can be built generically on
//! top of these basic trees.

use crate::Data;
use super::*;

pub trait BasicTreeTrait<D: Data, T>: SomeEntry<D> + Default where
    for<'a> &'a mut Self: SomeTreeRef<D>,
{
    type Node: BasicNodeTrait<D, T>;
    fn new() -> Self {
        Default::default()
    }

    /// Returns the action that is (locally) going to be applied to all of
    /// this tree's nodes.
    /// Returns `default()` if the tree is empty, and `self.node().action` otherwise
    fn action(&self) -> D::Action;

    /// Constructs a new non-empty tree from a node.
    fn from_node(node: Self::Node) -> Self;

    /// Returns the algorithm-specific data
    fn alg_data(&self) -> Option<&T>;

    /// Remakes the data that is stored in this node, based on its sons.
    /// This is necessary when the data in the sons might have changed.
    /// For example, after inserting a new node, all of the nodes from it to the root
    /// must be rebuilt, in order for the segment values accumulated over the whole
    /// subtree to be accurate.
    fn rebuild(&mut self) {
        if let Some(node) = self.node_mut() {
            node.rebuild()
        }
    }

    /// Pushes any actions stored in this node to its sons.
    /// Actions stored in nodes are supposed to be eventually applied to its
    /// whole subtree. Therefore, in order to access a node cleanly, without
    /// the still-unapplied-function complicating things, you must `access()` the node.
    fn access(&mut self) {
        if let Some(node) = self.node_mut() {
            node.access()
        }
    }

    /// Returns The inner node.
    fn node(&self) -> Option<&Self::Node>;

    /// Returns The inner node.
    fn node_mut(&mut self) -> Option<&mut Self::Node>;

    /// Returns The inner node.
    fn into_node(self) -> Option<Self::Node>;

    /// Checks that invariants remain correct. i.e., that every node's summary
    /// is the sum of the summaries of its children.
    /// If it is not, panics.
    fn assert_correctness_with<F>(&self, func: F)
    where
        F: Fn(&Self::Node) + Copy;
}

pub trait BasicNodeTrait<D: Data, T> {
    /// Creates a node with a single value, and the algorithm specific data.
    fn new_alg(value: D::Value, alg_data: T) -> Self;

    /// Returns the algorithm-specific data
    fn alg_data(&self) -> &T;

    fn action(&self) -> &D::Action;

    /// Returns the summary of all values in this node's subtree.
    /// Same as [`BasicTree::subtree_summary`].
    fn subtree_summary(&self) -> D::Summary;

    /// Returns a summary for the value in this node specifically,
    /// and not the subtree.
    fn node_summary(&self) -> D::Summary;

    /// Returns a reference to the value stored in this node specifically.
    /// Requires mutable access because it calls `BasicNode::access`, to ensure
    /// that the action applies.
    fn node_value(&mut self) -> &D::Value;

    /// Returns a mutable reference to the value stored in this node specifically.
    fn node_value_mut(&mut self) -> &mut D::Value;

    /// Returns the value stored in this node specifically.
    /// Assumes that the node has been accessed. Should panic otherwise.
    fn node_value_clean(&self) -> &D::Value;

    /// Pushes any actions stored in this node to its sons.
    /// Actions stored in nodes are supposed to be eventually applied to its
    /// whole subtree. Therefore, in order to access a node cleanly, without
    /// the still-unapplied-function complicating things, you must `access()` the node.
    fn access(&mut self);

    /// Remakes the data that is stored in this node, based on its sons.
    /// This is necessary when the data in the sons might have changed.
    /// For example, after inserting a new node, all of the nodes from it to the root
    /// must be rebuilt, in order for the segment values accumulated over the whole
    /// subtree to be accurate.
    fn rebuild(&mut self);

    /// This function applies the given action to its whole subtree.
    /// Same as [`SomeEntry::act_subtree`], but for [`BasicNode<D>`].
    ///
    /// This function leaves the [`self.action`] field "dirty" - after calling
    /// this you might need to call access, to push the action to this node's sons.
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
    fn act(&mut self, action: D::Action);

    /// This function applies the given action only to the current value in this node.
    /// Same as [`SomeEntry::act_node`].
    fn act_value(&mut self, action: D::Action);

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
        F: Fn(&Self) -> String;

    /// Asserts that the summaries were calculated correctly at the current node.
    /// Otherwise, panics.
    fn assert_correctness_locally(&self)
    where
        D::Summary: Eq;
}