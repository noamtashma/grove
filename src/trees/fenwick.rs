//! Fenwick trees. This is a light-weight version of a segment tree that stores all its values
//! in one allocation instead of many small allocations.
//!
//! See [`Fenwick`]

use std::default::Default;

use super::{CommutativeSummary, Group};

/// A fenwick tree. This is a light-weight version of segment tree.
///
/// Does not support applying actions over ranges, inserting or removing elements from the middle.
/// Also, as a design decision, it does not store the elements themselves, only the summaries.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Fenwick<S> {
    data: Vec<S>,
}

impl<S> Fenwick<S> {
    /// Create a new empty fenwick tree
    pub fn new() -> Self {
        Self { data: vec![] }
    }

    /// Checks if the fenwick tree is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// the current length of the fenwick tree
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl<S> Default for Fenwick<S> {
    fn default() -> Self {
        Self::new()
    }
}

/// impl block for the important basic methods
impl<S: Copy + Default + std::ops::Add<Output = S>> Fenwick<S> {
    /// Adds `additional_summary` at `index`, and recalculates.
    ///
    /// If the data isn't commutative, try using `set` instead.
    ///
    /// O(log N)
    pub fn add(&mut self, mut index: usize, additional_summary: S)
    where
        S: CommutativeSummary,
    {
        while index < self.len() {
            self.data[index] = self.data[index] + additional_summary;
            index = index | (index + 1);
        }
    }

    // TODO: It's possible to implement the `set` method without the requirement that the group
    // is commutative. It's a pain to implement, though.

    /// Sets the summary at index `index` to be `new_summary`, and recalculates.
    /// O(log N)
    pub fn set(&mut self, mut index: usize, new_summary: S)
    where
        S: Group + CommutativeSummary,
    {
        // The previous summary at `index`
        let prev = self.get(index);
        let additional = new_summary + -prev;

        while index < self.len() {
            self.data[index] = self.data[index] + additional;
            index = index | (index + 1);
        }
    }

    /// Calculate the summary of values in `..index`
    /// O(log N)
    pub fn sum_prefix(&self, mut index: usize) -> S {
        assert!(
            index <= self.len(),
            "Index {index} is out of bounds of fenwick tree length {}",
            self.len()
        );

        let mut result: S = Default::default();
        while index != usize::MAX {
            // The order here is important for non-commutative summaries.
            result = self.data[index] + result;
            index = (index & (index + 1)).wrapping_sub(1);
        }
        result
    }

    /// Calculate the summary of values in `start..index`.
    /// O(log N)
    /// TODO: Use the built in ranges for this.
    pub fn sum_range(&self, start: usize, end: usize) -> S
    where
        S: Group,
    {
        assert!(
            start <= end,
            "start of range {start} is bigger than end of range {end}"
        );

        // Note: the order is important for non commutative data
        -self.sum_prefix(start) + self.sum_prefix(end)
    }

    /// Calculate the summary at `index`.
    /// O(log N)
    pub fn get(&self, index: usize) -> S
    where
        S: Group,
    {
        assert!(
            index < self.len(),
            "Index {index} is out of bounds of fenwick tree length {}",
            self.len()
        );
        self.sum_range(index, index + 1)
    }

    /// Add a new value to the end of the fenwick tree.
    /// O(log N).
    ///
    /// But actually `k` successive pushes together only take
    /// O(log N + k) time, so the amortized complexity of a push is O(1),
    /// given that there are no calls to `pop`.
    pub fn push(&mut self, new_summary: S) {
        // The new summary that we will push to `self.data`
        // represents the summary of the segment `stop..new_length`
        let stop: usize = (self.len() & (self.len() + 1)).wrapping_sub(1);

        // `result` will have the summary of the segment `stop..old_len`,
        // without the new addition.
        let result: S = {
            // Start with the empty summary
            let mut result = Default::default();
            let mut index: usize = self.len().wrapping_sub(1);
            // One by one add segments up
            while index != stop {
                result = self.data[index] + result;
                index = (index & (index + 1)).wrapping_sub(1);
            }
            result
        };

        self.data.push(result + new_summary);
    }

    /// Pop the last element.
    /// O(1)
    pub fn pop(&mut self) {
        self.data.pop();
    }
}

impl<S: Copy + Default + std::ops::Add<Output = S>> FromIterator<S> for Fenwick<S> {
    /// Creates a fenwick tree from an iterator of one-element summaries.
    /// O(N).
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        // TODO: Interesting idea: this function could take advantage of optimizations where
        // we reuse an allocation from the iterator.

        let mut res = Self::new();
        res.extend(iter);
        res
    }
}

impl<S: Copy + Default + std::ops::Add<Output = S>> Extend<S> for Fenwick<S> {
    fn extend<T: IntoIterator<Item = S>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        let (lower_bound, _) = iter.size_hint();
        self.data.reserve(lower_bound);

        for elem in iter {
            // Successive pushes actually have amortized `O(1)` time complexity
            self.push(elem)
        }
    }
}
