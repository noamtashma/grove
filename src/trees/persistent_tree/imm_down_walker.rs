use super::PersistentTree;
use crate::*;

/// A BasicWalker version that is immutable, and can only go down.
#[derive(Copy)]
pub struct ImmDownBasicWalker<'a, D: Data, T = ()> {
    tree: &'a PersistentTree<D, T>,

    // to be applied to everything in `tree`.
    // already contains this node's actions.
    current_action: D::Action,

    // note: these should always have `current_action` already applied to them,
    // and in the already reversed order if `current_action.to_reverse() == true`.
    // the summary of everything to the left of the current subtree
    far_left_summary: D::Summary,
    // the summary of everything to the right of the current subtree
    far_right_summary: D::Summary,
}

/// This is needed because the automatic implementation also requires
/// `D: Clone` and `T: Clone`.
impl<'a, D: Data, T> Clone for ImmDownBasicWalker<'a, D, T> {
    fn clone(&self) -> Self {
        ImmDownBasicWalker { ..*self }
    }
}

impl<'a, D: Data, T> ImmDownBasicWalker<'a, D, T> {
    pub fn new(tree: &'a PersistentTree<D, T>) -> Self {
        ImmDownBasicWalker {
            tree,
            current_action: tree.action(),
            far_left_summary: Default::default(),
            far_right_summary: Default::default(),
        }
    }

    /// Goes to the left son.
    /// If at an empty position, returns [`None`].
    pub fn go_left(&mut self) -> Option<()> {
        self.go_left_extra()?;
        Some(())
    }

    /// Goes to the left son.
    /// If at an empty position, returns [`None`].
    /// Otherwise, also returns the summary of the current node
    /// with its right subtree.
    pub fn go_left_extra(&mut self) -> Option<D::Summary> {
        let node = self.tree.node()?;

        // deal with reversals
        let mut right = &node.right;
        let mut left = &node.left;
        if self.current_action.to_reverse() {
            std::mem::swap(&mut left, &mut right);
        }

        let extra = self.current_action.act(node.node_value.to_summary())
            + self.current_action.act(right.subtree_summary());
        self.far_right_summary = extra + self.far_right_summary;
        self.tree = left;
        self.current_action = self.current_action + left.action();
        Some(extra)
    }

    /// Goes to the right son.
    /// If at an empty position, returns [`None`].
    pub fn go_right(&mut self) -> Option<()> {
        self.go_right_extra()?;
        Some(())
    }

    /// Goes to the right son.
    /// If at an empty position, returns [`None`].
    /// Otherwise, also returns the summary of the current node
    /// with its left subtree.
    pub fn go_right_extra(&mut self) -> Option<D::Summary> {
        let node = self.tree.node()?;

        // deal with reversals
        let mut right = &node.right;
        let mut left = &node.left;
        if self.current_action.to_reverse() {
            std::mem::swap(&mut left, &mut right);
        }

        let extra = self.current_action.act(left.subtree_summary())
            + self.current_action.act(node.node_value.to_summary());
        self.far_left_summary = self.far_left_summary + extra;
        self.tree = right;
        self.current_action = self.current_action + right.action();
        Some(extra)
    }

    /// Returns the value at the current node.
    pub fn value(&self) -> Option<D::Value>
    where
        D::Value: Clone,
    {
        Some(
            self.current_action
                .act(self.tree.node()?.node_value.clone()),
        )
    }

    /// Returns the summary of just this node.
    pub fn node_summary(&self) -> Option<D::Summary> {
        Some(
            self.current_action
                .act(self.tree.node()?.node_value.to_summary()),
        )
    }

    pub fn left_summary(&self) -> D::Summary {
        if let Some(node) = self.tree.node() {
            let left = if self.current_action.to_reverse() {
                &node.right
            } else {
                &node.left
            };
            self.far_left_summary + self.current_action.act(left.subtree_summary())
        } else {
            self.far_left_summary
        }
    }

    pub fn right_summary(&self) -> D::Summary {
        if let Some(node) = self.tree.node() {
            let right = if self.current_action.to_reverse() {
                &node.left
            } else {
                &node.right
            };
            self.current_action.act(right.subtree_summary()) + self.far_right_summary
        } else {
            self.far_right_summary
        }
    }

    pub fn query_locator<L: Locator<D>>(&self, locator: &L) -> Option<locators::LocResult>
    where
        D::Value: Clone,
    {
        let node = self.tree.node()?;

        // deal with reversals
        let mut right = &node.right;
        let mut left = &node.left;
        if self.current_action.to_reverse() {
            std::mem::swap(&mut left, &mut right);
        }

        let direction = locator.locate(
            self.left_summary(),
            &self.value().expect("suddenly empty error"),
            self.right_summary(),
        );

        Some(direction)
    }
}
