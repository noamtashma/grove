//! This module implements the tree traits for the [`BasicTree`] and [`BasicWalker`]
//! It is mostly a separate file from the main module file, since it's a private module, and its
//! contents are re-exported.

use super::super::*; // crate::trees::*
use super::*;
use recursive_reference::RecRef;

const NO_VALUE_ERROR: &str = "invariant violated: RecRef can't be empty";

impl<D: Data> SomeTree<D> for BasicTree<D> {
    fn segment_summary<L>(&mut self, locator: L) -> D::Summary
    where
        L: Locator<D>,
    {
        methods::segment_summary(self, locator)
    }

    fn act_segment<L>(&mut self, action: D::Action, locator: L)
    where
        L: Locator<D>,
    {
        methods::act_segment(self, action, locator);
    }

    type TreeData = ();
    fn iter_locator<'a, L: locators::Locator<D>>(
        &'a mut self,
        locator: L,
    ) -> basic_tree::iterators::IterLocator<'a, D, L> {
        iterators::IterLocator::new(self, locator)
    }

    /// Checks that invariants remain correct. i.e., that every node's summary
    /// is the sum of the summaries of its children.
    /// If it is not, panics.
    fn assert_correctness(&self)
    where
        D::Summary: Eq,
    {
        self.assert_correctness_locally();
        if let Root(node) = self {
            node.left.assert_correctness();
            node.right.assert_correctness();
        }
    }
}

impl<D: Data> Default for BasicTree<D> {
    fn default() -> Self {
        Empty
    }
}

impl<D: Data> std::iter::FromIterator<D::Value> for BasicTree<D> {
    fn from_iter<T: IntoIterator<Item = D::Value>>(iter: T) -> Self {
        iterators::build(iter.into_iter())
    }
}

impl<D: Data> IntoIterator for BasicTree<D> {
    type Item = D::Value;
    type IntoIter = iterators::IntoIter<D, std::ops::RangeFull>;
    fn into_iter(self) -> Self::IntoIter {
        iterators::IntoIter::new(self, ..)
    }
}

impl<'a, D: Data, T> SomeTreeRef<D> for &'a mut BasicTree<D, T> {
    type Walker = BasicWalker<'a, D, T>;

    fn walker(self) -> Self::Walker {
        BasicWalker::new(self)
    }
}

impl<'a, D: Data, T> SomeWalker<D> for BasicWalker<'a, D, T> {
    fn go_left(&mut self) -> Result<(), ()> {
        let mut frame = self.vals.last().expect(NO_VALUE_ERROR).clone();
        let res = RecRef::extend_result(&mut self.rec_ref, |tree| {
            if let Some(node) = tree.node_mut() {
                // update values
                frame.right = node.node_summary() + node.right.subtree_summary() + frame.right;
                node.left.access();
                Ok(&mut node.left)
            } else {
                Err(())
            }
        });
        // push side information
        if res.is_ok() {
            self.is_left.push(Side::Left); // went left
            self.vals.push(frame);
        }
        res
    }

    fn go_right(&mut self) -> Result<(), ()> {
        let mut frame = self.vals.last().expect(NO_VALUE_ERROR).clone();
        let res = RecRef::extend_result(&mut self.rec_ref, |tree| {
            if let Some(node) = tree.node_mut() {
                // update values
                frame.left = frame.left + node.left.subtree_summary() + node.node_summary();

                node.right.access();
                Ok(&mut node.right)
            } else {
                Err(())
            }
        });
        // push side information
        if res.is_ok() {
            self.is_left.push(Side::Right); // went right
            self.vals.push(frame);
        }
        res
    }

    fn go_up(&mut self) -> Result<Side, ()> {
        match self.is_left.pop() {
            None => Err(()),
            Some(b) => {
                RecRef::pop(&mut self.rec_ref).expect(NO_VALUE_ERROR);
                self.vals.pop().expect(NO_VALUE_ERROR);
                self.rec_ref.rebuild();
                Ok(b)
            }
        }
    }

    fn depth(&self) -> usize {
        self.is_left.len()
    }

    fn far_left_summary(&self) -> D::Summary {
        self.vals.last().expect(NO_VALUE_ERROR).left
    }
    fn far_right_summary(&self) -> D::Summary {
        self.vals.last().expect(NO_VALUE_ERROR).right
    }

    // fn inner(&self) -> &BasicTree<A> {
    //     &*self.rec_ref
    // }

    fn value(&self) -> Option<&D::Value> {
        let value = self.rec_ref.node()?.node_value_clean();
        Some(value)
    }
}

impl<D: Data, T> SomeEntry<D> for BasicTree<D, T> {
    fn node_summary(&self) -> D::Summary {
        match self.node() {
            None => Default::default(),
            Some(node) => node.node_summary(),
        }
    }

    fn subtree_summary(&self) -> D::Summary {
        if let Some(node) = self.node() {
            node.subtree_summary()
        } else {
            Default::default()
        }
    }

    fn left_subtree_summary(&self) -> Option<D::Summary> {
        let res = self.node()?.left.subtree_summary();
        Some(res)
    }

    fn right_subtree_summary(&self) -> Option<D::Summary> {
        let res = self.node()?.right.subtree_summary();
        Some(res)
    }

    fn with_value<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut D::Value) -> R,
    {
        let node = self.node_mut()?;
        let value = node.node_value_mut(); // performs `access()`
        let res = f(value);
        node.rebuild();
        Some(res)
    }

    fn act_subtree(&mut self, action: D::Action) {
        if let Some(node) = self.node_mut() {
            node.act(action);
        }
    }

    fn act_node(&mut self, action: D::Action) -> Option<()> {
        let node = self.node_mut()?;
        node.act_value(action);
        node.rebuild();
        Some(())
    }

    fn act_left_subtree(&mut self, action: D::Action) -> Option<()> {
        let node = self.node_mut()?;
        node.access();
        node.left.act_subtree(action);
        node.rebuild();
        Some(())
    }

    fn act_right_subtree(&mut self, action: D::Action) -> Option<()> {
        let node = self.node_mut()?;
        node.access();
        node.right.act_subtree(action);
        node.rebuild();
        Some(())
    }

    fn assert_correctness_locally(&self)
    where
        D::Summary: Eq,
    {
        if let Some(node) = self.node() {
            BasicNode::assert_correctness_locally(node);
        }
    }
}

impl<'a, D: Data, T> SomeEntry<D> for BasicWalker<'a, D, T> {
    fn node_summary(&self) -> D::Summary {
        self.rec_ref.node_summary()
    }

    fn subtree_summary(&self) -> D::Summary {
        self.rec_ref.subtree_summary()
    }

    fn left_subtree_summary(&self) -> Option<D::Summary> {
        self.rec_ref.left_subtree_summary()
    }

    fn right_subtree_summary(&self) -> Option<D::Summary> {
        self.rec_ref.right_subtree_summary()
    }

    fn with_value<F, R>(&mut self, f: F) -> Option<R>
    where
        F: FnOnce(&mut D::Value) -> R,
    {
        self.rec_ref.with_value(f)
    }

    fn act_subtree(&mut self, action: D::Action) {
        self.rec_ref.act_subtree(action);
        self.rec_ref.access();
    }

    fn act_node(&mut self, action: D::Action) -> Option<()> {
        let node = self.rec_ref.node_mut()?;
        action.act_inplace(&mut node.node_value);
        node.rebuild();
        Some(())
    }

    fn act_left_subtree(&mut self, action: D::Action) -> Option<()> {
        let node = self.rec_ref.node_mut()?;
        node.left.act_subtree(action);
        node.rebuild();
        Some(())
    }

    fn act_right_subtree(&mut self, action: D::Action) -> Option<()> {
        let node = self.rec_ref.node_mut()?;
        node.right.act_subtree(action);
        node.rebuild();
        Some(())
    }

    fn assert_correctness_locally(&self)
    where
        D::Summary: Eq,
    {
        self.inner().assert_correctness_locally();
    }
}

impl<'a, D: Data> ModifiableWalker<D> for BasicWalker<'a, D> {
    /// Inserts the value into the tree at the current empty position.
    /// If the current position is not empty, return [`None`].
    /// When the function returns, the walker will be at the position the node
    /// was inserted.
    fn insert(&mut self, value: D::Value) -> Option<()> {
        self.insert_with_alg_data(value, ())
    }

    /// Removes the current value from the tree, and returns it.
    /// If currently at an empty position, returns [`None`].
    /// After deletion, the walker will stay at the same position, but the subtree below it may change
    /// and the current node will be a different node (of course).
    fn delete(&mut self) -> Option<D::Value> {
        let res = self.delete_with_alg_data()?;
        Some(res.0)
    }
}

#[test]
fn basic_tree_delete() {
    for i in 0..9 {
        let arr = vec![3, 5, 1, 4, 7, 8, 9, 20, 11];
        let mut tree: BasicTree<example_data::StdNum> = arr.iter().cloned().collect();
        let mut walker = methods::search(&mut tree, i);
        assert_eq!(walker.value().cloned(), Some(arr[i]));
        let res = walker.delete();
        assert_eq!(res, Some(arr[i]));
        drop(walker);
        assert_eq!(
            tree.into_iter().collect::<Vec<_>>(),
            arr[..i]
                .iter()
                .chain(arr[i + 1..].iter())
                .cloned()
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn basic_tree_insert() {
    for i in 0..10 {
        let arr = vec![3, 5, 1, 4, 7, 8, 9, 20, 11];
        let new_val = 13;
        let mut tree: BasicTree<example_data::StdNum> = arr.iter().cloned().collect();
        let mut walker = methods::search(&mut tree, i..i);
        walker.insert(new_val);
        assert_eq!(walker.value().cloned(), Some(new_val));
        drop(walker);
        assert_eq!(
            tree.into_iter().collect::<Vec<_>>(),
            arr[..i]
                .iter()
                .chain([new_val].iter())
                .chain(arr[i..].iter())
                .cloned()
                .collect::<Vec<_>>()
        );
    }
}
