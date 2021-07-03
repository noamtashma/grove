# Orchard
A segment tree library enabling generic user-defined queries and actions on segments of your data,
paired with different kinds of balanced binary trees (splay trees, avl trees, and so on).

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


[`Data`]: https://docs.rs/orchard/*/orchard/data/trait.Data.html
[`data`]: https://docs.rs/orchard/*/orchard/data/index.html
[`locators`]: https://docs.rs/orchard/*/orchard/locators/index.html
[`methods`]: https://docs.rs/orchard/*/orchard/trees/methods/index.html