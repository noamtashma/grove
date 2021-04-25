
//! The Orchard library is a library that implements "segment trees" in a generic way.
//! 
//! In Orchard, a segment tree is a data structure containing a sequence of values,
//! that can answer queries about contiguous subsegments of values,
//! and/or apply actions to contiguous subsegments of values, in logarithmic time, in addition
//! to other operations.
//!
//! For example, a standard segment tree of integers might be able to compute
//! the sum of values in a subsegment, the maximum value of a subsegment,
//! and be able to add a constant to all values in a subsegment, all in logarithmic time.
//!
//! In order to specify what queries and action can be made, the user needs to specify
//! a type that implements the [`Data`] trait, defined in the [`data`] module.
//! 
//! Overall, the operations you can do with a segment tree are (every one in logarithmic time):
//! * Insert, delete and modify specific values
//! * Compute a summary value of a subsegment
//!     Complexity is always logarithmic,
//!     however, the summary type must satisfy the [`Data`] trait and be compatible
//!     with the [`Data::Value`] and [`Data::Action`] types.
//! * Apply an action on every element of a subsegment
//!     Complexity is always logarithmic,
//!     however, the action type must satisfy the [`Data`] trait and be compatible
//!     with the [`Data::Summary`] and [`Data::Value`] types.
//! * Reverse subsegments
//!     Must be part of the action type.
//!     Possible only with some balanced tree algorithms.
//! * Search for specific elements. See [`locator`] module.
//! * Split and concatenate segment trees
//!     Possible only with some balanced tree algorithms.
//!
//! In order to use a certain kind of tree, i.e., red-black, AVL, splay tree, treaps,
//! scapegoat trees, regular unbalanced trees, or any other, the user has to specify
//! a tree type that implements the trait in the [`trees`] module. (currently only
//! splay trees are implemented, in [`trees::splay`])
//!
//! Indeed, the library is generic in both the tree type and the action type: you can use any
//! action type with any tree type.
//!
//! The [`methods`] module provides some general methods for use on all trees.

#[macro_use]
extern crate derive_destructure;

pub mod telescope; // TODO: should this be public? this should really be its own crate
pub mod trees;
pub mod data;
pub mod example;

pub use data::*;
pub use trees::*;
pub use trees::methods::*;
