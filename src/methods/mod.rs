pub mod locator;
pub use locator::*;

use crate::*;
/// Panics if a key was reused.
/// TODO: make this return an error.
pub fn insert<D : Data, TR>(tree : TR, data : D)
    -> TR::Walker where
    TR : SomeTreeRef<D>,
    D : crate::data::example_data::Keyed,
    D::Key : Clone, // this isn't really needed. it's just needed temporarily because of stuff.
{
    let res : Result<TR::Walker, void::Void> =
        insert_by_locator(tree, &mut locate_by_key(&data.get_key().clone()) , data);
    match res {
        Ok(walker) => walker,
        Err(void ) => match void {}
    }
}

/// Panics if the locator accepts a node.
/// TODO: make this return an error instead
pub fn insert_by_locator<D : Data, L, TR> (tree : TR, locator : &mut L, data : D)
    -> Result<TR::Walker, L::Error> where
    TR : SomeTreeRef<D>,
    L : Locator<D>,
{
    let mut walker = search_by_locator(tree, locator)?;
    walker.insert_new(data).expect("tried to insert into an existing node"); // TODO
    Ok(walker)
}

// TODO: a function that creates a perfectly balanced tree,
// given the input nodes.


pub fn search<TR, D>(tree : TR, key : &D::Key) ->  TR::Walker where
    TR : SomeTreeRef<D>,
    D : crate::data::example_data::Keyed, {
    let res : Result<_, void::Void> = search_by_locator(tree, &mut locate_by_key(key));
    match res {
        Ok(walker) => walker,
        Err(void) => match void {}
    }
}

/// Finds any node that the locator `Accept`s.
/// If there isn't any, it find the empty location the locator has navigated it to.
/// Returns an Err if the Locator has returned an Err.
pub fn search_by_locator<TR, D : Data, L>(tree : TR, locator : &mut L)
    -> Result<TR::Walker, L::Error> where
    TR : crate::trees::SomeTreeRef<D>,
    L : Locator<D>,
{
    use LocResult::*;

    let mut walker = tree.walker();
    while let basic_tree::BasicTree::Root(node) = walker.inner() {
        match locator.locate(node)? {
            Accept => (),
            LeftOfInterval => walker.go_right().unwrap(),
            RightOfInterval => walker.go_left().unwrap(),
        }
    }
    return Ok(walker);
}