//! This is an example solution to question #680 in [project euler], [yarra gnisrever].
//!
//! This file also contain some unit tests that test that the algorithm's
//! results are correct on small inputs.
//!
//! If you have understood the question and its solution, here's another challenge for you:
//! How will you answer the question, if you had to compute `R(10^8, k)` for
//! every `k <= 10^6`, instead of just `R(10^8, 10^6)`?
//!
//! [yarra gnisrever]: https://projecteuler.net/problem=680
//! [project euler]: https://projecteuler.net/

#[cfg(not(miri))] // miri can't access system resources
use std::time::Instant;

use grove::*;

use example_data::RevAction;
use trees::avl::*;
use trees::splay::*;
use trees::treap::*;

const MODULUS: I = 1_000_000_000;

// an example:

type I = i64;

// inclusive intervals
#[derive(Clone, Copy, Debug)]

// either increasing or decreasing intervals.
// i.e., {start:3, end:7} is [3,4,5,6,7],
// and {start:7, end:4} is [7,6,5,4]
struct Interval {
    start: I,
    end: I,
}

impl Interval {
    fn size(&self) -> usize {
        1 + if self.end > self.start {
            self.end - self.start
        } else {
            self.start - self.end
        } as usize
    }

    fn flip(&mut self) {
        std::mem::swap(&mut self.start, &mut self.end);
    }

    /*
    fn sum(&self) -> I {
        let mut a = self.start; let mut b = self.end;
        if a > b {
            std::mem::swap(&mut a, &mut b);
        }
        b += 1;
        let res = b*(b-1)/2 - a*(a-1)/2;
        res % MODULUS
    }
    */

    // sum of i*A[i] if the array only starts at index `index`
    fn sum_with_index(&self, index2: usize) -> I {
        let index = (index2 as I) % MODULUS;
        let b_modulus = 6 * (MODULUS as i128);
        let size = (self.size() as I) % (2 * MODULUS);
        let size128 = (self.size() as i128) % b_modulus;
        // denote i = index + j
        // i*A[i] = (index + j) * (inter.start + incr*j) = incr*j^2 + j*(incr*index + inter.start) + index*inter.start
        // sum (i < n) i^2 = n*(n-1)*(2n-1)/6
        // sum (i < n) i = n*(n-1)/2
        let ap = ((size128 * (size128 - 1)) % b_modulus) * ((2 * size128 - 1) % b_modulus);
        if ap % 6 != 0 {
            panic!();
        }
        let a = ((ap / 6) % (MODULUS as i128)) as i64;
        let b = if self.start < self.end {
            self.start + index
        } else {
            self.start - index
        } % MODULUS;
        let c = index * (self.start % MODULUS) % MODULUS;

        let xp = size * (size - 1);
        assert!(xp % 2 == 0);
        let x = ((xp / 2) % MODULUS) as i64;

        let mut res = if self.start < self.end {
            a + b * x + c * size
        } else {
            b * x + c * size - a
        };
        res %= MODULUS;

        res
    }

    // split into the index first values and the rest. i.e.,
    // spliting [6,5,4] with index=1 gives [6], [5,4]
    fn split_at_index(&self, index2: usize) -> (Interval, Interval) {
        let index = index2 as I;
        assert!(0 < index2 && index2 < self.size());
        if self.start <= self.end {
            (
                Interval {
                    end: self.start + index - 1,
                    ..*self
                },
                Interval {
                    start: self.start + index,
                    ..*self
                },
            )
        } else {
            (
                Interval {
                    end: self.start - index + 1,
                    ..*self
                },
                Interval {
                    start: self.start - index,
                    ..*self
                },
            )
        }
    }
}

impl Acts<Interval> for RevAction {
    fn act_inplace(&self, val: &mut Interval) {
        if self.to_reverse() {
            val.flip();
        }
    }
}

/// This represents the size of a segment.
/// This is separate from the standard [`example_data::Size`] struct
/// Because [`Size`][example_data::Size] has a default [`ToSummary`] method that
/// always returns a size of `1`, and we would like to override that.
/// Storing the size of a subtree.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct SegmentSize {
    /// The size of a subtree
    pub size: usize,
}

impl std::ops::Add for SegmentSize {
    type Output = SegmentSize;
    fn add(self, b: SegmentSize) -> SegmentSize {
        SegmentSize {
            size: self.size + b.size,
        }
    }
}

impl Default for SegmentSize {
    fn default() -> SegmentSize {
        SegmentSize { size: 0 }
    }
}

impl SizedSummary for SegmentSize {
    fn size(self) -> usize {
        self.size
    }
}

impl ToSummary<SegmentSize> for Interval {
    fn to_summary(&self) -> SegmentSize {
        SegmentSize { size: self.size() }
    }
}

impl Acts<SegmentSize> for RevAction {
    fn act_inplace(&self, _object: &mut SegmentSize) {
        // do nothing
    }
}

struct RevData {}

impl Data for RevData {
    type Action = RevAction;
    type Summary = SegmentSize;
    type Value = Interval;
}

// splits a segment inside the tree
fn search_split<TR: SplittableTreeRef<RevData>>(tree: TR, index: usize) -> TR::T {
    let mut walker = tree.walker();
    // using an empty range so that we'll only end up at a node
    // if we actually need to split that node
    walker.search_subtree(index..index);

    let left = walker.left_summary().size;
    let v2option = walker.with_value(|val| {
        let (v1, v2) = val.split_at_index(index - left);
        *val = v1;
        v2
    });

    if let Some(v2) = v2option {
        walker.next_empty().unwrap(); // not at an empty position
        walker.insert(v2).unwrap();
        walker.go_to_root();
        // after insertion, walker might be at some arbitrary location,
        // depending on the tree type
        walker.search_subtree(index..index);
    }

    walker.split_right().unwrap()
}

fn yarra<T: ConcatenableTree<RevData>>(n: usize, k: usize) -> I
where
    for<'b> &'b mut T: SplittableTreeRef<RevData, T = T>,
    T: std::iter::FromIterator<Interval>,
{
    let inter = Interval {
        start: 0,
        end: (n - 1) as I,
    };
    let mut tree: T = vec![inter].into_iter().collect();

    #[cfg(not(miri))] // miri can't access system resources
    let start_calculation = Instant::now();

    let mut sn = 1;
    let mut tn = 1;
    for _ in 0..k {
        if sn != tn {
            let (low, high) = if sn < tn { (sn, tn + 1) } else { (tn, sn + 1) };

            let mut mid = search_split(&mut tree, low);
            let right = search_split(&mut mid, high - low);
            mid.act_subtree(RevAction { to_reverse: true });
            mid.concatenate_right(right);
            tree.concatenate_right(mid);
        }

        sn += tn;
        sn %= n;
        tn += sn;
        tn %= n;
    }
    #[cfg(not(miri))] // miri can't access system resources
    let calculation_duration = Instant::now().duration_since(start_calculation);
    #[cfg(miri)]
    let calculation_duration = "???";

    // compute the final sum:
    let mut index = 0;
    let mut index_sum = 0;
    for inter in tree.into_iter() {
        index_sum += inter.sum_with_index(index);
        index_sum %= MODULUS;
        index += inter.size();
    }
    println!("n = {}, k = {}", n, k);
    println!(
        "result = {: >9}, main calculation time = {: >17?}",
        index_sum, calculation_duration
    );
    index_sum
}

pub fn main() {
    println!("splay:");
    let res = yarra::<SplayTree<_>>(1000_000_000_000_000_000, 1000_000);
    assert_eq!(res, 563917241);
    println!("done splay\n");

    println!("avl:");
    let res = yarra::<AVLTree<_>>(1000_000_000_000_000_000, 1000_000);
    assert_eq!(res, 563917241);
    println!("done avl\n");

    println!("treap:");
    let res = yarra::<Treap<_>>(1000_000_000_000_000_000, 1000_000);
    assert_eq!(res, 563917241);
    println!("done treap\n");
}

#[test]
pub fn yarra_splay() {
    let res = yarra::<SplayTree<_>>(100, 100);
    assert_eq!(res, 246597);
    #[cfg(not(miri))] // miri is too slow for this
    {
        let res = yarra::<SplayTree<_>>(10000, 10000);
        assert_eq!(res, 275481640);
    }
}

#[test]
pub fn yarra_treap() {
    let res = yarra::<Treap<_>>(100, 100);
    assert_eq!(res, 246597);
    #[cfg(not(miri))] // miri is too slow for this
    {
        let res = yarra::<Treap<_>>(10000, 10000);
        assert_eq!(res, 275481640);
    }
}

#[test]
pub fn yarra_avl() {
    let res = yarra::<AVLTree<_>>(100, 100);
    assert_eq!(res, 246597);
    #[cfg(not(miri))] // miri is too slow for this
    {
        let res = yarra::<AVLTree<_>>(10000, 10000);
        assert_eq!(res, 275481640);
    }
}
