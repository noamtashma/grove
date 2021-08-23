// This is a private module, so no documentation for it directly.
// instead look for documentation of the `PersistentWalker` struct.

use super::*;
use recursive_reference::*;

use crate::trees::SomeWalker; // in order to be able to use our own go_up method
                              // use super::implementations::*;

/// The persistent tree holds the invariant that every `&mut PersistentTree` containted
/// in the `rec_ref`, except the last one, must have a unique `Rc`. That is, its `Rc` is the
/// owner of the corresponding `PersistentNode` and there isn't any other `Rc` pointing to it.
/// This is because when going down the tree, the walker will clone the `Rc`'s if this is not
/// already the case.
///
/// When this invariant is violated, this error is thrown.
const UNIQUE_RC_IN_RECREF: &str = "persistent tree inner invariant violated";

pub(super) struct Frame<D: ?Sized + Data> {
    pub left: D::Summary,
    pub right: D::Summary,
}

// the default clone implementation requires that D: Clone, which is uneccessary
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

/// This struct implements a walker for the [`PersistentTree`] type.
/// It is struct that has a mutable reference of the tree, and allows you to walk up and down on it.
/// The walker may also be in a position which is the son of a node, but doesn't contain
/// a node by itself, and then it is said to be in an empty position.
///
/// Walkers for other kinds of trees may be built by wrapping around the [`PersistentWalker`] type,
/// as tree types can be built by wrapping around the [`PersistentTree`] type.
///
/// The walker will automatically go back up the tree to the root when dropped,
/// in order to rebuild all the nodes.
///
/// Internally, [`recursive_reference::RecRef`] is used, in order to be able to dynamically
/// go up and down the tree. Without the `RecRef`, the walker would have to duplicate its whole path
/// even when the current tree isn't sharing its nodes.
#[derive(destructure)]
pub struct PersistentWalker<'a, D: Data, T = ()> {
    /// The telescope, holding references to all the subtrees from the root to the
    /// current position.
    pub(super) rec_ref: RecRef<'a, PersistentTree<D, T>>,

    /// This array holds the accumulation of all the values left of the subtree, and
    /// all of the values right of the subtree, for every subtree from the root to
    /// the current subtree.
    pub(super) vals: Vec<Frame<D>>,

    /// This array holds for every node, whether the next subtree in the walker
    /// is its left son or the right son. (true corresponds to the left son).
    /// This array is always one shorter than [`PersistentWalker::rec_ref`] and [`PersistentWalker::vals`],
    /// because the last node has no son in the walker.
    pub(super) is_left: Vec<Side>,
}

impl<'a, D: Data, T> PersistentWalker<'a, D, T> {
    /// Creates a new walker that walks on the given tree.
    pub fn new(tree: &'a mut PersistentTree<D, T>) -> PersistentWalker<'a, D, T>
    where
        PersistentNode<D, T>: Clone,
    {
        tree.access();
        PersistentWalker {
            rec_ref: RecRef::new(tree),
            vals: vec![Frame::empty()],
            is_left: vec![],
        }
    }

    /// Returns a new walker at the root of the tree, but treats it as if it started in the
    /// of a larger tree, where the summaries to the left and right are
    /// `left_summary` and `right_summary`.
    pub fn new_with_context(
        tree: &'a mut PersistentTree<D, T>,
        left_summary: D::Summary,
        right_summary: D::Summary,
    ) -> PersistentWalker<'a, D, T>
    where
        PersistentNode<D, T>: Clone,
    {
        tree.access();
        PersistentWalker {
            rec_ref: RecRef::new(tree),
            vals: vec![Frame {
                left: left_summary,
                right: right_summary,
            }],
            is_left: vec![],
        }
    }

    /// Returns true if at an empty position.
    pub fn is_empty(&self) -> bool
    where
        PersistentNode<D, T>: Clone,
    {
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

    /// Not public since the walker should maintain the invariant that the current position
    /// is always clean. Ergo, for internal use.
    fn rebuild(&mut self)
    where
        PersistentNode<D, T>: Clone,
    {
        self.rec_ref.rebuild();
    }

    /// Gives access to the current position.
    pub fn inner(&self) -> &PersistentTree<D, T> {
        &*self.rec_ref
    }

    pub(in super::super) fn inner_mut(&mut self) -> &mut PersistentTree<D, T> {
        &mut *self.rec_ref
    }

    /// Gives access to the current node, if not at an empty position.
    pub fn node(&self) -> Option<&PersistentNode<D, T>> {
        self.rec_ref.node()
    }

    fn node_mut(&mut self) -> Option<&mut PersistentNode<D, T>>
    where
        PersistentNode<D, T>: Clone,
    {
        self.rec_ref.node_mut()
    }

    /// This is a copy of the [`SomeWalker::go_up`] function. However, it doesn't require
    /// `PersistentNode<D, T>: Clone`. This is important so that the `Drop` instance can
    /// be unconditional.
    fn go_up(&mut self) -> Result<Side, ()> {
        const NO_VALUE_ERROR: &str = "invariant violated: walker should not be empty";
        match self.is_left.pop() {
            None => Err(()),
            Some(b) => {
                RecRef::pop(&mut self.rec_ref).expect(NO_VALUE_ERROR);
                self.vals.pop().expect(NO_VALUE_ERROR);
                self.rec_ref.rebuild_unique().expect(UNIQUE_RC_IN_RECREF);
                Ok(b)
            }
        }
    }

    /// Performs a left rotation
    /// Returns [`None`] if this is an empty tree or if it has no right son.
    pub fn rot_left(&mut self) -> Option<()>
    where
        PersistentNode<D, T>: Clone,
    {
        self.rot_left_with_custom_rebuilder(|_| {})
    }

    /// Performs a left rotation.
    /// Returns [`None`] if this is an empty tree or if it has no right son.
    /// Uses a callback for a rebuilding action, that will be applied in addition
    /// to the regular summary rebuilding
    pub fn rot_left_with_custom_rebuilder<F: FnMut(&mut PersistentNode<D, T>)>(
        &mut self,
        mut rebuilder: F,
    ) -> Option<()>
    where
        PersistentNode<D, T>: Clone,
    {
        let owned_tree = std::mem::replace(&mut *self.rec_ref, PersistentTree::Empty);

        let mut bn1: Rc<PersistentNode<D, T>> = owned_tree.into_node_rc()?;
        assert!(bn1.action.is_identity());
        let bn1_ref = Rc::make_mut(&mut bn1);

        let mut bn2: Rc<PersistentNode<D, T>> = bn1_ref.right.take().into_node_rc()?;
        let bn2_ref = Rc::make_mut(&mut bn2);
        bn2_ref.access();

        bn1_ref.right = bn2_ref.left.take();
        bn2_ref.subtree_summary = bn1_ref.subtree_summary; // this is insetad of bn2.rebuild(), since we already know the result
        bn1_ref.rebuild();
        rebuilder(bn1_ref);
        bn2_ref.left = PersistentTree::from_rc_node(bn1);
        // bn2.rebuild()
        rebuilder(bn2_ref);

        *self.rec_ref = PersistentTree::from_rc_node(bn2); // restore the node back
        Some(())
    }

    /// Performs a right rotation
    /// Returns [`None`] if this node has no left son.
    pub fn rot_right(&mut self) -> Option<()>
    where
        PersistentNode<D, T>: Clone,
    {
        self.rot_right_with_custom_rebuilder(|_| {})
    }

    /// Performs a right rotation.
    /// Returns [`None`] if this node has no left son.
    /// Uses a callback for a rebuilding action, that will be applied in addition
    /// to the regular summary rebuilding
    pub fn rot_right_with_custom_rebuilder<F: FnMut(&mut PersistentNode<D, T>)>(
        &mut self,
        mut rebuilder: F,
    ) -> Option<()>
    where
        PersistentNode<D, T>: Clone,
    {
        let owned_tree = std::mem::replace(&mut *self.rec_ref, PersistentTree::Empty);

        let mut bn1: Rc<PersistentNode<D, T>> = owned_tree.into_node_rc()?;
        assert!(bn1.action.is_identity());
        let bn1_ref = Rc::make_mut(&mut bn1);

        let mut bn2: Rc<PersistentNode<D, T>> = bn1_ref.left.take().into_node_rc()?;
        let bn2_ref = Rc::make_mut(&mut bn2);
        bn2_ref.access();

        bn1_ref.left = bn2_ref.right.take();
        bn2_ref.subtree_summary = bn1_ref.subtree_summary; // this is insetad of bn2.rebuild(), since we already know the result
        bn1_ref.rebuild();
        rebuilder(bn1_ref);
        bn2_ref.right = PersistentTree::from_rc_node(bn1);
        // bn2.rebuild()
        rebuilder(bn2_ref);

        *self.rec_ref = PersistentTree::from_rc_node(bn2); // restore the node back
        Some(())
    }

    /// Performs rot_left if `side` is [`Side::Left`]
    /// rot_right otherwise
    pub fn rot_side(&mut self, side: Side) -> Option<()>
    where
        PersistentNode<D, T>: Clone,
    {
        match side {
            Side::Left => self.rot_left(),
            Side::Right => self.rot_right(),
        }
    }

    /// Performs rot_left if b is true
    /// rot_right otherwise
    pub fn rot_side_with_custom_rebuilder<F: FnMut(&mut PersistentNode<D, T>)>(
        &mut self,
        side: Side,
        rebuilder: F,
    ) -> Option<()>
    where
        PersistentNode<D, T>: Clone,
    {
        match side {
            Side::Left => self.rot_left_with_custom_rebuilder(rebuilder),
            Side::Right => self.rot_right_with_custom_rebuilder(rebuilder),
        }
    }

    /// Rotates so that the current node moves up.
    /// Basically moves up and then calls rot_side.
    /// Fails if the current node is the root.
    pub fn rot_up(&mut self) -> Result<Side, ()>
    where
        PersistentNode<D, T>: Clone,
    {
        let b = self.go_up()?;
        self.rot_side(b.flip())
            .expect("original node went missing?");
        Ok(b)
    }

    /// Rotates so that the current node moves up.
    /// Basically moves up and then calls rot_side.
    /// Fails if the current node is the root.
    pub fn rot_up_with_custom_rebuilder<F: FnMut(&mut PersistentNode<D, T>)>(
        &mut self,
        rebuilder: F,
    ) -> Result<Side, ()>
    where
        PersistentNode<D, T>: Clone,
    {
        let b = self.go_up()?;
        self.rot_side_with_custom_rebuilder::<F>(b.flip(), rebuilder)
            .expect("original node went missing?");
        Ok(b)
    }

    /// Goes up all the way to the root of the tree.
    /// This is called in the walker's [`Drop`] instacne, to rebuild all of the tree's values.
    pub fn go_to_root(&mut self) {
        while self.go_up().is_ok() {}
    }

    /// This takes the walker and turns it into a reference to the root
    pub fn root_into_ref(mut self) -> &'a mut PersistentTree<D, T>
    where
        PersistentNode<D, T>: Clone,
    {
        // go to the root
        self.go_to_root();
        let (tel, _, _) = self.destructure();
        RecRef::into_ref(tel)
    }

    /// Creates a walker that can only access the current subtree. However,
    /// it knows the context of the tree around it, so that locators still work on it as expected
    /// (e.g, looking for the seventh element will still find the element that is the seventh in
    /// the whole tree, and not the seventh in the subtree).
    pub fn detached_walker(&mut self) -> PersistentWalker<D, T>
    where
        PersistentNode<D, T>: Clone,
    {
        let left = self.far_left_summary();
        let right = self.far_right_summary();
        PersistentWalker::new_with_context(self.inner_mut(), left, right)
    }

    /// Inserts a node along with the balancing algorithm's custom data.
    pub fn insert_with_alg_data(&mut self, value: D::Value, alg_data: T) -> Option<()> {
        match *self.rec_ref {
            Empty => {
                *self.rec_ref = PersistentTree::from_node(PersistentNode::new_alg(value, alg_data));
                Some(())
            }
            _ => None,
        }
    }

    /// Takes the current subtree out of the tree, and writes `Empty` instead.
    /// Intended to help writing tree algorithms.
    pub(in super::super) fn take_subtree(&mut self) -> PersistentTree<D, T> {
        std::mem::replace(&mut *self.rec_ref, PersistentTree::Empty)
    }

    /// If the current position is empty, puts the given value there instead.
    /// Intended to help writing tree algorithms.
    pub(in super::super) fn put_subtree(&mut self, new: PersistentTree<D, T>) -> Option<()>
    where
        PersistentNode<D, T>: Clone,
    {
        if self.rec_ref.is_empty() {
            *self.rec_ref = new;
            Some(())
        } else {
            None
        }
    }

    /// deletes a node and returns the node's value along with
    /// the algorithm's custom data.
    pub fn delete_with_alg_data(&mut self) -> Option<(D::Value, T)>
    where
        D::Value: Clone,
        T: Clone,
    {
        let mut node = self.take_subtree().into_node()?;
        if node.right.is_empty() {
            self.put_subtree(node.left).unwrap();
        } else {
            // find the next node and move it to the current position
            let mut walker = node.right.walker();
            while walker.go_left().is_ok() {}
            let res = walker.go_up();
            assert_eq!(res, Ok(Side::Left));

            let mut rc_replacement_node = walker.take_subtree().into_node_rc().unwrap();
            assert!(rc_replacement_node.left.is_empty());
            let mut rc_replacement_ref = Rc::make_mut(&mut rc_replacement_node);
            let right = rc_replacement_ref.right.take();
            walker.put_subtree(right).unwrap();
            drop(walker);

            rc_replacement_ref.left = node.left;
            rc_replacement_ref.right = node.right;
            rc_replacement_ref.rebuild();
            self.put_subtree(PersistentTree::from_rc_node(rc_replacement_node))
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
        None
    }
}

/// This implementation exists in order to rebuild the nodes
/// when the walker gets dropped
impl<'a, D: Data, T> Drop for PersistentWalker<'a, D, T> {
    fn drop(&mut self) {
        self.go_to_root();
    }
}
