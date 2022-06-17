#![cfg_attr(feature = "bench", feature(test))]

pub mod common;
pub use common::*;

use grove::data::example_data::*;
use grove::{avl::AVLTree, basic_tree::BasicTree, splay::SplayTree, treap::Treap};

const NUM_ROUNDS: u32 = if cfg!(not(miri)) { 10_000 } else { 100 }; // miri is too slow
const NUM_ROUNDS_SLOW: u32 = if cfg!(not(miri)) { 100 } else { 10 }; // miri is too slow

#[test]
fn treap_consistency() {
    check_consistency::<StdNum, Treap<_>, Treap<_>>(NUM_ROUNDS);
}

#[test]
fn splay_and_treap_consistency() {
    check_consistency::<StdNum, SplayTree<_>, Treap<_>>(NUM_ROUNDS);
}

proptest::proptest! {
    #[test]
    fn splay_and_treap_consistency_proptest(array in proptest::collection::vec(round_action_strategy(150, -100..100, RevAffineAction_strategy() ), 100)) {
        match check_consistency_proptest::<StdNum, SplayTree<_>, Treap<_>>(array) {
            Ok(_) => (),
            Err(_) => panic!(),
        }
    }
}

#[test]
fn splay_and_avl_consistency() {
    check_consistency::<StdNum, SplayTree<_>, AVLTree<_>>(NUM_ROUNDS);
}

proptest::proptest! {
    // #![proptest_config(proptest::prelude::ProptestConfig {
    //     cases: 50, .. proptest::prelude::ProptestConfig::default()
    //   })]
    #[test]
    fn splay_and_avl_consistency_proptest(array in proptest::collection::vec(round_action_strategy(150, -100..100, RevAffineAction_strategy() ), 1..70)) {
        match check_consistency_proptest::<StdNum, SplayTree<_>, AVLTree<_>>(array) {
            Ok(_) => (),
            Err(_) => panic!(),
        }
    }
}

#[test]
fn splay_and_avl_consistency_specific() {
    let array = vec![
        RoundAction::Act {
            range: 0..56,
            action: RevAffineAction {
                to_reverse: false,
                mul: 0,
                add: 0,
            },
        },
        RoundAction::Act {
            range: 1..1,
            action: RevAffineAction {
                to_reverse: true,
                mul: 0,
                add: 0,
            },
        },
    ];
    match check_consistency_proptest::<StdNum, SplayTree<_>, AVLTree<_>>(array) {
        Ok(_) => (),
        Err(_) => panic!(),
    }
}

#[test]
fn splay_and_treap_consistency_noncommutative() {
    check_consistency::<(i32, PolyNum<3>, RevAffineAction), SplayTree<_>, Treap<_>>(
        NUM_ROUNDS_SLOW,
    );
}

#[test]
fn splay_and_avl_consistency_noncommutative() {
    check_consistency::<(i32, PolyNum<3>, RevAffineAction), SplayTree<_>, AVLTree<_>>(
        NUM_ROUNDS_SLOW,
    );
}

#[test]
fn treap_consistency_noncommutative() {
    check_consistency::<(i32, PolyNum<3>, RevAffineAction), Treap<_>, Treap<_>>(NUM_ROUNDS_SLOW);
}

#[test]
fn splay_insert() {
    check_insert::<SplayTree<_>>(true);
}

#[test]
fn avl_insert() {
    check_insert::<AVLTree<_>>(false);
}

#[test]
fn treap_insert() {
    check_insert::<Treap<_>>(true);
}

#[test]
fn basic_insert() {
    check_insert::<BasicTree<_>>(true);
}

#[test]
fn splay_delete() {
    check_delete::<SplayTree<_>>();
}

#[test]
fn avl_delete() {
    check_delete::<AVLTree<_>>();
}

#[test]
fn treap_delete() {
    check_delete::<Treap<_>>();
}

#[test]
fn basic_delete() {
    check_delete::<BasicTree<_>>();
}
