//! Methods module
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

// TODO - make this work for both filled and empty starting positions
// TODO - figure out how to make this callable like walker.next_empty()
/// If the walker is at an empty position, return an error.
/// Goes to the next empty position
pub fn next_empty<W: SomeWalker<A>, A: Data>(walker: &mut W) -> Result<(), ()> {
    walker.go_right()?; // if we're at an empty node, return error
    while !walker.is_empty() {
        walker.go_left().unwrap();
    }
    Ok(())
}

/// If the walker is at an empty position, return an error.
/// Goes to the previous empty position
pub fn previous_empty<W: SomeWalker<A>, A: Data>(walker: &mut W) -> Result<(), ()> {
    walker.go_left()?; // if we're at an empty node, return error
    while !walker.is_empty() {
        walker.go_right().unwrap();
    }
    Ok(())
}

/// Finds the next filled node.
/// If there isn't any, moves to root and return Err(()).
pub fn next_filled<W: SomeWalker<A>, A: Data>(walker: &mut W) -> Result<(), ()> {
    if !walker.is_empty() {
        next_empty(walker).unwrap();
    }
    loop {
        match walker.go_up() {
            Ok(Side::Left) => break,
            Ok(Side::Right) => (),
            Err(_) => return Err(()), // there was no next node
        }
    }
    Ok(())
}

/// Finds the previous filled node.
/// If there isn't any, moves to root and return Err(()).
pub fn previous_filled<W: SomeWalker<A>, A: Data>(walker: &mut W) -> Result<(), ()> {
    if !walker.is_empty() {
        previous_empty(walker).unwrap();
    }
    loop {
        match walker.go_up() {
            Ok(Side::Right) => break,
            Ok(Side::Left) => (),
            Err(_) => return Err(()), // there was no next node
        }
    }
    Ok(())
}

// TODO: finger searching.
/// Finds any node that the locator `Accept`s. Looks inside the whole tree.
/// If there isn't any, it finds the empty location where that node would be instead.
/// Moves the walker to the wanted position.
pub fn search_walker<W, D: Data, L>(walker: &mut W, locator: L)
where
    W: crate::trees::SomeWalker<D>,
    L: Locator<D>,
{
    walker.go_to_root();
    search_subtree(walker, locator);
}

/// Finds any node that the locator `Accept`s. Looks only inside the current subtree.
/// If there isn't any, it finds the empty location where that node would be instead.
/// Returns a walker at the wanted position.
pub fn search_subtree<W, D: Data, L>(walker: &mut W, locator: L)
where
    W: crate::trees::SomeWalker<D>,
    L: Locator<D>,
{
    while let Some(res) = walker_locate(walker, &locator) {
        match res {
            LocResult::Accept => break,
            LocResult::GoRight => walker.go_right().unwrap(),
            LocResult::GoLeft => walker.go_left().unwrap(),
        };
    }
}

/// Finds any node that the locator `Accept`s.
/// If there isn't any, it finds the empty location where that node would be instead.
/// Returns a walker at the wanted position.
pub fn search<TR, D: Data, L>(tree: TR, locator: L) -> TR::Walker
where
    TR: crate::trees::SomeTreeRef<D>,
    L: Locator<D>,
{
    let mut walker = tree.walker();
    search_walker(&mut walker, locator);
    walker
}

/// Returns the accumulated values on the locator's segment
/// Do not use with splay trees - it might mess up the complexity,
/// because it uses go_up().
///
/// Instead, use the specific [`SomeTree::segment_summary`]
pub fn segment_summary<TR, L, D: Data>(tree: TR, locator: L) -> D::Summary
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
                let first_half = segment_summary_on_suffix(&mut walker, locator.clone());
                // get back to the original node
                for _ in 0..walker.depth() - depth {
                    walker.go_up().unwrap();
                }
                walker.go_right().unwrap();
                let second_half = segment_summary_on_prefix(&mut walker, locator);

                return first_half + node_value + second_half;
            }
        }
    }

    // empty segment case
    Default::default()
}

fn segment_summary_on_suffix<W, L, A: Data>(walker: &mut W, locator: L) -> A::Summary
where
    W: SomeWalker<A>,
    L: Locator<A>,
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

fn segment_summary_on_prefix<W, L, A: Data>(walker: &mut W, locator: L) -> A::Summary
where
    W: SomeWalker<A>,
    L: Locator<A>,
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
