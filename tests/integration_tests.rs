mod common;
use common::*;

use grove::data::example_data::*;
use grove::{avl::AVLTree, basic_tree::BasicTree, splay::SplayTree, treap::Treap};

const NUM_ROUNDS: u32 = if cfg!(not(miri)) { 100_000 } else { 100 }; // miri is too slow
const NUM_ROUNDS_SLOW: u32 = if cfg!(not(miri)) { 1_000 } else { 10 }; // miri is too slow

#[test]
fn treap_consistency() {
    check_consistency::<StdNum, Treap<_>, Treap<_>>(NUM_ROUNDS);
}

#[test]
fn splay_and_treap_consistency() {
    check_consistency::<StdNum, SplayTree<_>, Treap<_>>(NUM_ROUNDS);
}

#[test]
fn splay_and_avl_consistency() {
    check_consistency::<StdNum, SplayTree<_>, AVLTree<_>>(NUM_ROUNDS);
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
