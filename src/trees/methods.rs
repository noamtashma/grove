//! Methods module
//! [TODO: outdated]
//! This module provides generic methods for use on general trees. for example,
//! search functions, querying on a segment of the tree, applying an
//! action on a segment of the tree, and so on.
//!
//! Since different balanced tree algorithms are different, the generic functions
//! may not work as intended. For example, splay trees shouldn't use the `go_up` method too much,
//! and so some generic functions which use `go_up` may have linear complexity when used with
//! splay trees.

use crate::*;
use locators::*;

// TODO: finger searching.

/// Returns the accumulated values on the locator's segment
/// Do not use with splay trees - it might mess up the complexity,
/// because it uses go_up().
///
/// Instead, use the specific [`SomeTree::segment_summary`]
pub fn segment_summary_unclonable<TR, L, D: Data>(tree: TR, locator: L) -> D::Summary
where
    TR: SomeTreeRef<D>,
    L: Locator<D>,
{
    use LocResult::*;

    let mut walker = tree.walker();
    while let Some(res) = walker_locate(&mut walker, &locator) {
        match res {
            GoRight => walker.go_right().unwrap(),
            GoLeft => walker.go_left().unwrap(),

            // at this point, we split into the two sides
            Accept => {
                let node_value = walker.node_summary();
                let depth = walker.depth();
                walker.go_left().unwrap();
                let first_half = segment_summary_on_suffix_unclonable(&mut walker, locator.clone());
                // get back to the original node
                for _ in 0..walker.depth() - depth {
                    walker.go_up().unwrap();
                }
                walker.go_right().unwrap();
                let second_half = segment_summary_on_prefix_unclonable(&mut walker, locator);

                return first_half + node_value + second_half;
            }
        }
    }

    // empty segment case
    Default::default()
}

fn segment_summary_on_suffix_unclonable<W, L, D: Data>(walker: &mut W, locator: L) -> D::Summary
where
    W: SomeWalker<D>,
    L: Locator<D>,
{
    let mut res = Default::default();
    use LocResult::*;

    while let Some(dir) = walker_locate(walker, &locator) {
        match dir {
            Accept => {
                res = walker.node_summary() + walker.right_subtree_summary().unwrap() + res;
                walker.go_left().unwrap();
            }
            GoRight => walker.go_right().unwrap(),
            GoLeft => panic!("inconsistent locator"),
        }
    }

    res
}

fn segment_summary_on_prefix_unclonable<W, L, D: Data>(walker: &mut W, locator: L) -> D::Summary
where
    W: SomeWalker<D>,
    L: Locator<D>,
{
    let mut res = Default::default();
    use LocResult::*;

    while let Some(dir) = walker_locate(walker, &locator) {
        match dir {
            Accept => {
                res = res + walker.left_subtree_summary().unwrap() + walker.node_summary();
                walker.go_right().unwrap();
            }
            GoRight => panic!("inconsistent locator"),
            GoLeft => walker.go_left().unwrap(),
        }
    }

    res
}

/// Applies an action on the locator's segment.
/// Do not use with splay trees - it might mess up the complexity,
/// because it uses go_up().
///
/// Don't use with actions that reverse segments. Panics otherwise.
///
/// Instead, use [`SomeTree::act_segment`]
pub fn act_segment<TR, L, D: Data>(tree: TR, action: D::Action, locator: L)
where
    TR: SomeTreeRef<D>,
    L: Locator<D>,
{
    assert!(!action.to_reverse());
    use LocResult::*;

    let mut walker = tree.walker();
    while let Some(res) = walker_locate(&mut walker, &locator) {
        match res {
            GoRight => walker.go_right().unwrap(),
            GoLeft => walker.go_left().unwrap(),

            // at this point, we split into the two sides
            Accept => {
                walker.act_node(action);
                let depth = walker.depth();
                walker.go_left().unwrap();
                act_on_suffix(&mut walker, action, locator.clone());
                // get back to the original node
                for _ in 0..walker.depth() - depth {
                    walker.go_up().unwrap();
                }
                walker.go_right().unwrap();
                act_on_prefix(&mut walker, action, locator);
                return;
            }
        }
    }
}

// Only works if `action.to_reverse()` is false. does not check.
fn act_on_suffix<W, L, D: Data>(walker: &mut W, action: D::Action, locator: L)
where
    W: SomeWalker<D>,
    L: Locator<D>,
{
    use LocResult::*;

    while let Some(dir) = walker_locate(walker, &locator) {
        match dir {
            Accept => {
                walker.act_node(action);
                walker.act_right_subtree(action).unwrap();
                walker.go_left().unwrap();
            }
            GoRight => walker.go_right().unwrap(),
            GoLeft => panic!("inconsistent locator"),
        }
    }
}

// Only works if `action.to_reverse()` is false. does not check.
fn act_on_prefix<W, L, D: Data>(walker: &mut W, action: D::Action, locator: L)
where
    W: SomeWalker<D>,
    L: Locator<D>,
{
    use LocResult::*;

    while let Some(dir) = walker_locate(walker, &locator) {
        match dir {
            Accept => {
                walker.act_node(action);
                walker.act_left_subtree(action).unwrap();
                walker.go_right().unwrap();
            }
            GoRight => panic!("inconsistent locator"),
            GoLeft => walker.go_left().unwrap(),
        }
    }
}





/// A BasicWalker version that is immutable, and can only go down.
#[derive(Copy, Clone)]
struct ImmDownBasicWalker<'a, D: Data, T=()> {
    tree: &'a basic_tree::BasicTree<D, T>,
    current_action: D::Action, // to be applied to everything in `tree`
    // note: these should always have `current_action` already applied to them,
    // and in the already reversed order if `current_action.to_reverse() == true`.

    // the summary of everything to the left
    left_summary: D::Summary,
    // the summary of everything to the right
    right_summary: D::Summary,
}

impl<'a, D: Data, T> ImmDownBasicWalker<'a, D, T> {
    pub fn new(tree: &'a basic_tree::BasicTree<D, T>) -> Self {
        ImmDownBasicWalker {
            tree,
            current_action : Default::default(),
            left_summary: Default::default(),
            right_summary: Default::default(),
        }
    }

    pub fn temp_go_left(&mut self) -> Option<()> {
        if let Some(node) = self.tree.node() {
            self.right_summary =
                self.current_action.act(
                    D::to_summary(&node.node_value)
                    + node.right.subtree_summary()
                )
                + self.right_summary;
            self.tree = &node.left;
        }
        None
    }
}


/// Returns the accumulated values on the locator's segment
/// Do not use with splay trees - it might mess up the complexity,
/// because it uses go_up().
///
/// Instead, use the specific [`SomeTree::segment_summary`]
pub fn segment_summary<D: Data, T, L>(mut tree: &basic_tree::BasicTree<D, T>, locator: L) -> D::Summary
where
    L: Locator<D>,
    D::Value: Clone,
{
    use trees::*;
    use LocResult::*;

    let mut left_summary: D::Summary = Default::default();
    let mut right_summary: D::Summary = Default::default();
    let mut current_action: D::Action = Default::default();

    while let Some(node) = tree.node() {
        current_action = current_action + *node.action();

        let direction = clone_locate(
            current_action,
            left_summary,
            &current_action.act(node.node_value.clone()),
            right_summary,
            &locator
        );
    
        match direction {
            GoRight => {
                left_summary = left_summary +
                    current_action.act(node.left.subtree_summary()
                        + D::to_summary(&node.node_value)
                    );
                tree = &node.right
            },
            GoLeft => {
                right_summary =
                    current_action.act(
                        D::to_summary(&node.node_value)
                        + node.right.subtree_summary()
                    )
                    + right_summary;
                tree = &node.left
            },

            // at this point, we split into the two sides
            Accept => {
                let left_node_summary = current_action.act(node.left.subtree_summary());
                let node_summary = current_action.act(D::to_summary(&node.node_value));
                let right_node_summary = current_action.act(node.right.subtree_summary());
                
                let first_half = segment_summary_on_suffix(
                    &node.left, 
                    current_action, 
                    left_summary, 
                    node_summary + right_node_summary + right_summary, 
                    locator.clone()
                );
                let second_half = segment_summary_on_prefix(
                    &node.right, 
                    current_action, 
                    left_summary + left_node_summary + node_summary, 
                    right_summary, locator
                );

                return first_half + node_summary + second_half;
            }
        }
    }

    // empty locator case
    Default::default()
}

fn segment_summary_on_suffix<D:Data, T, L>(
        mut tree: &basic_tree::BasicTree<D, T>, 
        mut current_action: D::Action, 
        mut left_summary: D::Summary,
        right_summary: D::Summary,
        locator: L
    ) -> D::Summary
where
    L: Locator<D>,
    D::Value: Clone,
{
    use trees::*;
    use LocResult::*;

    let mut current_summary: D::Summary = Default::default();

    while let Some(node) = tree.node() {
        current_action = current_action + *node.action();
        let direction = clone_locate(
            current_action,
            left_summary,
            &current_action.act(node.node_value.clone()),
            current_summary + right_summary,
            &locator
        );
    
        match direction {
            GoRight => {
                left_summary = left_summary +
                    current_action.act(node.left.subtree_summary()
                        + D::to_summary(&node.node_value)
                    );
                tree = &node.right
            }
            GoLeft => panic!("inconsistent locator"),

            Accept => {
                current_summary =
                    current_action.act(
                        D::to_summary(&node.node_value)
                        + node.right.subtree_summary())
                    + current_summary;
                
                tree = &node.left;
            }
        }
    }
    // when finished
    current_summary
}

fn segment_summary_on_prefix<D:Data, T, L>(
        mut tree: &basic_tree::BasicTree<D, T>, 
        mut current_action: D::Action, 
        left_summary: D::Summary,
        mut right_summary: D::Summary,
        locator: L
    ) -> D::Summary
where
    L: Locator<D>,
    D::Value: Clone,
{
    use trees::*;
    use LocResult::*;

    let mut current_summary: D::Summary = Default::default();

    while let Some(node) = tree.node() {
        current_action = current_action + *node.action();
        let direction = locator.locate(
            left_summary + current_summary,
            &current_action.act(node.node_value.clone()),
            right_summary
        );
    
        match direction {
            GoRight => panic!("inconsistent locator"),
            GoLeft => {
                right_summary = current_action.act(
                        D::to_summary(&node.node_value)
                        + node.right.subtree_summary()
                    ) + right_summary;
                tree = &node.left;
            }

            Accept => {
                current_summary = current_summary
                    + current_action.act(
                        node.left.subtree_summary()
                        + D::to_summary(&node.node_value)
                    );
                
                tree = &node.right;
            }
        }
    }
    // when finished
    current_summary
}
