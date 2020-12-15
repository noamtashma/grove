use super::trees::splay::*;
use super::trees::SomeTree;
use super::data::example_data::*;

// an example:



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
