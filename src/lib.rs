
//! This library implements "segment trees" in a generic way.
//! 
//! In this library, a segment tree is a data structure of a sequence of values,
//! that can answer queries about contiguous segments of values,
//! and/or apply actions to contiguous segments of values.
//!
//! For example, a standard segment tree of integers might be able to compute
//! the sum of values in a segment, the maximum values of a segment,
//! and be able to add a constant to all values in a segment.
//!
//! In order to specify what queries and action can be made, the user needs to specify
//! a type that implements the `Action` trait, defined in the `data` module.
//!
//! In order to use a certain kind of tree, i.e., red-black, AVL, splay tree, treaps,
//! scapegoat trees, regular unbalanced trees, the user has to specify a tree type that implements
//! the trait in the `trees` module. (currently only splay trees are implemented, in `trees::splay`)
//!
//! Indeed, the library is generic in both the tree type and the action type: you can use any
//! action type with any tree type.
//!
//! The `methods` module provides some general methods for use on all trees.

#[macro_use]
extern crate derive_destructure;

pub mod telescope; // TODO: should this be public? this should really be its own crate
pub mod trees;
pub mod data;
pub mod methods;
pub mod example;

pub use data::Action;
pub use trees::*;
pub use methods::*;
