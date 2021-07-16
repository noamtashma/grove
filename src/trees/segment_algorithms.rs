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

use trees::basic_tree::{*, ImmDownBasicWalker};

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
    while let Some(res) = query_locator(&mut walker, &locator) {
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

    while let Some(dir) = query_locator(walker, &locator) {
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

    while let Some(dir) = query_locator(walker, &locator) {
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
    assert!(!action.to_reverse(), "This tree type might not support reversals");
    use LocResult::*;

    let mut walker = tree.walker();
    while let Some(res) = query_locator(&mut walker, &locator) {
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

    while let Some(dir) = query_locator(walker, &locator) {
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

    while let Some(dir) = query_locator(walker, &locator) {
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

const SUDDENLY_EMPTY_ERROR: &'static str = "The locator unexpectedly became empty";
const INCONSISTENT_LOCATOR_ERROR: &'static str = "inconsistent locator";

/// Returns the accumulated values on the locator's segment
/// Do not use with splay trees - it might mess up the complexity,
/// because it uses go_up().
///
/// Instead, use the specific [`SomeTree::segment_summary`]
pub fn segment_summary_imm<D: Data, T, L>(tree: &BasicTree<D, T>, locator: L) -> D::Summary
where
    L: Locator<D>,
    D::Value: Clone,
{
    use locators::LocResult::*;
    use trees::*;

    let mut walker = ImmDownBasicWalker::new(tree);
    while let Some(direction) = walker.query_locator(&locator) {
        match direction {
            GoLeft => {
                walker.go_left();
            }
            GoRight => {
                walker.go_right();
            }
            Accept => {
                let current_node_summary = walker.node_summary().expect(SUDDENLY_EMPTY_ERROR);
                let mut left_walker = walker.clone();
                let mut right_walker = walker;
                left_walker.go_left().expect(SUDDENLY_EMPTY_ERROR);
                right_walker.go_right().expect(SUDDENLY_EMPTY_ERROR);

                let left_half = segment_summary_on_suffix(left_walker, locator.clone());
                let right_half = segment_summary_on_prefix(right_walker, locator);
                return left_half + current_node_summary + right_half;
            }
        }
    }
    // Empty segment case
    Default::default()
}

/// Returns the summary of the segment in the current tree,
/// provided the segment is a suffix.
fn segment_summary_on_suffix<D: Data, T, L>(
    mut walker: ImmDownBasicWalker<D, T>,
    locator: L,
) -> D::Summary
where
    L: Locator<D>,
    D::Value: Clone,
{
    use locators::LocResult::*;
    use trees::*;
    let mut result: D::Summary = Default::default();

    while let Some(direction) = walker.query_locator(&locator) {
        match direction {
            GoLeft => panic!("{}", INCONSISTENT_LOCATOR_ERROR),
            GoRight => walker.go_right().expect(SUDDENLY_EMPTY_ERROR),
            Accept => {
                let extra = walker.go_left_extra().expect(SUDDENLY_EMPTY_ERROR);
                result = extra + result;
            }
        }
    }
    result
}

/// Returns the summary of the segment in the current tree,
/// provided the segment is a suffix.
fn segment_summary_on_prefix<D: Data, T, L>(
    mut walker: ImmDownBasicWalker<D, T>,
    locator: L,
) -> D::Summary
where
    L: Locator<D>,
    D::Value: Clone,
{
    use locators::LocResult::*;
    use trees::*;
    let mut result: D::Summary = Default::default();

    while let Some(direction) = walker.query_locator(&locator) {
        match direction {
            GoLeft => walker.go_left().expect(SUDDENLY_EMPTY_ERROR),
            GoRight => panic!("{}", INCONSISTENT_LOCATOR_ERROR),
            Accept => {
                let extra = walker.go_right_extra().expect(SUDDENLY_EMPTY_ERROR);
                result = result + extra;
            }
        }
    }
    result
}