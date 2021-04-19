//! Module defining a trait for a way to locatee a node or group of nodes in a tree.
//! 
//! Locators are supposed to represent a segment of the tree.
//! When the locator is used, whenever we encounter a node, the locator is queried.
//! The locator has to reply:
//! * If the current node is to the left of the segment, return `GoRight`.
//! * If the current node is to the right of the segment, return `GoLeft`
//! * If the current node is part of the segment, return `Accept`.
//! Note that the subtree of the current node is irrelevant: only the current node matters.
//!
//! Functions like search, which logically expect only one accepted node, and not a segment,
//! will use any node that is accepted.
//! Functions like insertions, will expect a locator that doesn't accept any node,
//! but leads the locator into a space between nodes, where the node will be inserted.
//! 
//! The locator receives as input the current node,
//! the accumulated value left of the subtree of the current node,
//! and the accumulated value right of the current node.
//!
//! Locators are immutable, and therefore it is assumed that they can be called in any order,
//! i.e., earlier calls will not change the result of later calls. This is even though
//! that might not be the case, using interior mutability.
//! 
//! Anonymous functions of the type `Fn(...) -> Result<LocResult, Err>` can be used as locators.

use crate::trees::basic_tree::*;

pub enum LocResult {
    Accept, GoRight, GoLeft,
}
use LocResult::*;

pub trait Locator<A : Action> {
    type Error;
    fn locate(&self, left : A::Summary, node : &A::Value, right : A::Summary) -> Result<LocResult, Self::Error>;
}

impl<A : Action, E, F : Fn(A::Summary, &A::Value, A::Summary) -> Result<LocResult, E>> Locator<A> for F {
    type Error = E;
    fn locate(&self, left : A::Summary, node : &A::Value, right : A::Summary) -> Result<LocResult, E> {
        self(left, node, right)
    }
}

/// Returns the result of the locator at the walker
/// Returns None if the walker is at an empty position
pub fn walker_locate<W, A : Action, L> (walker : &mut W, locator : &L) -> Option<Result<LocResult, L::Error>> where
    W : crate::trees::SomeWalker<A>,
    L : Locator<A>,
{
    if let Some(value) = walker.value() {
        let left = walker.left_summary();
        let right = walker.right_summary();
        Some(locator.locate(left, value, right))
    } else {
        None
    }
}



/// Locator for finding an element using a key.
pub fn locate_by_key<'a, A>(key : &'a <A::Value as crate::data::example_data::Keyed>::Key) -> 
    impl Fn(A::Summary, &A::Value, A::Summary) -> Result<LocResult, void::Void> + 'a where
    A : Action,
    A::Value : crate::data::example_data::Keyed,
{
    use crate::data::example_data::Keyed;
    move |_, node : &A::Value, _| -> Result<LocResult, void::Void> {
        use std::cmp::Ordering::*;
        let res = match node.get_key().cmp(key) {
            Equal => Accept,
            Less => GoLeft,
            Greater => GoRight,
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

/// represents the segment of indices [low, high)
impl IndexLocator {
    pub fn expose(self) -> (usize, usize) {
        (self.low, self.high)
    }
}


impl<A : Action + example_data::SizedAction> Locator<A> for IndexLocator {
    type Error = void::Void;
    fn locate(&self, left : A::Summary, node : &A::Value, _right : A::Summary) -> Result<LocResult, void::Void> {
        // find the index of the current node
        let s = A::size(left);

        let res = if s >= self.high {
            GoLeft
        } else if s + A::size(A::to_summary(node)) <= self.low {
            GoRight
        } else {
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