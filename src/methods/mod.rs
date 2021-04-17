pub mod locator;
pub use locator::*;

use crate::*;




// TODO - figure out how to make this callable like walker.next_empty()
/// if the walker is at an empty position, return an error.
/// goes to the next empty position
pub fn next_empty<W : SomeWalker<D>, D : Action>(walker : &mut W) -> Result<(), ()> {
    walker.go_right()?; // if we're at an empty node, return error
    while !walker.is_empty() {
        walker.go_left().unwrap();
    }
    Ok(())
}

// if the walker is at an empty position, return an error.
// goes to the previous empty position
pub fn previous_empty<W : SomeWalker<D>, D : Action>(walker : &mut W) -> Result<(), ()> {
    walker.go_left()?; // if we're at an empty node, return error
    while !walker.is_empty() {
        walker.go_right().unwrap();
    }
    Ok(())
}

/// Panics if a key was reused.
/// TODO: make this return an error.
pub fn insert_by_key<A : Action, TR>(tree : TR, data : A::Value)
    -> TR::Walker where
    TR : SomeTreeRef<A>,
    A : crate::data::example_data::Keyed,
    A::Key : Clone, // this isn't really needed. it's just needed temporarily because of stuff.
    //<A as data::Action>::Value : std::fmt::Debug,
{
    let res : Result<TR::Walker, void::Void> =
        insert_by_locator(tree, &locate_by_key(&A::get_key(data)) , data);
    match res {
        Ok(walker) => walker,
        Err(void ) => match void {}
    }
}

/// Panics if the locator accepts a node.
/// TODO: make this return an error instead
pub fn insert_by_locator<A : Action, L, TR> (tree : TR, locator : &L, value : A::Value)
    -> Result<TR::Walker, L::Error> where
    TR : SomeTreeRef<A>,
    L : Locator<A>,
    //<A as data::Action>::Value : std::fmt::Debug,
{
    let mut walker = search_by_locator(tree, locator)?;
    walker.insert_new(value).expect("tried to insert into an existing node"); // TODO
    Ok(walker)
}

// TODO: a function that creates a perfectly balanced tree,
// given the input nodes.


pub fn search<TR, D : Action>(tree : TR, key : &D::Key) ->  TR::Walker where
    TR : SomeTreeRef<D>,
    D : crate::data::example_data::Keyed,
    //<D as data::Action>::Value : std::fmt::Debug,
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
pub fn search_by_locator<TR, D : Action, L>(tree : TR, locator : &L)
    -> Result<TR::Walker, L::Error> where
    TR : crate::trees::SomeTreeRef<D>,
    L : Locator<D>,
    //<D as data::Action>::Value : std::fmt::Debug,
{
    use LocResult::*;

    let mut walker = tree.walker();
    while let basic_tree::BasicTree::Root(node) = walker.inner() {
        match locator.locate(node, walker.far_left_value(), walker.far_right_value())? {
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
/// TODO - find an alternative
pub fn accumulate_values<TR, L, A : Action>(tree : TR, locator : &L) -> 
        Result<A::Value, L::Error> where
    TR : SomeTreeRef<A>,
    L : Locator<A>,
{
    use LocResult::*;

    let mut walker = tree.walker();
    while let basic_tree::BasicTree::Root(node) = walker.inner() {
        match locator.locate(node, walker.far_left_value(), walker.far_right_value())? {
            GoRight => walker.go_right().unwrap(),
            GoLeft => walker.go_right().unwrap(),

            // at this point, we split into the two sides
            Accept => {
                let node_value = node.node_value();
                let depth = walker.depth();
                walker.go_left().unwrap();
                let prefix = accumulate_values_on_prefix(&mut walker, locator)?;
                // get back to the original node
                for _ in 0..walker.depth() - depth {
                    walker.go_up().unwrap();
                }
                walker.go_right().unwrap();
                let suffix = accumulate_values_on_suffix(&mut walker, locator)?;

                return Ok(A::compose_v(prefix, A::compose_v(node_value, suffix)));
            },
        }
    }

    // empty segment case
    Ok(A::EMPTY)
}

fn accumulate_values_on_suffix<W, L, A : Action>(walker : &mut W, locator : &L) ->
        Result<A::Value, L::Error> where
    W : SomeWalker<A>,
    L : Locator<A>,
{
    let mut res = A::EMPTY;
    use LocResult::*;

    while let basic_tree::BasicTree::Root(node) = walker.inner() {
        match locator.locate(node, walker.far_left_value(), walker.far_right_value())? {
            Accept => {
                res = A::compose_v(node.right.segment_value(), res);
                res = A::compose_v(node.node_value(), res);
                walker.go_left().unwrap();
            },
            GoRight => walker.go_right().unwrap(),
            GoLeft => panic!("inconsistent locator"),
        }
    }

    Ok(res)
}

fn accumulate_values_on_prefix<W, L, A : Action>(walker : &mut W, locator : &L) ->
        Result<A::Value, L::Error> where
    W : SomeWalker<A>,
    L : Locator<A>,
{
    let mut res = A::EMPTY;
    use LocResult::*;

    while let basic_tree::BasicTree::Root(node) = walker.inner() {
        match locator.locate(node, walker.far_left_value(), walker.far_right_value())? {
            Accept => {
                res = A::compose_v(res, node.left.segment_value());
                res = A::compose_v(res, node.node_value());
                walker.go_right().unwrap();
            },
            GoRight => panic!("inconsistent locator"),
            GoLeft => walker.go_left().unwrap(), 
        }
    }

    Ok(res)
}