use void::ResultVoidExt;

use crate::*;

use super::trees::splay::*;
use super::trees::*;
use super::data::*;
use super::data::example_data::*;

// an example:



// inclusive intervals
#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
struct RAValue {
    size : usize,
    start_index : usize,
    sum : usize,
    // sum of i*a[i] for i in [start_index .. start_index+size]
    index_sum : usize,
    interval : Interval, // always with the same size. has meaning only in node values.
}

impl RAValue {
    pub fn from_interval(start_index : usize, interval : Interval) -> RAValue {
        RAValue {
            size : interval.size(),
            start_index,
            sum : interval.sum_with_index(1) - interval.sum_with_index(0), // TODO
            index_sum : interval.sum_with_index(start_index),
            interval,
        }
    }

    pub fn split_at_index(&self, index: usize) -> (Self, Self) {
        assert!(self.interval.size() > 0); // this must be a node value
        assert!(index > self.start_index && index <= self.start_index + self.size);
        let mut v1 = *self;
        let mut v2 = *self;
        let (i1, i2) = self.interval.split_at_index(index - self.start_index);
        v1.interval = i1;
        v2.interval = i2;
        (v1, v2)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct RAAction {
    // in the Err case, this is a reversal.
    // the value contains 2*the center of a reversal operation.
    // i.e., the operation switches A[i] to A[center - i] for all i.

    // in the Ok case, this is a shift.
    // i.e., the operation switches A[i] to A[i+c] for all i.

    // doesn't contain bounds because the action will be applied in the bounds only.
    transformation : Result<usize, usize>,
}

impl Action for RAAction {
    type Value = RAValue;
    fn compose_a(self, other : Self) -> RAAction {
        let new_transformation = match (self.transformation, other.transformation) {
            (Ok(a), Ok(b)) => Ok(a+b),
            (Ok(a), Err(b)) => Err(a+b),
            (Err(a), Ok(b)) => Err(a-b),
            (Err(a), Err(b)) => Ok(a-b),
        };
        RAAction { transformation : new_transformation }
    }
    const IDENTITY : Self = RAAction{transformation : Ok(0)};
    
    fn compose_v(left : Self::Value, right : Self::Value) -> Self::Value {
        assert!(left.size == 0 || right.size == 0 ||
            left.start_index + left.size == right.start_index); // TODO
        RAValue {
            size : left.size + right.size,
            sum : left.sum + right.sum,
            start_index : if left.size != 0 {left.start_index} else {right.start_index}, // for the case of the empty value
            index_sum : left.index_sum + right.index_sum,
            interval : Interval::Increasing(0, 0), // some default interval
        }
    }
    const EMPTY : RAValue = RAValue {
        size : 0,
        start_index : 0, // random value
        sum : 0,
        index_sum : 0,
        interval : Interval::Increasing(0, 0),
    };

    fn act(self, mut val : Self::Value) -> Self::Value {
        match self.transformation {
            Ok(a) => {
                val.start_index += a;
                val.index_sum += a*val.sum;
                val
            },
            Err(a) => {
                let end_index = val.start_index + val.size;
                let new_start_index = a - end_index;
                val.index_sum = (val.start_index + new_start_index)*val.sum - val.index_sum;
                val.start_index = new_start_index;
                val.interval.flip();
                val
            },
        }
    }

}
/*
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
*/
impl SizedAction for RAAction {
    fn size(val : RAValue) -> usize {
        val.size
    }
}

impl SplayTree<RAAction> {
    // splits the tree - modifies the first tree and returns the second tree
    // splits to [0, index) and [index, length)
    fn search_split<'a>(&'a mut self, mut index : usize) -> SplayTree<RAAction> {

        let mut locator = locate_by_index_range(index, index);
        let mut walker = // using an empty range so that we'll only end up at a node
            // if we actually need to split that node
            search_by_locator(self, &locator).void_unwrap();
        

        if let Some(val) = walker.value_mut() { // if we need to split a node
            let (v1, v2) = val.split_at_index(index);
            *val = v1;
            next_empty(&mut walker).unwrap(); // not at an empty node
            walker.insert_new(v2).unwrap(); // the position must be empty
            walker.go_left().unwrap();
        }
        return walker.split().unwrap();
    }

    // reverse the segment [low, high)
    fn reverse_segment(&mut self, low : usize, high : usize) {
        let mut self2 = self.search_split(low);
        let self3 = self2.search_split(high);
        //self2.root_data_mut().map(|r| r.reverse());
        // TODO
    }
}



pub fn test() {
    println!("Hello, world!");

    let mut tree : SplayTree<KeyAction<_>> = SplayTree::new();
    for x in 1..25 {
        insert_by_key(&mut tree, Key::new(x*5));
    }

    for x in 1..30 {
        search(&mut tree, &x);
        println!("{:?}, {:?}", x, tree.root_node_value().unwrap().key);
        if x % 5 == 0 { 
            if x != tree.root_node_value().unwrap().key.unwrap() {
            panic!();
            }
            else {
                dbg!("success!", x);
            }
        }
    }
    println!("done!");
}


