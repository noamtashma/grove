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




// TODO - make this work for both filled and empty starting positions
// TODO - figure out how to make this callable like walker.next_empty()
/// if the walker is at an empty position, return an error.
/// goes to the next empty position
pub fn next_empty<W : SomeWalker<A>, A : Data>(walker : &mut W) -> Result<(), ()> {
    walker.go_right()?; // if we're at an empty node, return error
    while !walker.is_empty() {
        walker.go_left().unwrap();
    }
    Ok(())
}

// if the walker is at an empty position, return an error.
// goes to the previous empty position
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

/// returns a vector of all the values in the tree.
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
        if let trees::basic_tree::BasicTree::Root(node) = walker.inner_mut() {
            res.push(node.node_value().clone());
        } else {panic!()}
    }
    res
}

/// Panics if a key was reused.
/// TODO: make this return an error.
pub fn insert_by_key<A : Data, TR>(tree : TR, data : A::Value)
    -> TR::Walker where
    TR : SomeTreeRef<A>,
    A::Value : crate::data::example_data::Keyed,
    //<A as data::Data>::Value : std::fmt::Debug,
{
    let res : Result<TR::Walker, void::Void> =
        insert_by_locator(tree, &locate_by_key(&data.get_key()) , data);
    match res {
        Ok(walker) => walker,
        Err(void ) => match void {}
    }
}

/// Panics if the locator accepts a node.
/// TODO: make this return an error instead
pub fn insert_by_locator<A : Data, L, TR> (tree : TR, locator : &L, value : A::Value)
    -> Result<TR::Walker, L::Error> where
    TR : SomeTreeRef<A>,
    L : Locator<A>,
    //<A as data::Data>::Value : std::fmt::Debug,
{
    let mut walker = search_by_locator(tree, locator)?;
    walker.insert_new(value).expect("tried to insert into an existing node"); // TODO
    Ok(walker)
}

// TODO: a function that creates a perfectly balanced tree,
// given the input nodes.

pub fn search<TR, A : Data>(tree : TR, key : &<<A as Data>::Value as Keyed>::Key) ->  TR::Walker where
    TR : SomeTreeRef<A>,
    A : Data,
    A::Value : crate::data::example_data::Keyed,
    //<A as data::Data>::Value : std::fmt::Debug,
{
    let res : Result<_, void::Void> = search_by_locator(tree, &locate_by_key(key));
    match res {
        Ok(walker) => walker,
        Err(void) => match void {}
    }
}

/// Finds any node that the locator `Accept`s.
/// If there isn't any, it find the empty location the locator has navigated it to.
/// Returns an Err if the Locator has returned an Err.
pub fn search_by_locator<TR, A : Data, L>(tree : TR, locator : &L)
    -> Result<TR::Walker, L::Error> where
    TR : crate::trees::SomeTreeRef<A>,
    L : Locator<A>,
    //<A as data::Data>::Value : std::fmt::Debug,
{
    use LocResult::*;

    let mut walker = tree.walker();
    while let Some(res) = walker_locate(&mut walker, locator) {
        match res? {
            Accept => break,
            GoRight => walker.go_right().unwrap(),
            GoLeft => walker.go_left().unwrap(),
        };
    }
    return Ok(walker);
}

/// Returns the accumulated values on the locator's segment
/// Do not use with splay trees - it might mess up the complexity,
/// because it uses go_up().
/// 
/// Instead, use `segment_value()`
pub fn accumulate_values<TR, L, A : Data>(tree : TR, locator : &L) -> 
        Result<A::Summary, L::Error> where
    TR : SomeTreeRef<A>,
    L : Locator<A>,
{
    use LocResult::*;

    let mut walker = tree.walker();
    while let Some(res) = walker_locate(&mut walker, locator) {
        match res? {
            GoRight => walker.go_right().unwrap(),
            GoLeft => walker.go_right().unwrap(),

            // at this point, we split into the two sides
            Accept => {
                let node_value = walker.node_summary();
                let depth = walker.depth();
                walker.go_left().unwrap();
                let prefix = accumulate_values_on_prefix(&mut walker, locator)?;
                // get back to the original node
                for _ in 0..walker.depth() - depth {
                    walker.go_up().unwrap();
                }
                walker.go_right().unwrap();
                let suffix = accumulate_values_on_suffix(walker, locator)?;

                return Ok(A::compose_s(prefix, A::compose_s(node_value, suffix)));
            },
        }
    }

    // empty segment case
    Ok(A::EMPTY)
}

fn accumulate_values_on_suffix<W, L, A : Data>(mut walker : W, locator : &L) ->
        Result<A::Summary, L::Error> where
    W : SomeWalker<A>,
    L : Locator<A>,
{
    let mut res = A::EMPTY;
    use LocResult::*;

    while let Some(dir) = walker_locate(&mut walker, locator) {
        match dir? {
            Accept => {
                if let basic_tree::BasicTree::Root(node) = walker.inner_mut() {
                    res = A::compose_s(node.right.subtree_summary(), res);
                    res = A::compose_s(node.node_summary(), res);
                    walker.go_left().unwrap();
                } else {panic!()}
            },
            GoRight => walker.go_right().unwrap(),
            GoLeft => panic!("inconsistent locator"),
        }
    }

    Ok(res)
}

fn accumulate_values_on_prefix<W, L, A : Data>(walker : &mut W, locator : &L) ->
        Result<A::Summary, L::Error> where
    W : SomeWalker<A>,
    L : Locator<A>,
{
    let mut res = A::EMPTY;
    use LocResult::*;

    while let Some(dir) = walker_locate(walker, locator) {
        match dir? {    Accept => {
                if let basic_tree::BasicTree::Root(node) = walker.inner_mut() {
                    res = A::compose_s(res, node.left.subtree_summary());
                    res = A::compose_s(res, node.node_summary());
                    walker.go_right().unwrap();
                } else {
                    panic!();
                }
            },
            GoRight => panic!("inconsistent locator"),
            GoLeft => walker.go_left().unwrap(), 
        }
    }

    Ok(res)
}

// TODO:
// apply action on segment,
// apply action on prefix,
// apply action on suffix