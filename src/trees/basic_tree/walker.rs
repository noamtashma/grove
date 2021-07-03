// This is a private module, so no documentation for it directly.
// instead look for documentation of the `BasicWalker` struct.

use super::*;
use recursive_reference::*;

use crate::trees::SomeWalker; // in order to be able to use our own go_up method

pub(super) struct Frame<D: ?Sized + Data> {
    pub left: D::Summary,
    pub right: D::Summary,
}

// the default clone implementation requires that A: Clone, which is uneccessary
impl<D: ?Sized + Data> Clone for Frame<D>
where
    D::Summary: Clone,
{
    fn clone(&self) -> Self {
        Frame {
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}

impl<D: Data> Frame<D> {
    pub fn empty() -> Frame<D> {
        Frame {
            left: Default::default(),
            right: Default::default(),
        }
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
/// Walkers for other kinds of trees may be built by wrapping around the [`BasicWalker`] type,
/// as tree types can be built by wrapping around the [`BasicTree`] type.
///
/// The walker will automatically go back up the tree to the root when dropped,
/// in order to rebuild all the nodes.
///
/// Internally, [`recursive_reference::RecRef`] is used, in order to be able to dynamically
/// go up and down the tree without upsetting the borrow checker.
#[derive(destructure)]
pub struct BasicWalker<'a, D: Data, T = ()> {
    /// The telescope, holding references to all the subtrees from the root to the
    /// current position.
    pub(super) rec_ref: RecRef<'a, BasicTree<D, T>>,

    /// This array holds the accumulation of all the values left of the subtree, and
    /// all of the values right of the subtree, for every subtree from the root to
    /// the current subtree.
    pub(super) vals: Vec<Frame<D>>,

    /// This array holds for every node, whether the next subtree in the walker
    /// is its left son or the right son. (true corresponds to the left son).
    /// This array is always one shorter than [`BasicWalker::rec_ref`] and [`BasicWalker::vals`],
    /// because the last node has no son in the walker.
    pub(super) is_left: Vec<Side>,
}

impl<'a, D: Data, T> BasicWalker<'a, D, T> {
    pub fn new(tree: &'a mut BasicTree<D, T>) -> BasicWalker<'a, D, T> {
        tree.access();
        BasicWalker {
            rec_ref: RecRef::new(tree),
            vals: vec![Frame::empty()],
            is_left: vec![],
        }
    }

    /// Returns a new walker at the root of the tree, but treats it as if it started in the
    /// of a larger tree, where the summaries to the left and right are
    /// `left_summary` and `right_summary`.
    pub fn new_with_context(
        tree: &'a mut BasicTree<D, T>,
        left_summary: D::Summary,
        right_summary: D::Summary,
    ) -> BasicWalker<'a, D, T> {
        tree.access();
        BasicWalker {
            rec_ref: RecRef::new(tree),
            vals: vec![Frame {
                left: left_summary,
                right: right_summary,
            }],
            is_left: vec![],
        }
    }

    /// Returns true if at an empty position.
    pub fn is_empty(&self) -> bool {
        self.rec_ref.is_empty()
    }

    /// Returns if this is the empty tree
    /// Note: even if you are the root, the root might still be empty.
    pub fn is_root(&self) -> bool {
        self.is_left.is_empty()
    }

    /// If the current position is the left son of a node, returns [`Some(Left)`].
    /// If the current position is the right son of a node, returns [`Some(Right)`].
    /// If at the root, returns [`None`].
    pub fn is_left_son(&self) -> Option<Side> {
        self.is_left.last().cloned()
    }

    /*
    /// Not public since the walker should maintain the invariant that the current position
    /// is always clean. Ergo, for internal use.
    pub (in super::super) fn access(&mut self) {
        self.rec_ref.access();
    }
    */

    /// Not public since the walker should maintain the invariant that the current position
    /// is always clean. Ergo, for internal use.
    pub(in super::super) fn rebuild(&mut self) {
        self.rec_ref.rebuild();
    }

    pub fn inner(&self) -> &BasicTree<D, T> {
        &*self.rec_ref
    }

    pub(in super::super) fn inner_mut(&mut self) -> &mut BasicTree<D, T> {
        &mut *self.rec_ref
    }

    pub fn node(&self) -> Option<&BasicNode<D, T>> {
        self.rec_ref.node()
    }

    pub(in super::super) fn node_mut(&mut self) -> Option<&mut BasicNode<D, T>> {
        self.rec_ref.node_mut()
    }

    /// Performs a left rotation
    /// Returns [`None`] if this is an empty tree or if it has no right son.
    pub fn rot_left(&mut self) -> Option<()> {
        self.rot_left_with_custom_rebuilder(|_| {})
    }

    /// Performs a left rotation.
    /// Returns [`None`] if this is an empty tree or if it has no right son.
    /// Uses a callback for a rebuilding action, that will be applied in addition
    /// to the regular summary rebuilding
    pub fn rot_left_with_custom_rebuilder<F: FnMut(&mut BasicNode<D, T>)>(
        &mut self,
        mut rebuilder: F,
    ) -> Option<()> {
        let owned_tree = std::mem::replace(&mut *self.rec_ref, BasicTree::Empty);

        let mut bn1: Box<BasicNode<D, T>> = owned_tree.into_node_boxed()?;
        assert!(bn1.action.is_identity());

        let mut bn2: Box<BasicNode<D, T>> = bn1.right.into_node_boxed()?;
        bn2.access();

        bn1.right = bn2.left;
        bn2.subtree_summary = bn1.subtree_summary; // this is insetad of bn2.rebuild(), since we already know the result
        bn1.rebuild();
        rebuilder(&mut *bn1);
        bn2.left = Root(bn1);
        // bn2.rebuild()
        rebuilder(&mut *bn2);

        *self.rec_ref = Root(bn2); // restore the node back
        Some(())
    }

    /// Performs a right rotation
    /// Returns [`None`] if this node has no left son.
    pub fn rot_right(&mut self) -> Option<()> {
        self.rot_right_with_custom_rebuilder(|_| {})
    }

    /// Performs a right rotation.
    /// Returns [`None`] if this node has no left son.
    /// Uses a callback for a rebuilding action, that will be applied in addition
    /// to the regular summary rebuilding
    pub fn rot_right_with_custom_rebuilder<F: FnMut(&mut BasicNode<D, T>)>(
        &mut self,
        mut rebuilder: F,
    ) -> Option<()> {
        let owned_tree = std::mem::replace(&mut *self.rec_ref, BasicTree::Empty);

        let mut bn1: Box<BasicNode<D, T>> = owned_tree.into_node_boxed()?;
        assert!(bn1.action.is_identity());

        let mut bn2: Box<BasicNode<D, T>> = bn1.left.into_node_boxed()?;
        bn2.access();

        bn1.left = bn2.right;
        bn2.subtree_summary = bn1.subtree_summary; // this is insetad of bn2.rebuild(), since we already know the result
        bn1.rebuild();
        rebuilder(&mut *bn1);
        bn2.right = Root(bn1);
        // bn2.rebuild()
        rebuilder(&mut *bn2);

        *self.rec_ref = Root(bn2); // restore the node back
        Some(())
    }

    /// Performs rot_left if `side` is [`Side::Left`]
    /// rot_right otherwise
    pub fn rot_side(&mut self, side: Side) -> Option<()> {
        match side {
            Side::Left => self.rot_left(),
            Side::Right => self.rot_right(),
        }
    }

    /// Performs rot_left if b is true
    /// rot_right otherwise
    pub fn rot_side_with_custom_rebuilder<F: FnMut(&mut BasicNode<D, T>)>(
        &mut self,
        side: Side,
        rebuilder: F,
    ) -> Option<()> {
        match side {
            Side::Left => self.rot_left_with_custom_rebuilder(rebuilder),
            Side::Right => self.rot_right_with_custom_rebuilder(rebuilder),
        }
    }

    /// Rotates so that the current node moves up.
    /// Basically moves up and then calls rot_side.
    /// Fails if the current node is the root.
    pub fn rot_up(&mut self) -> Result<Side, ()> {
        let b = self.go_up()?;
        self.rot_side(b.flip())
            .expect("original node went missing?");
        Ok(b)
    }

    /// Rotates so that the current node moves up.
    /// Basically moves up and then calls rot_side.
    /// Fails if the current node is the root.
    pub fn rot_up_with_custom_rebuilder<F: FnMut(&mut BasicNode<D, T>)>(
        &mut self,
        rebuilder: F,
    ) -> Result<Side, ()> {
        let b = self.go_up()?;
        self.rot_side_with_custom_rebuilder::<F>(b.flip(), rebuilder)
            .expect("original node went missing?");
        Ok(b)
    }

    pub fn go_to_root(&mut self) {
        while let Ok(_) = self.go_up() {}
    }

    /// This takes the walker and turns it into a reference to the root
    pub fn root_into_ref(mut self) -> &'a mut BasicTree<D, T> {
        // go to the root
        self.go_to_root();
        let (tel, _, _) = self.destructure();
        RecRef::into_ref(tel)
    }

    /// Creates a walker that can only access the current subtree. However,
    /// it knows the context of the tree around it, so that locators still work on it as expected
    /// (e.g, looking for the seventh element will still find the element that is the seventh in
    /// the whole tree, and not the seventh in the subtree).
    pub fn detached_walker(&mut self) -> BasicWalker<D, T> {
        let left = self.far_left_summary();
        let right = self.far_right_summary();
        BasicWalker::new_with_context(self.inner_mut(), left, right)
    }

    pub fn insert_with_alg_data(&mut self, value: D::Value, alg_data: T) -> Option<()> {
        match *self.rec_ref {
            Empty => {
                *self.rec_ref = BasicTree::from_node(BasicNode::new_alg(value, alg_data));
                Some(())
            }
            _ => None,
        }
    }

    /// Takes the current subtree out of the tree, and writes `Empty` instead.
    /// Intended to help writing tree algorithms.
    pub(in super::super) fn take_subtree(&mut self) -> BasicTree<D, T> {
        std::mem::replace(&mut *self.rec_ref, BasicTree::Empty)
    }

    /// If the current position is empty, puts the given value there instead.
    /// Intended to help writing tree algorithms.
    pub(in super::super) fn put_subtree(&mut self, new: BasicTree<D, T>) -> Option<()> {
        if self.rec_ref.is_empty() {
            *self.rec_ref = new;
            Some(())
        } else {
            None
        }
    }

    pub fn delete_with_alg_data(&mut self) -> Option<(D::Value, T)> {
        let mut node = self.take_subtree().into_node()?;
        if node.right.is_empty() {
            self.put_subtree(node.left).unwrap();
        } else {
            // find the next node and move it to the current position
            let mut walker = node.right.walker();
            while let Ok(_) = walker.go_left() {}
            let res = walker.go_up();
            assert_eq!(res, Ok(Side::Left));

            let mut boxed_replacement_node = walker.take_subtree().into_node_boxed().unwrap();
            assert!(boxed_replacement_node.left.is_empty());
            walker.put_subtree(boxed_replacement_node.right).unwrap();
            drop(walker);

            boxed_replacement_node.left = node.left;
            boxed_replacement_node.right = node.right;
            boxed_replacement_node.rebuild();
            self.put_subtree(BasicTree::Root(boxed_replacement_node))
                .unwrap();
        }
        Some((node.node_value, node.alg_data))
    }

    /// Returns how many times you need to go up in order to be a child of side `side`.
    /// i.e, if `side == Left`, it returns `1` if the current node is a left child.
    /// If it is a right child, but its parent is a left child, it returns `2`.
    /// And so on.
    /// If there isn't any, returns `None`.
    pub fn steps_until_sided_ancestor(&self, side: Side) -> Option<usize> {
        let mut res = 0;
        for s in self.is_left.iter().rev() {
            res += 1;
            if *s == side {
                return Some(res);
            }
        }
        return None;
    }
}

/// This implementation exists in order to rebuild the nodes
/// when the walker gets dropped
impl<'a, D: Data, T> Drop for BasicWalker<'a, D, T> {
    fn drop(&mut self) {
        self.go_to_root();
    }
}
