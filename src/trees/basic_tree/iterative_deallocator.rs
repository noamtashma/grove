use super::*;

/// The auto-generated deallocation code for [`BasicTree`] is recursive.
/// Since splay trees can have arbitrary depth, a problem arose where deallocating a large
/// splay tree could cause a stack overflow.
///
/// Therefore, we have this tiny struct in order to deallocate a [`BasicTree`] in an iterative way.
/// From the user's perspective this is a function from the `basic_tree` module.
struct IterativeDeallocator<D: Data, T> {
    stack: Vec<Box<BasicNode<D, T>>>,
}

impl<D: Data, T> IterativeDeallocator<D, T> {
    fn step(&mut self) -> Option<()> {
        let node = self.stack.pop()?;
        self.push(node.left);
        self.push(node.right);
        Some(())
    }

    fn push(&mut self, tree: BasicTree<D, T>) {
        if let Some(node) = tree.into_node_boxed() {
            self.stack.push(node);
        }
    }
}

/// Replaces the tree with an empty tree, and deallocates the tree iteratively.
/// Input is a reference and not an owned value so that this funcction can get
/// called in `Drop` implementations.
pub fn deallocate_iteratively<D: Data, T>(tree: &mut BasicTree<D, T>) {
    let my_tree = std::mem::replace(tree, BasicTree::new());
    let mut deallocator = IterativeDeallocator { stack: vec![] };
    deallocator.push(my_tree);
    while let Some(_) = deallocator.step() {}
}
