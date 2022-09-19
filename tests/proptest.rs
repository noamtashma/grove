#![cfg_attr(feature = "bench", feature(test))]

pub mod common;
pub use common::*;

use grove::data::{example_data::*, *};
use grove::{avl::AVLTree, splay::SplayTree, treap::Treap};
use proptest::prelude::*;

pub fn round_action_strategy<D: Data>(
    len: usize,
    data_strat: impl Strategy<Value = D::Value> + 'static,
    action_strat: impl Strategy<Value = D::Action> + 'static,
) -> impl Strategy<Value = RoundAction<D>>
where
    D: std::fmt::Debug,
    D::Action: std::fmt::Debug,
    D::Value: std::fmt::Debug,
{
    let range_strat =
        (0..len, 0..len).prop_filter_map("illogical range (start > end)", |(start, end)| {
            if start <= end {
                Some(start..end)
            } else {
                None
            }
        });
    // Delete
    (0..len)
        .prop_map(|ix| RoundAction::Delete { index: ix })
        .boxed()
        .prop_union(
            // Insert
            (0..=len, data_strat)
                .prop_map(|(ix, val)| RoundAction::Insert {
                    index: ix,
                    value: val,
                })
                .boxed(),
        )
        // Act
        .or((range_strat.clone(), action_strat)
            .prop_map(|(range, action)| RoundAction::Act { range, action })
            .boxed())
        // Query
        .or(range_strat
            .prop_map(|range| RoundAction::Query { range })
            .boxed())
}

/// `-mul_bound..=mul_bound` is the bound on the multiplicative factor in the action.
/// Should generally be `1` or `2` to avoid blowup and overflow in the tests.
pub fn RevAffineAction_strategy(
    mul_bound: i32,
) -> impl Strategy<Value = example_data::RevAffineAction> + 'static {
    (proptest::bool::ANY, -mul_bound..=mul_bound, -100..100).prop_map(|(to_reverse, mul, add)| {
        RevAffineAction {
            to_reverse,
            mul,
            add,
        }
    })
}

const NUM_ROUNDS: usize = if cfg!(not(miri)) { 10_000 } else { 100 }; // miri is too slow
const NUM_ROUNDS_SLOW: u32 = if cfg!(not(miri)) { 100 } else { 10 }; // miri is too slow

// Parameters for `StdNum` initial array
fn comm_initial() -> impl proptest::prelude::Strategy<Value = Vec<i32>> {
    proptest::collection::vec(-200..200i32, 0..200)
}

// Parameters for `StdNum` and noncommutative data
fn comm_params<D>() -> impl proptest::prelude::Strategy<Value = Vec<RoundAction<D>>>
where
    D: Data<Value = i32, Action = RevAffineAction> + std::fmt::Debug,
{
    proptest::collection::vec(
        round_action_strategy(200, -100..100, RevAffineAction_strategy(2)),
        1..500,
    )
}

// Parameters for `StdNum` and noncommutative data
fn noncomm_params<D>() -> impl proptest::prelude::Strategy<Value = Vec<RoundAction<D>>>
where
    D: Data<Value = i32, Action = RevAffineAction> + std::fmt::Debug,
{
    proptest::collection::vec(
        round_action_strategy(100, -100..100, RevAffineAction_strategy(1)),
        1..500,
    )
}

proptest::proptest! {
    #[test]
    fn treap_consistency_proptest(initial in comm_initial(), array in comm_params()) {
        check_consistency_proptest::<StdNum, Treap<_>, Treap<_>>(&initial, &array)?;
    }


    #[test]
    fn splay_and_treap_consistency_proptest(initial in comm_initial(), array in comm_params()) {
        check_consistency_proptest::<StdNum, SplayTree<_>, Treap<_>>(&initial, &array)?;
    }

    #[test]
    fn splay_and_avl_consistency_proptest(initial in comm_initial(), array in comm_params()) {
        check_consistency_proptest::<StdNum, SplayTree<_>, AVLTree<_>>(&initial, &array)?;
    }
}

proptest::proptest! {
    #![proptest_config(ProptestConfig {
        cases: 5, max_shrink_iters: 20480, .. ProptestConfig::default()
      })]
    #[test]
    fn treap_consistency_noncommutative_proptest(initial in comm_initial(), array in noncomm_params()) {
        check_consistency_proptest::<(i32, PolyNum<3>, RevAffineAction), Treap<_>, Treap<_>>(&initial, &array)?;
    }

    #[test]
    fn splay_treap_consistency_noncommutative_proptest(initial in comm_initial(), array in noncomm_params()) {
        check_consistency_proptest::<(i32, PolyNum<3>, RevAffineAction), SplayTree<_>, Treap<_>>(&initial, &array)?;
    }

    #[test]
    fn splay_avl_consistency_noncommutative_proptest(initial in comm_initial(), array in noncomm_params()) {
        check_consistency_proptest::<(i32, PolyNum<3>, RevAffineAction), SplayTree<_>, AVLTree<_>>(&initial, &array)?;
    }

}

// Used to debug a specific test case
//
// #[test]
// fn specific_temporary_test() {
//     use RoundAction::*;
//     let initial = [0, 0];
//     let array = [Act {
//         range: 0..2,
//         action: RevAffineAction {
//             to_reverse: true,
//             mul: 1,
//             add: 0,
//         },
//     }];
//     let res = check_consistency_proptest::<StdNum, SplayTree<_>, AVLTree<_>>(&initial, &array);
//     match res {
//         Ok(_) => (),
//         Err(_) => panic!(),
//     }
// }
