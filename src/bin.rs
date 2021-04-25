use orchard::example::*;

pub fn main() {
    
    use orchard::basic_tree::*;
    use orchard::example_data::StdNum;
    use orchard::locator;
    let mut tree : BasicTree<StdNum> = (20..80).collect();
    let part = tree.iter_locator(locator::locate_by_index_range(3,13)); // should also try 3..5
    assert_eq!(part.cloned().collect::<Vec<_>>(), (23..33).collect::<Vec<_>>());
    tree.assert_correctness();
    

    // test();
}