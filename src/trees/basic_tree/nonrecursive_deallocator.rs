use super::*;
struct NonrecursiveDeallocator<D : Data, T> {
    stack : Vec<Box<BasicNode<D, T>>>,
}

impl<D : Data, T> NonrecursiveDeallocator<D, T> {
    fn step(&mut self) -> Option<()> {
        let node = self.stack.pop()?;
        self.push(node.left);
        self.push(node.right);
        Some(())
    }

    fn push(&mut self, tree : BasicTree<D, T>) {
        if let Some(node) = tree.into_node_boxed() {
            self.stack.push(node);
        }
    }
}

/// Replaces the tree with an empty tree, and deallocates the tree iteratively
pub fn deallocate_nonrecursive<D : Data, T>(tree : &mut BasicTree<D, T>) {
    let my_tree = std::mem::replace(tree, BasicTree::new());
    let mut deallocator = NonrecursiveDeallocator { stack : vec![] };
    deallocator.push(my_tree);
    while let Some(_) = deallocator.step()
        {}
}