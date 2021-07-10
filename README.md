# Orchard
A segment tree library enabling generic user-defined queries and actions on segments of your data,
paired with different kinds of balanced binary trees (splay trees, avl trees, and so on).
(The package name is `generic_segment_trees`, however, the library's name is `orchard`).

In Orchard, a segment tree is a data structure containing a sequence of values,
that can answer queries about contiguous segments of values,
and/or apply actions to all values in a contiguous segment, in efficient logarithmic time.

For example, a standard segment tree of integers might be able to compute
the sum of values in a segment, the maximum value of a segment,
and add a constant to all values in a segment, all in logarithmic time.

In order to specify what queries and actions can be made, the user needs to specify
a marker type that implements the [`Data`] trait, defined in the [`data`] module. The
[`Data`] trait has three associated types:
* [`Data`]`::Value` is the type of values represented in the tree
* [`Data`]`::Summary` is the result when querying the tree about a specific segment
* [`Data`]`::Action` is the type of actions you can perform on segments of the tree
these types have to implement the trait and conform its restrictions, in order
for the tree to behave correctly. See its documentation.

Overall, you can perform these operations on a segment tree in logarithmic time:
* Insert, delete and modify specific values
* Compute a summary value of type `Data::Summary` of a segment
* Apply an action of type `Data::Action` on every element of a segment
* Choose what segment to query/apply action on, or search for a specific element, using binary search. See [`locators`] module.
* Reverse segments of trees, split and concatenate segment trees

In order to use a certain kind of tree, i.e., red-black, AVL, splay tree, treaps,
scapegoat trees, regular unbalanced trees, or any other, the user has to specify
a tree type that implements the trait in the [`trees`] module. (currently
splay/AVL/treaps/unbalanced trees are implemented)

Indeed, the library is generic in both the tree type and the [`Data`] instance: you can use any
setting with any tree type.

The [`methods`] module provides some general methods for use on all trees.

# Advanced examples

In the examples folder in the library (which is automatically stripped from crates.io), there are two
examples showing usage of the library in two data structure/algorithmic questions. One is [yarra gnisrever], question #680 in [project euler]. The other is [pyramid base], a question from IOI 2008.

Both questions show how it's possible to define your own instance of [`Data`] for your specific usecase.
They also show how you can write code that's generic with regards to the tree type:
Both use the same code to run with treaps, splay trees and avl trees.

Notes: In order to run pyramid_base, you will need to download the pyramid base test files from [here], and save them in a new folder named "pyramid_base_test_files". See also in the example code.

[`Data`]: https://docs.rs/generic_segment_trees/*/orchard/data/trait.Data.html
[`data`]: https://docs.rs/generic_segment_trees/*/orchard/data/index.html
[`locators`]: https://docs.rs/generic_segment_trees/*/orchard/locators/index.html
[`methods`]: https://docs.rs/generic_segment_trees/*/orchard/trees/methods/index.html
[yarra gnisrever]: https://projecteuler.net/problem=680
[project euler]: https://projecteuler.net/
[pyramid base]: https://dmoj.ca/problem/ioi08p6
[here]: https://ioinformatics.org/page/ioi-2008/34