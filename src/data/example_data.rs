use super::*;

/// straight values. no bookkeeping needed.
pub struct Value<T> {
    pub val : T,
}

impl<T> Data for Value<T> {
    fn rebuild_data<'a>(&'a mut self, _ : Option<&'a Self>, _ : Option<&'a Self>) {}
    fn access<'a>(&'a mut self, _ : Option<&'a mut Self>, _ : Option<&'a mut Self>) {}
}

/// storing the size of a subtree
/// assumes that storing the size of the structure in a usize is enough.
/// if it's enough for all the addresses in the computers... it must always be enough, right? right?
#[derive(Clone, Copy)]
pub struct Size {
    pub size : usize
}
#[derive(PartialEq, Eq, Copy, Clone)]
struct SizeAction {}

impl Data for Size {
    fn rebuild_data<'a>(&'a mut self, left : Option<&'a Self>, right : Option<&'a Self>) {
        self.size = 1;
        self.size += left.map_or(0, |r| r.size);
        self.size += right.map_or(0, |r| r.size);
    }

    fn access<'a>(&'a mut self, _ : Option<&'a mut Self>, _ : Option<&'a mut Self>) {}
}

impl Action for SizeAction {
    type Value = Size;
    fn compose_a(self : SizeAction, _ : SizeAction) -> SizeAction {
        SizeAction {}
    }
    const IDENTITY : Self = SizeAction {};
    fn compose_v(a : Size, b : Size) -> Size {
        Size { size : a.size + b.size }
    }
    const EMPTY : Size = Size {size : 0};
    fn act(self : SizeAction, b : Size) -> Size { b }
}

/// actions in which action::Value keeps track of the size of subtrees.
pub trait SizedAction : Action {
    /// The size of the subtree of the current node
    fn size(val : Self::Value) -> usize;
}

impl SizedAction for SizeAction {
    fn size(val : Size) -> usize { val.size }
}

pub trait SizedData : Data {
    /// The size of the subtree of the current node
    fn size(&self) -> usize;

    // TODO: should we keep the option of wide values?
    /// The "width" of the current element alone.
    /// The default implementation always returns 1.
    fn width(&self) -> usize {
        1
    }
}

impl SizedData for Size {
    fn size(&self) -> usize { self.size }
}

/// the height of a subtree
pub struct Height {
    pub height : usize
}

impl Data for Height {
    fn rebuild_data<'a>(&'a mut self, left : Option<&'a Self>, right : Option<&'a Self>) {
        let lh = left.map_or(0, |r| r.height);
        let rh = right.map_or(0, |r| r.height);
        self.height = 1 + std::cmp::max(lh, rh);
    }
    fn access<'a>(&'a mut self, _ : Option<&'a mut Self>, _ : Option<&'a mut Self>) {}
}


// ordering keys
// similar to Value<K>, but implements the keying trait. TODO

#[derive(Clone, Copy, Debug)]
pub struct Key<K> {
    pub key : Option<K>,
}

impl<K> Key<K> {
    pub fn new(key : K) -> Key<K> {
        Key {key : Some(key)}
    }
}

// TODO: complete
use std::marker::PhantomData;
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct KeyAction<K> {
    phantom : PhantomData<K>,
}
impl<K> KeyAction<K> {
    pub fn new() -> KeyAction<K> {
        KeyAction {phantom : PhantomData}
    }
}

impl<K:Eq + Copy> Action for KeyAction<K> {
    type Value = Key<K>;
    const IDENTITY : Self = KeyAction {phantom : PhantomData};
    fn compose_a(self, _ : Self) -> Self {KeyAction::new()}

    const EMPTY : Key<K> = Key {key : None};
    fn compose_v(left : Key<K>, right : Key<K>) -> Key<K> {
        match left.key {
            None => right,
            Some(_) => match right.key {
                None => left,
                Some(_) => Key{key : None},
            }
        }
    }
}

/// The convention is that smaller values go on the left
pub trait Keyed : Action {
    type Key : std::cmp::Ord;
    fn get_key(val : <Self as Action>::Value) -> <Self as Keyed>::Key;
}

impl<K : std::cmp::Ord + Copy> Keyed for KeyAction<K> {
    type Key = K;
    fn get_key(val : Key<K>) -> K {
        val.key.unwrap()
    }
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