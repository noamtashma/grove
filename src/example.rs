use void::ResultVoidExt;

use crate::*;

use super::trees::splay::*;
use super::trees::*;
use super::data::*;
use super::data::example_data::*;

// an example:

// inclusive intervals
enum Interval {
    Increasing(usize , usize),
    Decreasing(usize, usize),
}

impl Interval {
    fn size(&self) -> usize {
        unimplemented!()
    }

    fn flip(&mut self) {
        unimplemented!()
    }
    fn sum_with_index(&self, index : usize) -> usize {
        unimplemented!()
    }

    fn split_at_index(&self, index : usize) -> (Interval, Interval) {
        unimplemented!()
    }
}

struct RAData {
    size : Size,
    should_reverse : bool,
    interval : Interval,
}

impl RAData {
    fn new(i : Interval) -> Self {
        RAData {size : Size {size : 0}, should_reverse : false, interval : i}
    }
}

impl Data for RAData {
    fn rebuild_data<'a>(&'a mut self, left : Option<&'a Self>, right : Option<&'a Self>) {
        self.size.rebuild_data(left.map(|r| &r.size), right.map(|r| &r.size));
        self.size.size += self.interval.size();
    }

    fn access<'a>(&'a mut self, left : Option<&'a mut Self>, right : Option<&'a mut Self>) {
        // actually a no-op
        // left here for completeness
        self.size.access(left.map(|r| &mut r.size), right.map(|r| &mut r.size));
    }

    fn to_reverse(&mut self) -> bool {
        let res = self.should_reverse;
        self.should_reverse = false;
        return res;
	}

    fn reverse(&mut self) {
        self.should_reverse = !self.should_reverse;
        self.interval.flip();
	}
}

impl SizedData for RAData {
    fn size(&self) -> usize {
        self.size.size()
    }

    fn width(&self) -> usize {
        self.interval.size()
    }
}

impl SplayTree<RAData> {
    // splits the tree - modifies the first tree and returns the second tree
    // splits to [0, index) and [index, length)
    fn search_split<'a>(&'a mut self, mut index : usize) -> SplayTree<RAData> {

        let mut locator = locate_by_index_range(index, index);
        let mut walker = // using an empty range so that we'll only end up at a node
            // if we actually need to split that node
            search_by_locator(self, &mut locator).void_unwrap();
        index = locator.expose().0;

        if let Some(data) = walker.data_mut() { // if we need to split a node
            let (i1, i2) = data.interval.split_at_index(index);
            data.interval = i1;
            next_empty(&mut walker).unwrap(); // not at an empty node
            walker.insert_new(RAData::new(i2)).unwrap(); // the position must be empty
            walker.go_left().unwrap();
        }
        // TODO: call split by ourselves
        return walker.split().unwrap();
    }

    // reverse the segment [low, high)
    fn reverse_degment(&mut self, low : usize, high : usize) {
        let mut self2 = self.search_split(low);
        let self3 = self.search_split(high);
        self2.root_data_mut().map(|r| r.reverse());
        
    }
}



pub fn main() {
    println!("Hello, world!");

    let mut tree = SplayTree::new();
    for x in 1..25 {
        insert(&mut tree, Key {key : x*5});
    }

    for x in 1..30 {
        search(&mut tree, &x);
        println!("{}", tree.root_data().unwrap().key);
        if x % 5 == 0 && x != tree.root_data().unwrap().key {
            panic!();
        }
    }
}
