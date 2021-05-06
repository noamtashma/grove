
mod common;
use common::*;

use orchard::trees::{splay::SplayTree, treap::Treap};

#[test]
fn splay_and_treap_consistency() {
    check_consistency::<SplayTree<_>, Treap<_>>();
}