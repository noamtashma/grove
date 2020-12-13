pub mod tree_base;
pub mod telescope;
pub mod trees;
pub mod data;

// an example:
fn main() {
    println!("Hello, world!");
    use trees::splay::*;
    use data::basic_data::*;

    let mut tree = SplayTree::new();
    for x in 1..25 {
        tree.insert(Key {key : x*5})
    }

    for x in 1..30 {
        tree.search(&x);
        println!("{}", tree.data().unwrap().key);
        if x % 5 == 0 && x != tree.data().unwrap().key {
            panic!();
        }
    }
}
