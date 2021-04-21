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

    /*
    fn sum(&self) -> I {
        let mut a = self.start; let mut b = self.end;
        if a > b {
            std::mem::swap(&mut a, &mut b);
        }
        b += 1;
        let res = b*(b-1)/2 - a*(a-1)/2;
        res % MODULUS
    }
    */

    // sum of i*A[i] if the array only starts at index `index` 
    fn sum_with_index(&self, index2 : usize) -> I {
        let index = (index2 as I) % MODULUS;
        let bMODULUS = 6*(MODULUS as i128);
        let size = (self.size() as I) % (2*MODULUS);
        let size128 = (self.size() as i128) % bMODULUS;
        // denote i = index + j
        // i*A[i] = (index + j) * (inter.start + incr*j) = incr*j^2 + j*(incr*index + inter.start) + index*inter.start
        // sum (i < n) i^2 = n*(n-1)*(2n-1)/6
        // sum (i < n) i = n*(n-1)/2
        let ap = ((size128*(size128-1)) % bMODULUS)*((2*size128-1) % bMODULUS);
        if ap % 6 != 0 {
            panic!();
        }
        let a = ((ap/6) % (MODULUS as i128)) as i64;
        let b = if self.start < self.end {self.start + index} else {self.start - index} % MODULUS;
        let c = index*(self.start % MODULUS) % MODULUS;

        let xp = size*(size-1);
        assert!(xp % 2 == 0);
        let x = ((xp/2) % MODULUS) as i64;

        let mut res = if self.start < self.end {a + b*x + c*size} else { b*x + c*size - a};
        res %= MODULUS;


        res
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

struct RevData {}

impl Data for RevData {
    type Action = RevAction;
    type Summary = Size;
    type Value = Interval;

    const IDENTITY : RevAction = RevAction { to_reverse : false };
    const EMPTY : Size = Size {size : 0};

    fn compose_a(a : Self::Action, b : Self::Action) -> Self::Action {
        RevAction { to_reverse : a.to_reverse != b.to_reverse }
    }

    fn compose_s(s1 : Size, s2 : Size) -> Size {
        Size {size : s1.size + s2.size }
    }

    fn act_value(act : Self::Action, val : &mut Interval) {
        if act.to_reverse {
            val.flip();
        }
    }

    fn to_summary(val : &Interval) -> Size {
        Size {size : val.size()}
    }

    fn to_reverse(act : Self::Action) -> bool {
        act.to_reverse
    }
}

impl Reverse for RevData {
    fn internal_reverse(node : &mut basic_tree::BasicNode<Self>) {
        node.act(RevAction{to_reverse : true})
    }
}

impl SizedData for RevData {
    fn size(summary : Self::Summary) -> usize {
        summary.size
    }
}

impl SplayTree<RevData> {
    // splits a segment inside the tree
    fn search_split(&mut self, index : usize) {
        let locator = locate_by_index_range(index, index);
        let mut walker = // using an empty range so that we'll only end up at a node
            // if we actually need to split that node
            search_by_locator(self, &locator).void_unwrap();
        

        let left = walker.left_summary().size;
        if let Some(val) = walker.value_mut() { // if we need to split a node
            let (v1, v2) = val.split_at_index(index - left);
            *val = v1;
            next_empty(&mut walker).unwrap(); // not at an empty node
            walker.insert_new(v2).unwrap(); // the position must be empty
        }
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
        self.search_split(low);
        // high-low and not high since this counts the index based on the split tree and not the original tree
        self.search_split(high);
        self.act_on_segment(locate_by_index_range(low, high), RevAction { to_reverse : true });
    }
}


pub fn yarra(n : usize, k : usize) -> I {
    let mut tree : SplayTree<RevData> = SplayTree::new();
    let inter = Interval {start : 0, end : (n-1) as I};
    tree.walker().insert_new(inter).unwrap();

    let mut sn = 1;
    let mut tn = 1;
    for round in 0..k {
        
        if round < 10 {
            dbg!(round);
            dbg!(to_array(&mut tree));
        }

        if round % 2000 == 0 {
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

    let arr = to_array(&mut tree);
    let mut index = 0;
    let mut index_sum = 0;
    for inter in arr {
        index_sum += inter.sum_with_index(index);
        index_sum %= MODULUS;
        index += inter.size();
        if index % 1000 == 0 {
            dbg!(index);
        }
    }
    dbg!(index_sum);
    drop(tree);
    println!("done drop");
    index_sum
}


pub fn test() {
    println!("Hello, world!");

    let res = yarra(1000_000_000_000_000_000, 1000_000);
    //let res = yarra(10000, 10000);
    println!("{:?}", res);
    dbg!(res);
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


