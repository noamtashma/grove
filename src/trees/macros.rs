#[macro_use]

/// deriving SomeWalker by an inner walker
/// format is:
///```
/// derive_SomeWalker!{walker,
///     impl<'a, D: Data> SomeWalker<D> for TreapWalker<'a, D> {
///         fn go_up(&mut self) -> Result<Side, ()> {
///             ...
///         }
///     }
/// }
///```
/// expects the `go_up` method to be implemented
macro_rules! derive_SomeWalker {
    ($accessor:ident, impl<$lifetime:lifetime, $data:ident: Data> SomeWalker<D> for $self:ty
        { $($token:tt)* }
    ) => {
        impl<$lifetime, $data: Data> SomeWalker<$data> for $self {
            fn go_left(&mut self) -> Result<(), ()> {
                self.$accessor.go_left()
            }

            fn go_right(&mut self) -> Result<(), ()> {
                self.$accessor.go_right()
            }

            fn depth(&self) -> usize {
                self.$accessor.depth()
            }

            fn far_left_summary(&self) -> $data::Summary {
                self.$accessor.far_left_summary()
            }

            fn far_right_summary(&self) -> $data::Summary {
                self.$accessor.far_left_summary()
            }

            fn value(&self) -> Option<& $data::Value> {
                self.$accessor.value()
            }

            $($token)*
        }
    }
}
/// deriving SomeWalker by an inner walker
/// format is:
///```
/// derive_SomeEntry!{walker,
///     impl<'a, D: Data> SomeEntry<D> for TreapWalker<'a, D> {
///         fn assert_correctness_locally(&self)
///         where
///             D::Summary: Eq,
///         {
///             ...
///         }
///     }
/// }
///```
/// expects the `assert_correctness_locally` method to be implemented
macro_rules! derive_SomeEntry {
    ($accessor:ident, impl <$($lifetime:lifetime,)? $data:ident : Data> SomeEntry<D> for $self:ty
        { $($token:tt)* }
    ) => {
        impl<$($lifetime,)? $data : Data> SomeEntry<$data> for $self {
            fn with_value<F, R>(&mut self, f: F) -> Option<R>
            where
                F: FnOnce(&mut D::Value) -> R,
            {
                self.$accessor.with_value(f)
            }

            fn node_summary(&self) -> D::Summary {
                self.$accessor.node_summary()
            }

            fn subtree_summary(&self) -> D::Summary {
                self.$accessor.subtree_summary()
            }

            fn left_subtree_summary(&self) -> Option<D::Summary> {
                self.$accessor.left_subtree_summary()
            }

            fn right_subtree_summary(&self) -> Option<D::Summary> {
                self.$accessor.right_subtree_summary()
            }

            fn act_subtree(&mut self, action: D::Action) {
                self.$accessor.act_subtree(action);
            }

            fn act_node(&mut self, action: D::Action) -> Option<()> {
                self.$accessor.act_node(action)
            }

            fn act_left_subtree(&mut self, action: D::Action) -> Option<()> {
                self.$accessor.act_left_subtree(action)
            }

            fn act_right_subtree(&mut self, action: D::Action) -> Option<()> {
                self.$accessor.act_right_subtree(action)
            }

            $($token)*
        }
    }
}
