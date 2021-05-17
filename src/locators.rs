//! The locator module provides an interface for locating a specific value
//! or a segment, generalizing the search in a binary search tree.
//! 
//! Locators are supposed to represent a segment of the tree. See [`Locator`].
//!
//! Functions like search, which logically expect only one accepted node, and not a segment,
//! will use any node that is accepted.
//! Functions like insertions, will expect a locator that doesn't accept any node,
//! but leads the locator into a space between nodes, where the node will be inserted.


use crate::*;

#[derive(PartialEq, Eq, Debug)]
pub enum LocResult {
    Accept, GoRight, GoLeft,
}
use LocResult::*;


/// Locators are type that represent a segment of the tree.
/// When the locator is used, we query the locator about the current node.
/// The locator has to reply:
/// * If the current node is to the left of the segment, return `GoRight`.
/// * If the current node is to the right of the segment, return `GoLeft`
/// * If the current node is part of the segment, return `Accept`.
///
/// In each query, the locator receives as input the current node's value,
/// the accumulated summary left of the current node,
/// and the accumulated summary right of the current node.
/// Note that the subtree of the current node is irrelevant: only the current node's value matters.
///
/// References to anonymous functions of the type `Fn(...) -> LocResult` can be used as locators.
///
/// Locators are immutable, and therefore it is assumed that they can be called in any order,
/// i.e., earlier calls will not change the result of later calls. This is even though
/// that might not be the case, using interior mutability.
/// Locators must be [`Clone`], in order for usage to be comfortable. This can always be achieved
/// by taking a reference.
pub trait Locator<D : Data> : Clone {
    fn locate(&self, left : D::Summary, node : &D::Value, right : D::Summary) -> LocResult;
}

impl<D : Data, F> Locator<D> for F where
    F : Fn(D::Summary, &D::Value, D::Summary) -> LocResult + Clone
{
    fn locate(&self, left : D::Summary, node : &D::Value, right : D::Summary) -> LocResult {
        self(left, node, right)
    }
}

/// Returns the result of the locator at the walker
/// Returns None if the walker is at an empty position
pub fn walker_locate<W, D : Data, L> (walker : &mut W, locator : &L) -> Option<LocResult> where
    W : crate::trees::SomeWalker<D>,
    L : Locator<D>,
{
    if let Some(value) = walker.value() {
        let left = walker.left_summary();
        let right = walker.right_summary();
        Some(locator.locate(left, value, right))
    } else {
        None
    }
}


// TODO: Splitter. locators that can't `Accept`. used for splitting trees
// and for insertions.


/// Locator instance for `(usize,)` representing a single index.
impl<D : Data> Locator<D> for (usize,) where D::Summary : SizedSummary {
    fn locate(&self, left : D::Summary, node : &D::Value, _right : D::Summary) -> LocResult {
        // find the index of the current node
        let s = left.size();

        if s > self.0 {
            GoLeft
        } else if s + D::to_summary(node).size() <= self.0 {
            GoRight
        } else {
            Accept
        }
    }
}

/// Locator instance for [`std::ops::RangeFull`].
impl<D : Data> Locator<D> for std::ops::RangeFull {
    fn locate(&self, _left : D::Summary, _node : &D::Value, _right : D::Summary) -> LocResult {
        Accept
    }
}

/// Locator instance for a reference to [`std::ops::RangeFull`].
impl<D : Data> Locator<D> for &std::ops::RangeFull {
    fn locate(&self, _left : D::Summary, _node : &D::Value, _right : D::Summary) -> LocResult {
        Accept
    }
}

/// Locator instance for [`std::ops::Range<usize>`] representing an index range.
impl<D : Data> Locator<D> for std::ops::Range<usize> where D::Summary : SizedSummary {
    fn locate(&self, left : D::Summary, node : &D::Value, _right : D::Summary) -> LocResult {
        // find the index of the current node
        let s = left.size();

        if s >= self.end {
            GoLeft
        } else if s + D::to_summary(node).size() <= self.start {
            GoRight
        } else {
            Accept
        }
    }
}

/// Locator instance for a reference to [`std::ops::Range<usize>`] representing an index range.
impl<D : Data> Locator<D> for &std::ops::Range<usize> where D::Summary : SizedSummary {
    fn locate(&self, left : D::Summary, node : &D::Value, _right : D::Summary) -> LocResult {
        // find the index of the current node
        let s = left.size();

        if s >= self.end {
            GoLeft
        } else if s + D::to_summary(node).size() <= self.start {
            GoRight
        } else {
            Accept
        }
    }
}

/// Locator instance for [`std::ops::RangeInclusive<usize>`] representing an index range.
/// Do not use with ranges that have been iterated on to exhaustion.
impl<D : Data> Locator<D> for std::ops::RangeInclusive<usize> where D::Summary : SizedSummary {
    fn locate(&self, left : D::Summary, node : &D::Value, _right : D::Summary) -> LocResult {
        // find the index of the current node
        let s = left.size();

        if s > *self.end() {
            GoLeft
        } else if s + D::to_summary(node).size() <= *self.start() {
            GoRight
        } else {
            Accept
        }
    }
}

/// Locator instance fora reference to [`std::ops::RangeInclusive<usize>`] representing an index range.
/// Do not use with ranges that have been iterated on to exhaustion.
impl<D : Data> Locator<D> for &std::ops::RangeInclusive<usize> where D::Summary : SizedSummary {
    fn locate(&self, left : D::Summary, node : &D::Value, _right : D::Summary) -> LocResult {
        // find the index of the current node
        let s = left.size();

        if s > *self.end() {
            GoLeft
        } else if s + D::to_summary(node).size() <= *self.start() {
            GoRight
        } else {
            Accept
        }
    }
}

/// Locator instance for [`std::ops::RangeFrom<usize>`] representing an index range.
impl<D : Data> Locator<D> for std::ops::RangeFrom<usize> where D::Summary : SizedSummary {
    fn locate(&self, left : D::Summary, node : &D::Value, _right : D::Summary) -> LocResult {
        // find the index of the current node
        let s = left.size();

        if s + D::to_summary(node).size() <= self.start {
            GoRight
        } else {
            Accept
        }
    }
}

/// Locator instance for a reference to [`std::ops::RangeFrom<usize>`] representing an index range.
impl<D : Data> Locator<D> for &std::ops::RangeFrom<usize> where D::Summary : SizedSummary {
    fn locate(&self, left : D::Summary, node : &D::Value, _right : D::Summary) -> LocResult {
        // find the index of the current node
        let s = left.size();

        if s + D::to_summary(node).size() <= self.start {
            GoRight
        } else {
            Accept
        }
    }
}

/// Locator instance for [`std::ops::RangeTo<usize>`] representing an index range.
impl<D : Data> Locator<D> for std::ops::RangeTo<usize> where D::Summary : SizedSummary {
    fn locate(&self, left : D::Summary, _node : &D::Value, _right : D::Summary) -> LocResult {
        // find the index of the current node
        let s = left.size();

        if s >= self.end {
            GoLeft
        } else {
            Accept
        }
    }
}

/// Locator instance for a referencfe to [`std::ops::RangeTo<usize>`] representing an index range.
impl<D : Data> Locator<D> for &std::ops::RangeTo<usize> where D::Summary : SizedSummary {
    fn locate(&self, left : D::Summary, _node : &D::Value, _right : D::Summary) -> LocResult {
        // find the index of the current node
        let s = left.size();

        if s >= self.end {
            GoLeft
        } else {
            Accept
        }
    }
}

/// Locator instance for [`std::ops::RangeToInclusive<usize>`] representing an index range.
/// Do not use with ranges that have been iterated on to exhaustion.
impl<D : Data> Locator<D> for std::ops::RangeToInclusive<usize> where D::Summary : SizedSummary {
    fn locate(&self, left : D::Summary, _node : &D::Value, _right : D::Summary) -> LocResult {
        // find the index of the current node
        let s = left.size();

        if s > self.end {
            GoLeft
        } else {
            Accept
        }
    }
}

/// Locator instance for a reference to [`std::ops::RangeToInclusive<usize>`] representing an index range.
/// Do not use with ranges that have been iterated on to exhaustion.
impl<D : Data> Locator<D> for &std::ops::RangeToInclusive<usize> where D::Summary : SizedSummary {
    fn locate(&self, left : D::Summary, _node : &D::Value, _right : D::Summary) -> LocResult {
        // find the index of the current node
        let s = left.size();

        if s > self.end {
            GoLeft
        } else {
            Accept
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct ByKey<T> (
    pub T,
);

impl<D : Data> Locator<D> for ByKey<(<D::Value as Keyed>::Key, )> where
    D::Value : Keyed,
{
    fn locate(&self, _left : D::Summary, node : &D::Value, _right : D::Summary) -> LocResult {
        match node.get_key().cmp(&self.0.0) {
            std::cmp::Ordering::Less => GoLeft,
            std::cmp::Ordering::Equal => Accept,
            std::cmp::Ordering::Greater => GoRight,
        }
    }
}

/// Locator instance for [`ByKey`]`<`[`std::ops::RangeFull`]`>`.
impl<D : Data> Locator<D> for ByKey<std::ops::RangeFull> {
    fn locate(&self, _left : D::Summary, _node : &D::Value, _right : D::Summary) -> LocResult {
        Accept
    }
}

/// Locator instance for [`ByKey`]`<std::ops::Range<D::Value::Key>>` representing searching by a key.
impl<D : Data> Locator<D> for ByKey<std::ops::Range<<D::Value as Keyed>::Key>> where
    D::Value : Keyed,
{
    fn locate(&self, _left : D::Summary, node : &D::Value, _right : D::Summary) -> LocResult {
        // find the index of the current node
        let key = node.get_key();
        if key < self.0.start {
            GoLeft
        } else if self.0.end <= key {
            GoRight
        } else {
            Accept
        }
    }
}

/*
/// Locator instance for a reference to [`ByKey`]`<std::ops::Range<D::Value::Key>>` representing searching by a key.
impl<D : Data> Locator<D> for &ByKey<std::ops::Range<<D::Value as Keyed>::Key>> where
    D::Value : Keyed,
    <D::Value as Keyed>::Key : Copy,
{
    fn locate(&self, _left : D::Summary, node : &D::Value, _right : D::Summary) -> LocResult {
        // find the index of the current node
        let key = node.get_key();
        if key < self.0.start {
            GoLeft
        } else if self.0.end <= key {
            GoRight
        } else {
            Accept
        }
    }
}
*/

/// Locator instance for [`ByKey`]`<std::ops::RangeInclusive<D::Value::Key>>` representing searching by a key.
/// Do not use with ranges that have been iterated on to exhaustion.
impl<D : Data> Locator<D> for ByKey<std::ops::RangeInclusive<<D::Value as Keyed>::Key>> where
    D::Value : Keyed,
{
    fn locate(&self, _left : D::Summary, node : &D::Value, _right : D::Summary) -> LocResult {
        // find the index of the current node
        let key = &node.get_key();
        if key < self.0.start() {
            GoLeft
        } else if self.0.end() < key {
            GoRight
        } else {
            Accept
        }
    }
}

/*
/// Locator instance fora reference to [`ByKey`]`<std::ops::RangeInclusive<D::Value::Key>>` representing searching by a key.
/// Do not use with ranges that have been iterated on to exhaustion.
impl<D : Data> Locator<D> for &ByKey<std::ops::RangeInclusive<<D::Value as Keyed>::Key>> where
    D::Value : Keyed,
    <D::Value as Keyed>::Key : Copy,
{
    fn locate(&self, _left : D::Summary, node : &D::Value, _right : D::Summary) -> LocResult {
        // find the index of the current node
        let key = node.get_key();
        if key < *self.0.start() {
            GoLeft
        } else if *self.0.end() < key {
            GoRight
        } else {
            Accept
        }
    }
}
*/

/// Locator instance for [`ByKey`]`<`[`std::ops::RangeFrom`]`<D::Value::Key>>` representing an index range.
impl<D : Data> Locator<D> for ByKey<std::ops::RangeFrom<<D::Value as Keyed>::Key>> where
    D::Value : Keyed,
{
    fn locate(&self, _left : D::Summary, node : &D::Value, _right : D::Summary) -> LocResult {
        // find the index of the current node
        let key = node.get_key();
        if key < self.0.start {
            GoLeft
        } else {
            Accept
        }
    }
}

/// Locator instance for [`ByKey`]`<std::ops::RangeTo<D::Value::Key>>` representing searching by a key.
impl<D : Data> Locator<D> for ByKey<std::ops::RangeTo<<D::Value as Keyed>::Key>> where
    D::Value : Keyed,
{
    fn locate(&self, _left : D::Summary, node : &D::Value, _right : D::Summary) -> LocResult {
        // find the index of the current node
        let key = node.get_key();
        if self.0.end <= key {
            GoRight
        } else {
            Accept
        }
    }
}


/// Locator instance for [`ByKey`]`<std::ops::RangeToInclusive<D::Value::Key>>` representing searching by a key.
impl<D : Data> Locator<D> for ByKey<std::ops::RangeToInclusive<<D::Value as Keyed>::Key>> where
    D::Value : Keyed,
{
    fn locate(&self, _left : D::Summary, node : &D::Value, _right : D::Summary) -> LocResult {
        // find the index of the current node
        let key = node.get_key();
        if self.0.end < key {
            GoRight
        } else {
            Accept
        }
    }
}



// TODO: finish all the range types
// TODO: switch to ranges of references to keys


/// A Wrapper for other locators what will find exactly the left edge
/// of the previous locator. So, this is always a splitting locator.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct LeftEdgeOf<L> (
    pub L,
);
/// A Wrapper for other locators what will find exactly the right edge
/// of the previous locator. So, this is always a splitting locator.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RightEdgeOf<L> (
    pub L,
);

impl<D : Data, L : Locator<D>> Locator<D> for LeftEdgeOf<L> {
    fn locate(&self, left : D::Summary, node : &D::Value, right : D::Summary) -> 
        LocResult
    {
        match self.0.locate(left, node, right) {
            Accept => GoLeft,
            res => res,
        }
    }
}

impl<D : Data, L : Locator<D>> Locator<D> for RightEdgeOf<L> {
    fn locate(&self, left : D::Summary, node : &D::Value, right : D::Summary) -> 
        LocResult
    {
        match self.0.locate(left, node, right) {
            Accept => GoRight,
            res => res,
        }
    }
}


/// A Wrapper for other locators what will find the segment to the left
/// of the previous locator. So, `LeftOf(5..8)` is equivalent to `0..5`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct LeftOf<L> (
    pub L,
);

/// A Wrapper for other locators what will find the segment to the right
/// of the previous locator. So, `RightOf(5..8)` is equivalent to `8..`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RightOf<L> (
    pub L,
);

impl<D : Data, L : Locator<D>> Locator<D> for LeftOf<L> {
    fn locate(&self, left : D::Summary, node : &D::Value, right : D::Summary) -> 
        LocResult
    {
        match self.0.locate(left, node, right) {
            GoRight => Accept,
            _ => GoLeft,
        }
    }
}

impl<D : Data, L : Locator<D>> Locator<D> for RightOf<L> {
    fn locate(&self, left : D::Summary, node : &D::Value, right : D::Summary) -> 
        LocResult
    {
        match self.0.locate(left, node, right) {
            GoLeft => Accept,
            _ => GoRight,
        }
    }
}

/// A Wrapper for two other locators, that finds the smallest segment containing both of them.
/// For example, the Union of ranges `[3,6)` and `[8,12)` will  be `[3,12)`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct UnionLocator<L1, L2> (
    pub L1, pub L2
);

impl<D : Data, L1 : Locator<D>, L2 : Locator<D>> Locator<D> for UnionLocator<L1, L2> {
    fn locate(&self, left : D::Summary, node : &D::Value, right : D::Summary) -> 
        LocResult
    {
        let a = self.0.locate(left, node, right);
        let b = self.1.locate(left, node, right);
        if a == b {a} else { Accept }
    }
}