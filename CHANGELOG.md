This is an imprecise changelog, as the library is still in development.

# Version 0.2.0
* Improved the documentation of the library
* The structure of the [`example_data`] module changed to become more modular.
* The [`Keyed`] trait changed its structure.
* The `data` trait became more modular. In particular, the [`to_summary`] method is now defined in its own trait, and three-tuples `(Value, Summary, Action)` can be used as [`Data`] instead of manually implementing it on a new marker type.
* Added an immutable [`segment_summary_imm`] method in the [`SomeTree`] trait.

[`example_data`]: https://docs.rs/grove/0.2.0/grove/data/example_data/index.html
[`Keyed`]: https://docs.rs/grove/0.2.0/grove/data/example_data/trait.Keyed.html
[`to_summary`]: https://docs.rs/grove/0.2.0/grove/data/trait.ToSummary.html#tymethod.to_summary
[`Data`]: https://docs.rs/grove/0.2.0/grove/data/trait.Data.html
[`SomeTree`]: https://docs.rs/grove/0.2.0/grove/trees/trait.SomeTree.html
[`segment_summary_imm`]: https://docs.rs/grove/0.2.0/grove/trees/trait.SomeTree.html#tymethod.segment_summary_imm

# Version 0.1.0
The first version of the library.