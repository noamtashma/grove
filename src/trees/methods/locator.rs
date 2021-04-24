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

use crate::*;

#[derive(PartialEq, Eq, Debug)]
pub enum LocResult {
    Accept, GoRight, GoLeft,
}
use LocResult::*;

pub fn all<D : Data>(_left : D::Summary, _val : &D::Value, _right : D::Summary) -> Result<LocResult, void::Void> {
    Ok(Accept)
}

pub trait Locator<A : Data> {
    type Error;
    fn locate(&self, left : A::Summary, node : &A::Value, right : A::Summary) -> Result<LocResult, Self::Error>;
}

impl<A : Data, E, F : Fn(A::Summary, &A::Value, A::Summary) -> Result<LocResult, E>> Locator<A> for F {
    type Error = E;
    fn locate(&self, left : A::Summary, node : &A::Value, right : A::Summary) -> Result<LocResult, E> {
        self(left, node, right)
    }
}

/// Returns the result of the locator at the walker
/// Returns None if the walker is at an empty position
pub fn walker_locate<W, A : Data, L> (walker : &mut W, locator : &L) -> Option<Result<LocResult, L::Error>> where
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
    A : Data,
    A::Value : crate::data::example_data::Keyed,
{
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

/// represents the segment of indices [low, high)
#[derive(Clone)]
pub struct IndexLocator {
    pub low : usize,
    pub high : usize,
}


impl<A : Data + SizedData> Locator<A> for IndexLocator {
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


/// A Wrapper for other locators what will find exactly the left edge
/// of the previous locator. So, this is always a splitting locator.
pub struct LeftEdgeLocator<L> (
    pub L,
);
/// A Wrapper for other locators what will find exactly the right edge
/// of the previous locator. So, this is always a splitting locator.
pub struct RightEdgeLocator<L> (
    pub L,
);

impl<A : Data, L : Locator<A>> Locator<A> for LeftEdgeLocator<L> {
    type Error = L::Error;
    fn locate(&self, left : A::Summary, node : &A::Value, right : A::Summary) -> 
        Result<LocResult, L::Error>
    {
        Ok(match self.0.locate(left, node, right)? {
            Accept => GoLeft,
            res => res,
        })
    }
}

impl<A : Data, L : Locator<A>> Locator<A> for RightEdgeLocator<L> {
    type Error = L::Error;
    fn locate(&self, left : A::Summary, node : &A::Value, right : A::Summary) -> 
        Result<LocResult, L::Error>
    {
        Ok(match self.0.locate(left, node, right)? {
            Accept => GoRight,
            res => res,
        })
    }
}

/// A Wrapper for two other locators, that finds the smallest segment containing both of them.
/// For example, the Union of ranges `[3,6)` and `[7,12)` will  be `[3,12)`.
pub struct UnionLocator<L> (
    pub L, pub L
);

impl<A : Data, L : Locator<A>> Locator<A> for UnionLocator<L> {
    type Error = L::Error;
    fn locate(&self, left : A::Summary, node : &A::Value, right : A::Summary) -> 
        Result<LocResult, L::Error>
    {
        let a = self.0.locate(left, node, right)?;
        let b = self.1.locate(left, node, right)?;
        Ok(if a == b {a} else { Accept })
    }
}