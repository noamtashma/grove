use void::ResultVoidExt;

use crate::*;

use super::trees::splay::*;
use super::trees::*;
use super::data::*;
use super::data::example_data::*;

const MODULUS : I = 1_000_000_000;

// an example:


type I = i64;

// inclusive intervals
#[derive(Clone, Copy, Debug)]

// either increasing or decreasing intervals. 
// i.e., {start:3, end:7} is [3,4,5,6,7],
// and {start:7, end:4} is [7,6,5,4]
struct Interval {
    start : I,
    end : I,
}

impl Interval {
    fn size(&self) -> usize {
        1 + if self.end > self.start { self.end - self.start } else { self.start - self.end } as usize
    }

    fn flip(&mut self) {
        std::mem::swap(&mut self.start, &mut self.end);
    }
    
    // sum of i*A[i] if the array only starts at index `index` 
    fn sum_with_index(&self, index2 : usize) -> I {
        let index = (index2 as I) % MODULUS;
        let inter = *self;
        let size = (inter.size() as I) % MODULUS;
        // denote i = index + j
        // i*A[i] = (index + j) * (inter.start + incr*j) = incr*j^2 + j*(incr*index + inter.start) + index*inter.start
        // sum (i < n) i^2 = n*(n-1)*(2n-1)/6
        // sum (i < n) i = n*(n-1)/2
        let a = (size*(size-1) % MODULUS)*(2*size-1)/6;
        let b = if self.start < self.end {inter.start + index} else {inter.start - index} % MODULUS;
        let c = index*(inter.start % MODULUS) % MODULUS;

        let x = (size*(size-1)/2) % MODULUS;

        let res = if self.start < self.end {a + b*x + c*size} else { b*x + c*size - a};
        return res % MODULUS;
    }

    // split into the index first values and the rest. i.e.,
    // spliting [6,5,4] with index=1 gives [6], [5,4]
    fn split_at_index(&self, index2 : usize) -> (Interval, Interval) {
        let index = index2 as I;
        assert!(0 < index2 && index2 < self.size());
        if self.start <= self.end {
            (Interval {end : self.start + index - 1, ..*self}, Interval {start : self.start + index, ..*self})
        } else {
            (Interval {end : self.start - index + 1, ..*self}, Interval {start : self.start - index, ..*self})
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct RAValue {
    size : usize,
    start_index : usize,
    sum : I,
    // sum of i*a[i] for i in [start_index .. start_index+size]
    index_sum : I,
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

    // TODO - this doesn't update the values!
    pub fn split_at_index(&self, index: usize) -> (Self, Self) {
        assert!(self.interval.size() > 0); // this must be a node value
        assert!(index > self.start_index && index <= self.start_index + self.size);

        let (i1, i2) = self.interval.split_at_index(index - self.start_index);
        let v1 = RAValue::from_interval(self.start_index, i1);
        let v2 = RAValue::from_interval(self.start_index + i1.size(), i2);
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
    transformation : Result<I, I>,
}

impl Action for RAAction {
    type Summary = RAValue;
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
    
    fn compose_s(left : Self::Summary, right : Self::Summary) -> Self::Summary {
        assert!(left.size == 0 || right.size == 0 ||
            left.start_index + left.size == right.start_index); // TODO
        RAValue {
            size : left.size + right.size,
            sum : left.sum + right.sum % MODULUS,
            start_index : if left.size != 0 {left.start_index} else {right.start_index}, // for the case of the empty value
            index_sum : left.index_sum + right.index_sum % MODULUS,
            interval : Interval{start : 0, end : 0}, // some default interval
        }
    }
    const EMPTY : RAValue = RAValue {
        size : 0,
        start_index : 0, // random value
        sum : 0,
        index_sum : 0,
        interval : Interval{start : 0, end : 0},
    };

    fn act(self, mut val : Self::Summary) -> Self::Summary {
        match self.transformation {
            Ok(a) => {
                val.start_index = (val.start_index as I + a) as usize;
                val.sum %= MODULUS; // why is this needed?
                val.index_sum += ((a as I) % MODULUS)*val.sum;
                val.index_sum %= MODULUS;
                val
            },
            Err(a) => {
                let end_index = val.start_index + val.size - 1;
                let new_start_index = a as usize - end_index;
                val.sum %= MODULUS; // why is this needed?
                val.index_sum = ((a as I) % MODULUS)*val.sum - val.index_sum;
                val.index_sum %= MODULUS;
                val.start_index = new_start_index;
                val.interval.flip();
                val
            },
        }
    }

    fn to_reverse(&self) -> bool {
        match self.transformation {
            Ok(_) => false,
            Err(_) => true,
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

impl crate::data::Reverse for RAAction {
    fn internal_reverse(node : &mut crate::trees::basic_tree::BasicNode<Self>) {
        node.access(); // TODO
        let RAValue {start_index, size, ..} = node.segment_summary();
        let action = RAAction {transformation : Err((2*start_index + size - 1) as I)};
        node.act(action);
    }
}

impl SplayTree<RAAction> {
    // splits the tree - modifies the first tree and returns the second tree
    // splits to [0, index) and [index, length)
    // TODO: exmplain
    fn search_split(&mut self, index1 : usize, index2 : usize) -> SplayTree<RAAction> {

        let locator = locate_by_index_range(index1, index1);
        let mut walker = // using an empty range so that we'll only end up at a node
            // if we actually need to split that node
            search_by_locator(self, &locator).void_unwrap();
        

        if let Some(val) = walker.value_mut() { // if we need to split a node
            let (v1, v2) = val.split_at_index(index2);
            *val = v1;
            next_empty(&mut walker).unwrap(); // not at an empty node
            walker.insert_new(v2).unwrap(); // the position must be empty
            walker.go_left().unwrap();
        }
        return walker.split().unwrap();
    }

    /// Unites the two trees into one.
    fn union(&mut self, other : Self) {
        let mut walker = self.walker();
        while let Ok(_) = walker.go_right()
            {}
        match walker.go_up() {
            Err(()) => { // the tree is empty; just substiture the other tree.
                drop(walker);
                *self = other;
                return;
            },
            Ok(b) => assert!(b == false),
        };
        walker.splay();
        if let trees::basic_tree::BasicTree::Root(node) = walker.inner_mut() {
            node.right = other.into_inner();
            node.rebuild();
            return;
        }
        else {
            panic!();
        }
    }

    // reverse the segment [low, high)
    fn reverse_segment(&mut self, low : usize, high : usize) {
        let mut self2 = self.search_split(low, low);
        // high-low and not high since this counts the index based on the split tree and not the original tree
        let self3 = self2.search_split(high-low, high);
        self2.reverse();
        // unite back together
        self2.union(self3);
        self.union(self2);
    }
}


pub fn yarra(n : usize, k : usize) -> I {
    let mut tree : SplayTree<RAAction> = SplayTree::new();
    let inter = Interval {start : 0, end : (n-1) as I};
    tree.walker().insert_new(RAValue::from_interval(0, inter)).unwrap();

    let mut sn = 1;
    let mut tn = 1;
    for round in 0..k {
        if round % 1000 == 999 {
            //dbg!(tree.segment_value());
            dbg!(round);
        }

        if sn < tn {
            tree.reverse_segment(sn, tn+1);
        } else if sn > tn {
            tree.reverse_segment(tn, sn+1);
        }

        sn += tn;
        sn %= n;
        tn += sn;
        tn %= n;
    }

    let res = tree.segment_value().index_sum;
    res
}


pub fn test() {
    println!("Hello, world!");

    dbg!(yarra(1000_000_000_000_000_000, 1000_000));
    //dbg!(yarra(15, 10));
    use std::io::Write;
    use std::io::stdout;
    stdout().flush().unwrap();

    /*
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
    */
}


