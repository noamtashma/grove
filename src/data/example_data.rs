use super::*;
use std::marker::PhantomData;


/// Storing the size of a subtree.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Size {
    pub size : usize
}

impl Add for Size {
    type Output = Size;
    fn add(self, b : Size) -> Size {
        Size {size : self.size + b.size}
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
struct SizeData<V> {phantom : PhantomData<V>}

impl<V> Data for SizeData<V> {
    type Action = Unit;
    type Summary = Size;
    type Value = V;
    const IDENTITY : Self::Action = Unit{};
    const EMPTY : Size = Size {size : 0};
    fn act(_ : Unit, b : Size) -> Size { b }
    fn to_summary(_val : &Self::Value) -> Self::Summary {
        Size {size : 1}
    }
}

/// actions in which action::Value keeps track of the size of subtrees.
pub trait SizedData : Data {
    /// The size of the subtree of the current node
    fn size(val : Self::Summary) -> usize;
}

impl<V : Eq + Copy> SizedData for SizeData<V> {
    fn size(val : Size) -> usize { val.size }
}




#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct NoAction<V> {
    phantom : PhantomData<V>,
}

impl<V : Eq + Copy> Data for NoAction<V> {
    type Summary = Unit;
    type Action = Unit;
    type Value = V;
    const IDENTITY : Self::Action = Unit{};

    const EMPTY : Unit = Unit{};

    fn to_summary(_val : &Self::Value) -> Self::Summary {
        Unit{}
    }
}

// TODO: consider retiring this and just requiring Value : Ord instead.
/// The convention is that smaller values go on the left
pub trait Keyed {
    type Key : std::cmp::Ord;
    fn get_key(&self) -> <Self as Keyed>::Key;
}