use super::*;
extern crate test;
use test::Bencher;

use grove::{avl::AVLTree, basic_tree::BasicTree, splay::SplayTree, treap::Treap};

pub fn bench_tree<D, T>(b: &mut Bencher)
where
    D: Data<Value = i32, Action = RevAffineAction>,
    D: Clone + std::fmt::Debug + Eq, // useless bounds because the auto-generated clone instance for RoundAction requires it
    D::Summary: std::fmt::Debug + Eq + SizedSummary,
    T: SomeTree<D>,
    for<'a> &'a mut T: ModifiableTreeRef<D>,
{
    let mut rng = rand::thread_rng();
    let mut len: usize = INITIAL_SIZE;

    let range = 0..(len as _);
    let mut tree: T = range.clone().collect();
    b.iter(|| {
        let round_action = random_round_action::<D>(&mut rng, len);
        let res = run_round(round_action.clone(), &mut tree, len);
        test::bench::black_box(res);

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

        // should this be included in the benchmark?
        // let s = tree1.subtree_summary();
        // assert_eq!(s.size(), len);
    });
}

#[bench]
fn bench_splay(b: &mut Bencher) {
    bench_tree::<StdNum, SplayTree<_>>(b)
}

#[bench]
fn bench_treap(b: &mut Bencher) {
    bench_tree::<StdNum, Treap<_>>(b)
}

#[bench]
fn bench_avl(b: &mut Bencher) {
    bench_tree::<StdNum, AVLTree<_>>(b)
}