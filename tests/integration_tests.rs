mod common;
use common::*;

use orchard::trees::{avl::AVLTree, basic_tree::BasicTree, splay::SplayTree, treap::Treap};

#[test]
fn splay_and_treap_consistency() {
    check_consistency::<SplayTree<_>, Treap<_>>();
}
#[test]
fn splay_and_avl_consistency() {
    check_consistency::<SplayTree<_>, AVLTree<_>>();
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

// #[test]
// fn basic_insert() {
//     check_insert::<BasicTree<_>>(true);
// }

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

// #[test]
// fn basic_delete() {
//     check_delete::<BasicTree<_>>();
// }
