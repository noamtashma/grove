//! # Grove
//! A segment tree library enabling generic user-defined queries and actions on segments of your data,
//! paired with different kinds of balanced binary trees (splay trees, avl trees, and so on). Grove
//! can represent the vast majority of kinds of augmented binary tree, with only a few trait implementations.
//!
//! In Grove, a segment tree is a data structure containing a sequence of values,
//! that can answer queries about contiguous segments of values,
//! and/or apply actions to all values in a contiguous segment.
//!
//! For example, a standard segment tree of integers might be able to compute
//! the sum of values in a segment, the maximum value of a segment,
//! and add a constant to all values in a segment, all in logarithmic time.
//!
//! # Why Grove
//!
//! Although there is a wealth of implementations of balanced binary trees, most
//! implementations only implement a `Set` of `Map` type of ordered values. Almost all implementations
//! implement only a particular kind of balanced tree, with a particular value type, no kind
//! of summary data other than size and indexing data, and certainly no kind of segment
//! actions. In addition, virtually no implementation exposes enough of its implementation
//! to let the user implement his own tree algorithms if he needs to.
//!
//! Currently, if you would like to find the `i`'th element of your `BTreeSet`,
//! you would need to re-implement the data structure from scratch or modify the existing code. If you would like to have an efficient `union` method, tough luck.
//!
//! That is in spite of the fact all of these can be implemented in a generic way, giving the user the ability to get their desired data structure with only implementing a few traits.
//!
//! ## Goals
//!
//! Grove aims to be the most generic segment tree library possible. Grove should be able to represent as many kinds of segment tree / augmented tree as possible (and it certainly can much more than any other implementation known to the author). Grove is generic in:
//! * Balanced tree algorithm (currently only implements Splay tree, AVL tree, and Treap).
//! * Value type.
//! * The way in which you can search for elements in the structure.
//! * Segment summaries - the augmentation data about the subtree stored in each node.
//! * Segment actions - actions that can be efficiently applied to whole segments, implemented using lazy push-down.
//!
//! In addition, Grove aims to be somewhat amenable implementing the user's own tree algorithms. However, this probably requires more work.
//!
//! ## Drawbacks
//! Naturally, as the code will be generically written, common optimizations that aren't always possible will be missed. It will not be as efficient as possible for every specific instantiation. That is even though Rust's monomorphization guarantees the code will be optimized for your specific instantiation.
//!
//! In addition, the library may be too big and complex, with a large number of traits and abstractions. I try my best to provide good documentation everywhere. However, that is the cost of writing highly generic code.
//!
//! ## Prior art
//!
//! There already are some implementations that are somewhat generic. To the best of the author's knowledge, they can be summed up as:
//! * Many `Set` and `Map` types are written generically over the value type, as long as it is ordered.
//! * Haskell's [`FingerTree`](https://hackage.haskell.org/package/fingertree-0.1.4.2/docs/Data-FingerTree.html) is an implementation of finger trees that allows the user to provide the summary data, and implements generic binary searching based on that data.
//!
//! # Basic usage
//!
//! ## The [`Data`] trait
//! The [`Data`] trait is a marker trait that specifies what queries and actions can be made. You can use the pre-made instances from [`example_data`], such as `PlainData<V>` or `SizeData<V>`. Alternatively, You can make your own combination.
//! The [`Data`] trait has three associated types:
//! * [`Data::Value`] is the type of values represented in the tree.
//! * [`Data::Summary`] is the result when querying the tree about a specific segment.
//! * [`Data::Action`] is the type of actions you can perform on segments of the tree.
//! If you don't want summaries or actions for your tree, use [`example_data::Unit`] in their place.
//!
//! These types have to implement the trait and conform its restrictions, in order
//! for the tree to behave correctly. See its documentation.
//!
//! After choosing the three types, you can use either `(Value, Summary, Action)` as the type that implements the `Data` trait, or create your own new marker trait that implements the `Data` trait.
//!
//! ```rust
//! use grove::*;
//! use example_data::{Size, Unit};
//!
//! /// instantiation for an int set type
//! type IntSetData = (
//!     /* ordered integers */ i32,
//!     /* for indexing purposes */ Size,
//!     /* no actions */ Unit
//! );
//!
//! type Set = treap::Treap<IntSetData>;
//! ```
//!
//! ## Tree operations
//! Overall, you can perform these operations on a segment tree in logarithmic time:
//! * Insert, delete and modify specific values.
//! * Compute a summary value of type [`Data::Summary`] of a segment.
//! * Apply an action of type [`Data::Action`] on every element of a segment.
//! * Choose what segment to query/apply action on, or search for a specific element, using binary search. See [`locators`] module.
//! * Reverse segments of trees (as part of the `Action` type).
//! * split and concatenate segment trees.
//!
//! These tree operations can be found in the corresponding traits in the `trees` module. In addition, most operations can be accessed using the [`Slice`] type, such as `tree.slice(locator).insert(new_value)`.
//!
//! In order to use a certain kind of tree, i.e., red-black, AVL, splay tree, treaps,
//! scapegoat trees, regular unbalanced trees, or any other, the user has to specify
//! a tree type that implements the trait in the [`trees`] module. (currently
//! splay/AVL/treaps/unbalanced trees are implemented)
//!
//! ```rust
//! use grove::*;
//! use locators::ByKey; // for ordered sets
//! use example_data::{Size, Unit};
//!
//! /// instantiation for an int set type
//! type IntSetData = (
//!     /* integers */ i32,
//!     /* for indexing purposes */ Size,
//!     /* no actions */ Unit
//! );
//!
//! type Set = treap::Treap<IntSetData>;
//!
//! let mut my_set: Set = [0,1,2,4,6,7,8].iter().cloned().collect();
//! // at the location in the ordered set where 5 should be, insert 5
//! my_set.slice(ByKey((&5,))).insert(5);
//! let vec: Vec<i32> = my_set.into_iter().collect();
//! assert_eq!(vec, vec![0,1,2,4,5,6,7,8]);
//! ```
//!
//! # Advanced examples
//!
//! In the examples folder in the library (which is automatically stripped from crates.io), there are two
//! examples showing usage of the library in two data structure/algorithmic questions. One is [yarra gnisrever], question #680 in [project euler]. The other is [pyramid base], a question from IOI 2008.
//!
//! Both questions show how it's possible to define your own instance of [`Data`] for your specific usecase.
//! They also show how you can write code that's generic with regards to the tree type:
//! Both use the same code to run with treaps, splay trees and avl trees.
//!
//! Notes: In order to run pyramid_base, you will need to download the pyramid base test files from [here], and save them in a new folder named "pyramid_base_test_files". See also in the example code.
//!
//! [`Slice`]: [slice::Slice]
//!
//! [yarra gnisrever]: https://projecteuler.net/problem=680
//! [project euler]: https://projecteuler.net/
//! [pyramid base]: https://dmoj.ca/problem/ioi08p6
//! [here]: https://ioinformatics.org/page/ioi-2008/34

#![deny(missing_docs)]
#![forbid(unsafe_code)]

#[macro_use]
extern crate derive_destructure;

pub mod data;
pub mod locators;
pub mod trees;

pub use data::*;
pub use locators::Locator;
pub use trees::*;
