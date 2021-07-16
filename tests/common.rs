use example_data::{RevAffineAction, StdNum};
use grove::*;
use rand::{self, Rng};

fn random_range(len: usize) -> std::ops::Range<usize> {
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

const INITIAL_SIZE: usize = 200;
const NUM_ROUNDS: usize = if cfg!(not(miri)) { 10_000 } else { 100 }; // miri is too slow
pub fn check_consistency<T1, T2>()
where
    T1: SomeTree<StdNum>,
    for<'a> &'a mut T1: ModifiableTreeRef<StdNum>,
    T2: SomeTree<StdNum>,
    for<'a> &'a mut T2: ModifiableTreeRef<StdNum>,
{
    let mut rng = rand::thread_rng();
    let mut len: usize = INITIAL_SIZE;

    let range = 0..(len as _);
    let mut tree1: T1 = range.clone().collect();
    let mut tree2: T2 = range.collect();

    for _ in 0..NUM_ROUNDS {
        match rng.gen_range(0..4) {
            // act on a segment
            0 => {
                let range = &random_range(len);
                let action = random_action(&mut rng);
                tree1.act_segment(action, range);
                tree2.act_segment(action, range);
            }
            // query a segment
            1 => {
                let range = &random_range(len);
                let sum1 = tree1.segment_summary(range);
                let sum2 = tree2.segment_summary(range);
                assert_eq!(sum1, sum2);
                assert_eq!(sum1.size(), range.len());
            }
            // insert a value
            2 => {
                let value = rng.gen_range(-MAX_ADD..=MAX_ADD);
                let index = rng.gen_range(0..=len);
                tree1.slice(index..index).insert(value).unwrap();
                tree2.slice(index..index).insert(value).unwrap();
                len += 1;
            }
            // delete a value
            3 if len > 0 => {
                let index = rng.gen_range(0..len);
                let val1 = tree1.slice(index..=index).delete();
                let val2 = tree2.slice(index..=index).delete();
                assert_eq!(val1, val2);
                assert!(val1.is_some()); // actually deleted a value
                len -= 1;
            }
            // delete but the tree is empty
            3 => {}
            _ => panic!(),
        }
        // do these checks in all cases
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
