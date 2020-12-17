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
pub struct Size {
    pub size : usize
}

impl Data for Size {
    fn rebuild_data<'a>(&'a mut self, left : Option<&'a Self>, right : Option<&'a Self>) {
        self.size = 1;
        self.size += left.map_or(0, |r| r.size);
        self.size += right.map_or(0, |r| r.size);
    }

    fn access<'a>(&'a mut self, _ : Option<&'a mut Self>, _ : Option<&'a mut Self>) {}
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

pub struct Key<K> {
    pub key : K,
}

impl<K : std::cmp::Ord> Data for Key<K> {
    fn rebuild_data<'a>(&'a mut self, _ : Option<&'a Self>, _ : Option<&'a Self>) {}
    fn access<'a>(&'a mut self, _ : Option<&'a mut Self>, _ : Option<&'a mut Self>) {}
}

pub trait Keyed : Data where {
    type Key : std::cmp::Ord;
    fn get_key(&self) -> &<Self as Keyed>::Key;
}

// the convention is that smaller values go on the left
impl<K : std::cmp::Ord> Keyed for Key<K> {
    type Key = K;
    fn get_key(&self) -> &K {
        &self.key
    }
}