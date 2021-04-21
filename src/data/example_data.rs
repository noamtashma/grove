use super::*;


/// storing the size of a subtree
/// assumes that storing the size of the structure in a usize is enough.
/// if it's enough for all the addresses in the computers... it must always be enough, right? right?
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Size {
    pub size : usize
}
#[derive(PartialEq, Eq, Copy, Clone)]
struct SizeData<V> {phantom : PhantomData<V>}

// TODO: remove the Eq + Copy requirement
impl<V : Eq + Copy> Data for SizeData<V> {
    type Action = ();
    type Summary = Size;
    type Value = V;
    fn compose_a(_ : (), _ : ()) -> () {
        ()
    }
    const IDENTITY : Self::Action = ();
    fn compose_s(a : Size, b : Size) -> Size {
        Size { size : a.size + b.size }
    }
    const EMPTY : Size = Size {size : 0};
    fn act(_ : (), b : Size) -> Size { b }
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


/// the height of a subtree
pub struct Height {
    pub height : usize
}


// ordering keys
// similar to Value<K>, but implements the keying trait. TODO



// TODO: complete
use std::marker::PhantomData;
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct NoAction<V> {
    phantom : PhantomData<V>,
}
impl<V> NoAction<V> {
    pub fn new() -> NoAction<V> {
        NoAction {phantom : PhantomData}
    }
}

impl<V : Eq + Copy> Data for NoAction<V> {
    type Summary = ();
    type Action = ();
    type Value = V;
    const IDENTITY : Self::Action = ();
    fn compose_a( _ : Self::Action, _ : Self::Action) -> () {
        ()
    }

    const EMPTY : () = ();
    fn compose_s(_left : (), _right : ()) -> () {
        ()
    }

    fn to_summary(_val : &Self::Value) -> Self::Summary {
        ()
    }
}

/// The convention is that smaller values go on the left
pub trait Keyed {
    type Key : std::cmp::Ord;
    fn get_key(&self) -> <Self as Keyed>::Key;
}


/*
impl<K : std::cmp::Ord> Data for Key<K> {
    fn rebuild_data<'a>(&'a mut self, _ : Option<&'a Self>, _ : Option<&'a Self>) {}
    fn access<'a>(&'a mut self, _ : Option<&'a mut Self>, _ : Option<&'a mut Self>) {}
}


pub trait KeyedData : Data where {
    type Key : std::cmp::Ord;
    fn get_key(&self) -> &<Self as KeyedData>::Key;
}

impl<K : std::cmp::Ord> KeyedData for Key<K> {
    type Key = K;
    fn get_key(&self) -> &K {
        &self.key
    }
}
*/