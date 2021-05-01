//! Methods module
//! This module provides generic methods for use on general trees. for example,
//! search functions, querying on a segment of the tree, applying an
//! action on a segment of the tree, and so on.
//!
//! The locator module provides an interface for locating a specific value
//! or a segment, generalizing the search in a binary search tree.
//!
//! Since different balanced tree algorithms are different, the generic functions
//! may not work as intended. For example, splay trees shouldn't use the `go_up` method too much,
//! and so some generic functions which use `go_up` may have linear complexity when used with
//! splay trees.

pub mod locator;

pub use locator::*;
use data::example_data::Keyed;

use crate::*;
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
pub fn insert_by_locator<D : Data, L, TR> (tree : TR, locator : &L, value : D::Value)
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
pub fn search<TR, A : Data>(tree : TR, key : &<<A as Data>::Value as Keyed>::Key) ->  TR::Walker where
    TR : SomeTreeRef<A>,
    A : Data,
    A::Value : crate::data::example_data::Keyed,
    //<A as data::Data>::Value : std::fmt::Debug,
{
    search_by_locator(tree, &locate_by_key::<A>(key))
}

/// Finds any node that the locator `Accept`s.
/// If there isn't any, it finds the empty location where that node would be instead.
/// Returns a walker at the wanted position.
pub fn search_by_locator<TR, A : Data, L>(tree : TR, locator : &L)
    -> TR::Walker where
    TR : crate::trees::SomeTreeRef<A>,
    L : Locator<A>,
{
    use LocResult::*;

    let mut walker = tree.walker();
    while let Some(res) = walker_locate(&mut walker, locator) {
        match res {
            Accept => break,
            GoRight => walker.go_right().unwrap(),
            GoLeft => walker.go_left().unwrap(),
        };
    }
    return walker;
}

/// Returns the accumulated values on the locator's segment
/// Do not use with splay trees - it might mess up the complexity,
/// because it uses go_up().
/// 
/// Instead, use `segment_value()`
pub fn accumulate_values<TR, L, A : Data>(tree : TR, locator : &L) -> 
        A::Summary where
    TR : SomeTreeRef<A>,
    L : Locator<A>,
{
    use LocResult::*;

    let mut walker = tree.walker();
    while let Some(res) = walker_locate(&mut walker, locator) {
        match res {
            GoRight => walker.go_right().unwrap(),
            GoLeft => walker.go_right().unwrap(),

            // at this point, we split into the two sides
            Accept => {
                let node_value = walker.node_summary();
                let depth = walker.depth();
                walker.go_left().unwrap();
                let prefix = accumulate_values_on_prefix(&mut walker, locator);
                // get back to the original node
                for _ in 0..walker.depth() - depth {
                    walker.go_up().unwrap();
                }
                walker.go_right().unwrap();
                let suffix = accumulate_values_on_suffix(walker, locator);

                return prefix + node_value + suffix;
            },
        }
    }

    // empty segment case
    A::EMPTY
}

fn accumulate_values_on_suffix<W, L, A : Data>(mut walker : W, locator : &L) ->
       A::Summary where
    W : SomeWalker<A>,
    L : Locator<A>,
{
    let mut res = A::EMPTY;
    use LocResult::*;

    while let Some(dir) = walker_locate(&mut walker, locator) {
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

fn accumulate_values_on_prefix<W, L, A : Data>(walker : &mut W, locator : &L) ->
        A::Summary where
    W : SomeWalker<A>,
    L : Locator<A>,
{
    let mut res = A::EMPTY;
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

// TODO:
// apply action on segment,
// apply action on prefix,
// apply action on suffix