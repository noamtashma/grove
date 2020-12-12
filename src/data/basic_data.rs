use super::*;

// straight values. no bookkeeping needed.
struct Value<T> {
    val : T,
}

impl<T> Data for Value<T> {
    fn rebuild_data<'a>(&'a mut self, _ : Option<&'a Self>, _ : Option<&'a Self>) {}
    fn access<'a>(&'a mut self, _ : Option<&'a mut Self>, _ : Option<&'a mut Self>) {}
}

// storing the size of a subtree
// assumes that storing the size of the structure in a usize is enough.
// if it's enough for all the addresses in the computers... it must always be enough, right? right?
struct Size {
    size : usize
}

impl Data for Size {
    fn rebuild_data<'a>(&'a mut self, left : Option<&'a Self>, right : Option<&'a Self>) {
        self.size = 1;
        self.size += match left {
            None => 0,
            Some(r) => r.size,
        };
        self.size += match right {
            None => 0,
            Some(r) => r.size,
        };
    }

    fn access<'a>(&'a mut self, _ : Option<&'a mut Self>, _ : Option<&'a mut Self>) {}
}


// the height of a subtree
struct Height {
    height : usize
}

impl Data for Height {
    fn rebuild_data<'a>(&'a mut self, left : Option<&'a Self>, right : Option<&'a Self>) {
        let lh = match left {
            None => 0,
            Some(r) => r.height,
        };
        let rh = match right {
            None => 0,
            Some(r) => r.height,
        };
        self.height = 1 + std::cmp::max(lh, rh);
    }
    fn access<'a>(&'a mut self, _ : Option<&'a mut Self>, _ : Option<&'a mut Self>) {}
}


// ordering keys
// similar to Value<K>, but implements the keying trait. TODO

struct Key<K> {
    key : K,
}

impl<K : std::cmp::Ord> Data for Key<K> {
    fn rebuild_data<'a>(&'a mut self, _ : Option<&'a Self>, _ : Option<&'a Self>) {}
    fn access<'a>(&'a mut self, _ : Option<&'a mut Self>, _ : Option<&'a mut Self>) {}
}

trait Keyed : Data where {
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