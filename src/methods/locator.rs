/// Module defining a trait for a value that locates a node or group of nodes in a tree.
/// A locator, given a node, can either reply, `LeftOfInterval`, `RightOfInterval`, `Accept`,
/// or return an error value.
/// 
/// Locators are supposed to represent a sub interval of the tree. It is supposed to do this:
/// * If the current node is to the left of the segment, return `LeftOfInterval`.
/// * If the current node is to the right of the segment, return `RightOfInterval`
/// * If the current node is part of the segment, return `Accept`.
/// 
/// Functions like search, which logically expect only one accepted node, and not a segment,
/// will use any node that is accepted.
/// 
/// Note that the locator is allowed to mutate a state while being asked questions.
/// Therefore, in order for a function to go both to the left edge and the right edge
/// of the interval, the locator has to implement `Clone`. Thus, some functions require the
/// locator to implement Clone.
/// 
/// Anonymous functions of the type `FnMut(&D) -> Result<LocResult, Err>` can be used as locators.

use crate::trees::basic_tree::*;

pub enum LocResult {
    Accept, LeftOfInterval, RightOfInterval,
}
use LocResult::*;

pub trait Locator<D> {
    type Error;
    fn locate(&mut self, node : &BasicNode<D>) -> Result<LocResult, Self::Error>;
}

// TODO: immutable locator trait?
// this might be needed. but might not.

impl<D, E, F : FnMut(&BasicNode<D>) -> Result<LocResult, E>> Locator<D> for F {
    type Error = E;
    fn locate(&mut self, node : &BasicNode<D>) -> Result<LocResult, E> {
        self(node)
    }
}


/// Locator for finding an element using a key.
/// Returns `Fn` instead of `&Fn` because the caller has to own this locator.
/// We could return a manually-made owned locator instead. however, we don't.
pub fn locate_by_key<'a, D>(key : &'a <D as crate::data::example_data::Keyed>::Key) -> 
    impl Fn(&BasicNode<D>) -> Result<LocResult, void::Void> + 'a where
    D : crate::data::example_data::Keyed,
{
    move |node : &BasicNode<D>| -> Result<LocResult, void::Void> {
        use std::cmp::Ordering::*;
        let res = match node.get_key().cmp(key) {
            Equal => Accept,
            Less => RightOfInterval,
            Greater => LeftOfInterval,
        };
        Ok(res)
    }
}

// TODO: Splitter. locators that can't `Accept`. used for splitting trees
// and for insertions.

#[derive(Clone)]
pub struct IndexLocator {
    low : usize,
    high : usize,
}

impl IndexLocator {
    pub fn expose(self) -> (usize, usize) {
        (self.low, self.high)
    }
}

// TODO: IndexLocator can't really go to two sides, since when it returns accept, it doesn't
// mutate its state the right way for it to continue...
impl<D : example_data::SizedData> Locator<D> for IndexLocator {
    type Error = void::Void;
    fn locate(&mut self, node : &BasicNode<D>) -> Result<LocResult, void::Void> {
        let s = match &node.left {
            BasicTree::Empty => 0,
            BasicTree::Root(node) => node.size(),
        };

        let res = if self.high <= s {
            RightOfInterval
        } else if s + node.size() <= self.low {
            self.low -= s + node.size();
            self.high -= s + node.size();
            LeftOfInterval
        } else {
            self.low -= s;
            self.high -= s;
            Accept
        };
        return Ok(res)
    }
}

// TODO: range by keyes, inclusive and exclusive on both sides, etc.
// also, versions for inserting with duplicate keys, that skip any elements that have the same key.

/// Returns the locator corresponding to the interval [a, b).
/// With regards to wide nodes: a node is considered accepted if its interval intersects
/// [a, b).
pub fn locate_by_index_range(low : usize, high : usize) -> 
    IndexLocator {
    IndexLocator {low, high}
}

pub fn locate_by_index(index : usize) -> 
    IndexLocator {
    locate_by_index_range(index, index+1)
}