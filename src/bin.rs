use orchard::example::*;

pub fn main() {
    
    use orchard::splay::*;
    use orchard::example_data::StdNum;
    
    let mut tree : SplayTree<StdNum> = (17..=89).collect();
    let tree2 : SplayTree<StdNum> = (13..=25).collect();
    tree.concatenate(tree2);
    
    assert_eq!(tree.iter().cloned().collect::<Vec<_>>(), (17..=89).chain(13..=25).collect::<Vec<_>>());
    tree.assert_correctness();
    

    test();
}