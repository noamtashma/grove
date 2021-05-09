//! Methods module
//! This module provides generic methods for use on general trees. for example,
//! search functions, querying on a segment of the tree, applying an
//! action on a segment of the tree, and so on.
//!
//! Since different balanced tree algorithms are different, the generic functions
//! may not work as intended. For example, splay trees shouldn't use the `go_up` method too much,
//! and so some generic functions which use `go_up` may have linear complexity when used with
//! splay trees.


use data::example_data::Keyed;

use crate::*;
use locators::*;
// use std::iter::Iterator;




// TODO - make this work for both filled and empty starting positions
// TODO - figure out how to make this callable like walker.next_empty()
/// If the walker is at an empty position, return an error.
/// Goes to the next empty position
pub fn next_empty<W : SomeWalker<A>, A : Data>(walker : &mut W) -> Result<(), ()> {
    walker.go_right()?; // if we're at an empty node, return error
    while !walker.is_empty() {
        walker.go_left().unwrap();
    }
    Ok(())
}

/// If the walker is at an empty position, return an error.
/// Goes to the previous empty position
pub fn previous_empty<W : SomeWalker<A>, A : Data>(walker : &mut W) -> Result<(), ()> {
    walker.go_left()?; // if we're at an empty node, return error
    while !walker.is_empty() {
        walker.go_right().unwrap();
    }
    Ok(())
}

/// Finds the next filled node.
/// If there isn't any, moves to root and return Err(()).
pub fn next_filled<W : SomeWalker<A>, A : Data>(walker : &mut W) -> Result<(), ()> {
    if !walker.is_empty() {
        next_empty(walker).unwrap();
    }
    loop {
        match walker.go_up() {
            Ok(true) => break,
            Ok(false) => (),
            Err(_) => return Err(()), // there was no next node
        }
    }
    return Ok(());
}


/// Finds the previous filled node.
/// If there isn't any, moves to root and return Err(()).
pub fn previous_filled<W : SomeWalker<A>, A : Data>(walker : &mut W) -> Result<(), ()> {
    if !walker.is_empty() {
        previous_empty(walker).unwrap();
    }
    loop {
        match walker.go_up() {
            Ok(false) => break,
            Ok(true) => (),
            Err(_) => return Err(()), // there was no next node
        }
    }
    return Ok(());
}

// TODO: make an iterator
/// Returns a vector of all the values in the tree.
pub fn to_array<A : Data, TR>(tree : TR)
-> Vec<A::Value> where
TR : SomeTreeRef<A>,
A::Value : Clone,
{
    let mut walker = tree.walker();
    let mut res = vec![];
    while let Ok(_) = walker.go_left()
        {}

    while let Ok(_) = next_filled(&mut walker) {
        if let Some(value) = walker.value() {
            res.push(value.clone());
        } else {panic!()}
    }
    res
}


// TODO: make this return an error.
/// Panics if a key was reused.
pub fn insert_by_key<D : Data, TR>(tree : TR, data : D::Value)
    -> TR::Walker where
    TR : SomeTreeRef<D>,
    TR::Walker : ModifiableWalker<D>,
    D::Value : crate::data::example_data::Keyed,
    //<A as data::Data>::Value : std::fmt::Debug,
{
    insert_by_locator(tree, &locate_by_key::<D>(&data.get_key()) , data)
}

// TODO: make this return an error instead
/// Panics if the locator accepts a node.
pub fn insert_by_locator<D : Data, L, TR> (tree : TR, locator : L, value : D::Value)
    -> TR::Walker where
    TR : SomeTreeRef<D>,
    TR::Walker : ModifiableWalker<D>,
    L : Locator<D>,
    //<A as data::Data>::Value : std::fmt::Debug,
{
    let mut walker = search_by_locator(tree, locator);
    walker.insert(value).expect("tried to insert into an existing node"); // TODO
    walker
}

/// Finds any node by key.
/// If there isn't any, it finds the empty location where that node would be instead.
/// Returns a walker at the wanted position.
pub fn search_walker<W, D : Data>(walker : &mut W, key : &<<D as Data>::Value as Keyed>::Key) where
    W : SomeWalker<D>,
    D : Data,
    D::Value : crate::data::example_data::Keyed,
    //<A as data::Data>::Value : std::fmt::Debug,
{
    search_walker_by_locator(walker, &locate_by_key::<D>(key));
}

/// Finds any node by key.
/// If there isn't any, it finds the empty location where that node would be instead.
/// Returns a walker at the wanted position.
pub fn search<TR, D : Data>(tree : TR, key : &<<D as Data>::Value as Keyed>::Key) -> TR::Walker where
    TR : SomeTreeRef<D>,
    D : Data,
    D::Value : crate::data::example_data::Keyed,
    //<A as data::Data>::Value : std::fmt::Debug,
{
    let mut walker = tree.walker();
    search_walker(&mut walker, key);
    return walker;
}

/// Finds any node that the locator `Accept`s.
/// If there isn't any, it finds the empty location where that node would be instead.
/// Returns a walker at the wanted position.
pub fn search_walker_by_locator<W, D : Data, L>(walker : &mut W, locator : L) where
    W : crate::trees::SomeWalker<D>,
    L : Locator<D>,
{
    use LocResult::*;

    while let Some(res) = walker_locate(walker, locator) {
        match res {
            Accept => break,
            GoRight => walker.go_right().unwrap(),
            GoLeft => walker.go_left().unwrap(),
        };
    }
}

/// Finds any node that the locator `Accept`s.
/// If there isn't any, it finds the empty location where that node would be instead.
/// Returns a walker at the wanted position.
pub fn search_by_locator<TR, D : Data, L>(tree : TR, locator : L) -> TR::Walker where
    TR : crate::trees::SomeTreeRef<D>,
    L : Locator<D>,
{
    let mut walker = tree.walker();
    search_walker_by_locator(&mut walker, locator);
    return walker;
}

/// Returns the accumulated values on the locator's segment
/// Do not use with splay trees - it might mess up the complexity,
/// because it uses go_up().
/// 
/// Instead, use [`SomeTree::segment_summary`]
pub fn segment_summary<TR, L, D : Data>(tree : TR, locator : L) -> 
        D::Summary where
    TR : SomeTreeRef<D>,
    L : Locator<D>,
{
    use LocResult::*;

    let mut walker = tree.walker();
    while let Some(res) = walker_locate(&mut walker, locator) {
        match res {
            GoRight => walker.go_right().unwrap(),
            GoLeft => walker.go_left().unwrap(),

            // at this point, we split into the two sides
            Accept => {
                let node_value = walker.node_summary();
                let depth = walker.depth();
                walker.go_left().unwrap();
                let first_half = segment_summary_on_suffix(&mut walker, locator);
                // get back to the original node
                for _ in 0..walker.depth() - depth {
                    walker.go_up().unwrap();
                }
                walker.go_right().unwrap();
                let second_half = segment_summary_on_prefix(&mut walker, locator);

                return first_half + node_value + second_half;
            },
        }
    }

    // empty segment case
    Default::default()
}

fn segment_summary_on_suffix<W, L, A : Data>(walker : &mut W, locator : L) ->
       A::Summary where
    W : SomeWalker<A>,
    L : Locator<A>,
{
    let mut res = Default::default();
    use LocResult::*;

    while let Some(dir) = walker_locate(walker, locator) {
        match dir {
            Accept => {
                res = walker.node_summary() + walker.right_subtree_summary().unwrap() + res;
                walker.go_left().unwrap();
            },
            GoRight => walker.go_right().unwrap(),
            GoLeft => panic!("inconsistent locator"),
        }
    }

    res
}

fn segment_summary_on_prefix<W, L, A : Data>(walker : &mut W, locator : L) ->
        A::Summary where
    W : SomeWalker<A>,
    L : Locator<A>,
{
    let mut res = Default::default();
    use LocResult::*;

    while let Some(dir) = walker_locate(walker, locator) {
        match dir {
            Accept => {
                res = res + walker.left_subtree_summary().unwrap() + walker.node_summary();
                walker.go_right().unwrap();
            },
            GoRight => panic!("inconsistent locator"),
            GoLeft => walker.go_left().unwrap(), 
        }
    }

    res
}

/// Returns the accumulated values on the locator's segment
/// Do not use with splay trees - it might mess up the complexity,
/// because it uses go_up().
/// 
/// Instead, use [`SomeTree::act_segment`]
pub fn act_segment<TR, L, D : Data>(tree : TR, action : D::Action, locator : L) where
    TR : SomeTreeRef<D>,
    L : Locator<D>,
{
    assert!(action.to_reverse() == false);
    use LocResult::*;

    let mut walker = tree.walker();
    while let Some(res) = walker_locate(&mut walker, locator) {
        match res {
            GoRight => walker.go_right().unwrap(),
            GoLeft => walker.go_left().unwrap(),

            // at this point, we split into the two sides
            Accept => {
                walker.act_node(action);
                let depth = walker.depth();
                walker.go_left().unwrap();
                act_on_suffix(&mut walker, action, locator);
                // get back to the original node
                for _ in 0..walker.depth() - depth {
                    walker.go_up().unwrap();
                }
                walker.go_right().unwrap();
                act_on_prefix(&mut walker, action, locator);
                return;
            },
        }
    }
}

// Only works if `action.to_reverse()` is false. does not check.
fn act_on_suffix<W, L, D : Data>(walker : &mut W, action : D::Action, locator : L) where
    W : SomeWalker<D>,
    L : Locator<D>,
{
    use LocResult::*;

    while let Some(dir) = walker_locate(walker, locator) {
        match dir {
            Accept => {
                walker.act_node(action);
                walker.act_right_subtree(action).unwrap();
                walker.go_left().unwrap();
            },
            GoRight => walker.go_right().unwrap(),
            GoLeft => panic!("inconsistent locator"),
        }
    }
}


// Only works if `action.to_reverse()` is false. does not check.
fn act_on_prefix<W, L, D : Data>(walker : &mut W, action : D::Action, locator : L) where
    W : SomeWalker<D>,
    L : Locator<D>,
{
    use LocResult::*;

    while let Some(dir) = walker_locate(walker, locator) {
        match dir {
            Accept => {
                walker.act_node(action);
                walker.act_left_subtree(action).unwrap();
                walker.go_right().unwrap();
            },
            GoRight => panic!("inconsistent locator"),
            GoLeft => walker.go_left().unwrap(), 
        }
    }
}

