use crate::*;
use basic_tree::*;
enum Fragment<'a, D : Data> {
    Value (&'a mut D::Value),
    Node (&'a mut BasicNode<D>)
}

struct MutIterator<'a, D : Data, L : Locator<D>> {
    left : D::Summary,
    // a stack of the fragments, and for every fragment,
    // the summary of everything to its right
    stack : Vec<(Fragment<'a, D>, D::Summary)>,
    locator : L,
}

impl<'a, D : Data, L : Locator<D>> MutIterator<'a, D, L> {


    // same as stack.push(...), but deals with the [`Empty`] case.
    fn push(&mut self, tree : &'a mut BasicTree<D>, summary : D::Summary) {
        match tree {
            BasicTree::Root(node) =>
                self.stack.push((Fragment::Node(node), summary)),
            BasicTree::Empty => (),
        }
    }
}

impl<'a, D : Data, L : Locator<D>> Iterator for MutIterator<'a, D, L> {
    type Item = &'a mut D::Value;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (frag, summary) = match self.stack.pop() {
                None => return None,
                Some(x) => x,
            };

            let node = match frag {
                // if value has been inserted to the stack, the locator has already been called
                // on it and returned `Accept`.
                Fragment::Value(val) => {
                    self.left = self.left + summary;
                    return Some(val); 
                },
                Fragment::Node(node) => {
                    node
                }
            };

            node.access();
            let value = &mut node.node_value;
            let right_node = &mut node.right;
            let left_node = &mut node.left;

            let value_summary = D::to_summary(value);
            let near_left_summary : D::Summary = self.left + left_node.subtree_summary();
            let near_right_summary : D::Summary = right_node.subtree_summary() + summary;

            let dir = self.locator.locate(near_left_summary, value, near_right_summary);
            match dir {
                Err(_) => panic!(),
                Ok(LocResult::GoLeft) => {
                    if self.stack.len() > 0 {
                        panic!("GoLeft received in the middle of a segment");
                    }
                    self.push(left_node, value_summary + near_right_summary);
                },
                Ok(LocResult::GoRight) => {
                    self.push(right_node, summary);
                    self.left = self.left + near_left_summary;
                },
                Ok(LocResult::Accept) => {
                    self.push(right_node, summary);
                    self.stack.push((Fragment::Value(value), near_right_summary));
                    self.push(left_node, value_summary + near_right_summary);
                }
            }
        }
    }
}