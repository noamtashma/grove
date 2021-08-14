#[cfg(feature = "bench")]
pub mod bench;

use example_data::{RevAffineAction, StdNum};
use grove::*;
use rand::{self, Rng};
use std::ops::Range;

/// Something to perform in one round of tests
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum RoundAction<D: Data> {
    Act {
        range: Range<usize>,
        action: D::Action,
    },
    Query {
        range: Range<usize>,
    },
    Insert {
        index: usize,
        value: D::Value,
    },
    Delete {
        index: usize,
    },
}

/// The result after one round of querying
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum RoundResult<D: Data> {
    Empty,
    Summary(D::Summary),
    Value(D::Value),
}

fn random_range(len: usize) -> Range<usize> {
    let mut rng = rand::thread_rng();
    let res = (rng.gen_range(0..len + 1), rng.gen_range(0..len + 1));
    if res.0 <= res.1 {
        res.0..res.1
    } else {
        res.1..res.0
    }
}

const MAX_ADD: i32 = 200;
fn random_action(rng: &mut rand::prelude::ThreadRng) -> RevAffineAction {
    RevAffineAction {
        to_reverse: rng.gen(),
        mul: if rng.gen() { 1 } else { -1 },
        add: rng.gen_range(-MAX_ADD..=MAX_ADD),
    }
}

fn random_round_action<D>(rng: &mut rand::prelude::ThreadRng, len: usize) -> RoundAction<D>
where
    D: Data<Value = i32, Action = RevAffineAction>,
    D::Summary: std::fmt::Debug + Eq + SizedSummary,
{
    use RoundAction::*;
    match rng.gen_range(0..4) {
        // act on a segment
        0 => {
            let range = random_range(len);
            let action = random_action(rng);
            Act { action, range }
        }
        // query a segment
        1 => {
            let range = random_range(len);
            Query { range }
        }
        // insert a value
        2 => {
            let value = rng.gen_range(-MAX_ADD..=MAX_ADD);
            let index = rng.gen_range(0..=len);
            Insert { value, index }
        }
        // delete a value
        3 => {
            let index = if len > 0 {
                rng.gen_range(0..len)
            } else {
                0 // dummy value
            };
            Delete { index }
        }
        _ => {
            panic!()
        }
    }
}

fn run_round<D, T>(
    round_action: RoundAction<D>,
    tree: &mut T,
    len: usize,
    mutable_query: bool,
) -> RoundResult<D>
where
    D: Data<Value = i32, Action = RevAffineAction>,
    D::Summary: std::fmt::Debug + Eq + SizedSummary,
    T: SomeTree<D>,
    for<'a> &'a mut T: ModifiableTreeRef<D>,
{
    use RoundAction::*;
    use RoundResult::*;

    match round_action {
        // act on a segment
        Act { range, action } => {
            tree.act_segment(action, range);
            Empty
        }
        // query a segment
        Query { range } => {
            let sum = if mutable_query {
                tree.segment_summary(&range)
            } else {
                tree.segment_summary_imm(&range)
            };
            assert_eq!(sum.size(), range.len());
            Summary(sum)
        }
        // insert a value
        Insert { index, value } => {
            tree.slice(index..index).insert(value).unwrap();
            // len += 1;
            Empty
        }
        // delete a value
        Delete { index } if len > 0 => {
            let val_op = tree.slice(index..=index).delete();
            match val_op {
                None => panic!(), // didn't actually delete a value
                Some(val) => {
                    /* len -= 1; */
                    Value(val)
                }
            }
        }
        // delete but the tree is empty
        Delete { .. } => {
            assert!(tree.is_empty());
            Empty
        }
    }
}

const INITIAL_SIZE: usize = 200;
pub fn check_consistency<D, T1, T2>(num_rounds: u32)
where
    D: Data<Value = i32, Action = RevAffineAction>,
    D: Clone + std::fmt::Debug + Eq, // useless bounds because the auto-generated clone instance for RoundAction requires it
    D::Summary: std::fmt::Debug + Eq + SizedSummary,
    T1: SomeTree<D>,
    for<'a> &'a mut T1: ModifiableTreeRef<D>,
    T2: SomeTree<D>,
    for<'a> &'a mut T2: ModifiableTreeRef<D>,
{
    let mut rng = rand::thread_rng();
    let mut len: usize = INITIAL_SIZE;

    let range = 0..(len as _);
    let mut tree1: T1 = range.clone().collect();
    let mut tree2: T2 = range.collect();

    for _ in 0..num_rounds {
        let round_action = random_round_action::<D>(&mut rng, len);
        let res1 = run_round(round_action.clone(), &mut tree1, len, true);
        let res2 = run_round(round_action.clone(), &mut tree2, len, false);
        assert_eq!(res1, res2);
        // update length
        match round_action {
            RoundAction::Delete { .. } => {
                if len > 0 {
                    len -= 1;
                }
            }
            RoundAction::Insert { .. } => {
                len += 1;
            }
            _ => {}
        }

        let s1 = tree1.subtree_summary();
        let s2 = tree2.subtree_summary();
        assert_eq!(s1, s2);
        assert_eq!(s1.size(), len);
        // This check takes `O(n)` time. However, since the trees aren't so big in this test
        // (200 starting size + order of magnitude of the variance is about 100)
        // the check doesn't take too long.
        tree1.assert_correctness();
        tree2.assert_correctness();
    }
}

pub fn check_delete<T>()
where
    T: SomeTree<StdNum>,
    for<'a> &'a mut T: ModifiableTreeRef<StdNum>,
{
    let arr: Vec<_> = (0..500).collect();
    for i in 0..arr.len() {
        let mut tree: T = arr.iter().cloned().collect();
        let mut walker = tree.search(i);
        assert_eq!(walker.value().cloned(), Some(arr[i]));
        let res = walker.delete();
        assert_eq!(res, Some(arr[i]));
        drop(walker);
        tree.assert_correctness();
        assert_eq!(
            tree.into_iter().collect::<Vec<_>>(),
            arr[..i]
                .iter()
                .chain(arr[i + 1..].iter())
                .cloned()
                .collect::<Vec<_>>()
        );
    }
}

pub fn check_insert<T>(should_walker_stay_at_inserted_value: bool)
where
    T: SomeTree<StdNum>,
    for<'a> &'a mut T: ModifiableTreeRef<StdNum>,
{
    let arr: Vec<_> = (0..500).collect();
    for i in 0..=arr.len() {
        let new_val = 13;
        let mut tree: T = arr.iter().cloned().collect();
        let mut walker = tree.search(i..i);
        walker.insert(new_val);
        if !should_walker_stay_at_inserted_value {
            // after inserting, the walker can move, because of rebalancing.
            // for example, in avl trees, the walker should be in an ancestor of the inserted value.
            // therefore, we check with `search_subtree`.
            walker.search_subtree(i);
        }
        assert_eq!(walker.value().cloned(), Some(new_val));
        drop(walker);
        tree.assert_correctness();
        assert_eq!(
            tree.into_iter().collect::<Vec<_>>(),
            arr[..i]
                .iter()
                .chain([new_val].iter())
                .chain(arr[i..].iter())
                .cloned()
                .collect::<Vec<_>>()
        );
    }
}
