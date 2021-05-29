mod common;
use common::*;

use orchard::trees::{avl::AVLTree, splay::SplayTree, treap::Treap};

#[test]
fn splay_and_treap_consistency() {
    check_consistency::<SplayTree<_>, Treap<_>>();
}
#[test]
fn splay_and_avl_consistency() {
    check_consistency::<SplayTree<_>, AVLTree<_>>();
}
