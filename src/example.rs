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

#[derive(PartialEq, Eq, Clone, Copy)]
struct RevAction {
    to_reverse : bool,
}

impl Action for RevAction {
    type Summary = Size;
    type Value = Interval;

    const IDENTITY : RevAction = RevAction { to_reverse : false };
    const EMPTY : Size = Size {size : 0};

    fn compose_a(self, other : Self) -> Self {
        RevAction { to_reverse : self.to_reverse != other.to_reverse }
    }

    fn compose_s(s1 : Size, s2 : Size) -> Size {
        Size {size : s1.size + s2.size }
    }

    fn act_value(self, val : &mut Interval) {
        if self.to_reverse {
            val.flip();
        }
    }

    fn to_summary(val : &Interval) -> Size {
        Size {size : val.size()}
    }
}

impl Reverse for RevAction {
    fn internal_reverse(node : &mut basic_tree::BasicNode<Self>) {
        node.act(RevAction{to_reverse : true})
    }
}

impl SizedAction for RevAction {
    fn size(summary : Self::Summary) -> usize {
        summary.size
    }
}

impl SplayTree<RevAction> {
    // splits the tree - modifies the first tree and returns the second tree
    // splits to [0, index) and [index, length)
    // TODO: exmplain
    fn search_split(&mut self, index1 : usize, index2 : usize) -> SplayTree<RevAction> {

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
    let mut tree : SplayTree<RevAction> = SplayTree::new();
    let inter = Interval {start : 0, end : (n-1) as I};
    tree.walker().insert_new(inter).unwrap();

    let mut sn = 1;
    let mut tn = 1;
    for round in 0..k {
        if round < 10 {
            dbg!(round);
            dbg!(to_array(&mut tree));
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

    let arr = to_array(&mut tree);
    let mut index = 0;
    let mut index_sum = 0;
    for inter in arr {
        index_sum += inter.sum_with_index(index);
        index_sum %= MODULUS;
        index += inter.size();
    }

    index_sum
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


