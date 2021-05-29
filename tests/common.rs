pub use orchard::*;

/*
struct Simulator<D : Data> {
    vec : Vec<D::Value>,
}

impl<D : Data> Simulator<D> {
    pub fn segment_summary<L : Locator<D>>(&self, loc : L) -> D::Summary {
        todo!()

    }
}
*/

use example_data::StdNum;
use orchard::example_data::RevAffineAction;
use rand::{self, Rng};

pub fn check_consistency<T1, T2>()
where
    T1: SomeTree<StdNum>,
    for<'a> &'a mut T1: SomeTreeRef<StdNum>,
    T2: SomeTree<StdNum>,
    for<'a> &'a mut T2: SomeTreeRef<StdNum>,
{
    const LEN: usize = 200;
    const MAX_ADD: i32 = 200;

    fn random_range() -> std::ops::Range<usize> {
        let mut rng = rand::thread_rng();
        let res = (rng.gen_range(0..LEN + 1), rng.gen_range(0..LEN + 1));
        if res.0 <= res.1 {
            res.0..res.1
        } else {
            res.1..res.0
        }
    }

    fn random_action() -> RevAffineAction {
        let mut rng = rand::thread_rng();
        RevAffineAction {
            to_reverse: rng.gen(),
            mul: if rng.gen() { 1 } else { -1 },
            add: rng.gen_range(-MAX_ADD..=MAX_ADD),
        }
    }

    let range = 0..(LEN as _);
    let mut tree1: T1 = range.clone().collect();
    let mut tree2: T2 = range.collect();
    for _ in 0..10_000 {
        let range = &random_range();
        if rand::random() {
            let action = random_action();
            tree1.act_segment(action, range);
            tree2.act_segment(action, range);
            let s1 = tree1.subtree_summary();
            let s2 = tree2.subtree_summary();
            assert_eq!(s1, s2);
            assert_eq!(s1.size(), LEN);
        } else {
            let sum1 = tree1.segment_summary(range);
            let sum2 = tree2.segment_summary(range);
            assert_eq!(sum1, sum2);
            assert_eq!(sum1.size(), range.len());
        }
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
        let mut walker = methods::search(&mut tree, i);
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
        let mut walker = methods::search(&mut tree, i..i);
        walker.insert(new_val);
        if !should_walker_stay_at_inserted_value {
            // after inserting, the walker can move, because of rebalancing.
            // for example, in avl trees, the walker should be in an ancestor of the inserted value.
            // therefore, we check with `search_subtree`.
            methods::search_subtree(&mut walker, i);
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
