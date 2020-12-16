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

impl SplayTree<RAData> {
    // does not splay by itself
    fn search_split<'a>(&'a mut self, mut index : usize) -> SplayWalker<'a, RAData> {
        let mut walker = self.walker();
        while let crate::trees::basic_tree::Tree::Root(node) = walker.inner_mut() {
            
            let s = node.left.data_mut().map_or(0, |r| r.size.size);
            if index < s {
                walker.go_left();
            } else if index >= s + node.interval.size() {
                index -= s + node.interval.size();
                walker.go_right();
            } else { // this node is the right node
                index -= s;
                break;
            }
        }
        
        let data = walker.data_mut().expect("index not found!");
        let (i1, i2) = data.interval.split_at_index(index);
        data.interval = i1;
        // walker.next();
        walker.go_right(); // TODO - use the results
        while !walker.is_empty() {
            walker.go_left().unwrap();
        }
        walker.insert_new(RAData::new(i2));
        return walker;

        // TODO: edgecases
    }
}



pub fn main() {
    println!("Hello, world!");

    let mut tree = SplayTree::new();
    for x in 1..25 {
        tree.insert(Key {key : x*5})
    }

    for x in 1..30 {
        tree.search(&x);
        println!("{}", tree.root_data().unwrap().key);
        if x % 5 == 0 && x != tree.root_data().unwrap().key {
            panic!();
        }
    }
}
