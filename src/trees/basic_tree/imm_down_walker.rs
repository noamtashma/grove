
use crate::*;
use trees::basic_tree::BasicTree;

const suddenly_empty : &'static str = "The locator unexpectedly became empty";

/// A BasicWalker version that is immutable, and can only go down.
#[derive(Copy)]
struct ImmDownBasicWalker<'a, D: Data, T=()> {
    tree: &'a basic_tree::BasicTree<D, T>,

    // to be applied to everything in `tree`.
    // already contains this node's actions.
    current_action: D::Action,

    // note: these should always have `current_action` already applied to them,
    // and in the already reversed order if `current_action.to_reverse() == true`.
    // the summary of everything to the left of the current subtree
    far_left_summary: D::Summary,
    // the summary of everything to the right of the current subtree
    far_right_summary: D::Summary,
}

/// This is needed because the automatic implementation also requires
/// `D: Clone` and `T: Clone`.
impl<'a, D: Data, T> Clone for ImmDownBasicWalker<'a, D, T> {
    fn clone(&self) -> Self {
        ImmDownBasicWalker {
            ..*self
        }
    }
}

impl<'a, D: Data, T> ImmDownBasicWalker<'a, D, T> {
    pub fn new(tree: &'a basic_tree::BasicTree<D, T>) -> Self {
        ImmDownBasicWalker {
            tree,
            current_action: tree.action(),
            far_left_summary: Default::default(),
            far_right_summary: Default::default(),
        }
    }

    /// Goes to the left son.
    /// If at an empty position, returns [`None`].
    pub fn go_left(&mut self) -> Option<()> {
        self.go_left_extra()?;
        Some(())
    }

    /// Returns the two sons of this tree _in correct order_.
    /// (left_son, right_son).
    pub fn sons(&self) -> Option<(&BasicTree<D, T>, &BasicTree<D, T>)> {
        let node = self.tree.node()?;
        let mut left = &node.left;
        let mut right = &node.right;
        if self.current_action.to_reverse() {
            std::mem::swap(&mut left, &mut right);
        }
        Some((left, right))
    }

    /// Goes to the left son.
    /// If at an empty position, returns [`None`].
    /// Otherwise, also returns the summary of the current node
    /// with its right subtree.
    pub fn go_left_extra(&mut self) -> Option<D::Summary> {
        let node = self.tree.node()?;

        // deal with reversals
        let mut right = &node.right;
        let mut left = &node.left;
        if self.current_action.to_reverse() {
            std::mem::swap(&mut left, &mut right);
        }

        let extra = self.current_action.act(
            D::to_summary(&node.node_value)
            + right.subtree_summary()
        );
        self.far_right_summary = extra + self.far_right_summary;
        self.tree = left;
        self.current_action = self.current_action + left.action();
        Some(extra)
    }

    /// Goes to the right son.
    /// If at an empty position, returns [`None`].
    pub fn go_right(&mut self) -> Option<()> {
        self.go_right_extra()?;
        Some(())
    }

    /// Goes to the right son.
    /// If at an empty position, returns [`None`].
    /// Otherwise, also returns the summary of the current node
    /// with its left subtree.
    pub fn go_right_extra(&mut self) -> Option<D::Summary> {
        let node = self.tree.node()?;

        // deal with reversals
        let mut right = &node.right;
        let mut left = &node.left;
        if self.current_action.to_reverse() {
            std::mem::swap(&mut left, &mut right);
        }

        let extra = self.current_action.act(
            left.subtree_summary()
            + D::to_summary(&node.node_value)
        );
        self.far_left_summary = self.far_left_summary + extra;
        self.tree = right;
        self.current_action = self.current_action + right.action();
        Some(extra)
    }

    /// Returns the value at the current node.
    pub fn value(&self) -> Option<D::Value> where D::Value: Clone {
        Some(self.current_action.act(self.tree.node()?.node_value.clone()))
    }

    pub fn far_left_summary(&self) -> D::Summary {
        if let Some(node) = self.tree.node() {
            let left = if self.current_action.to_reverse() {
                &node.right
            } else {
                &node.left
            };
            self.far_left_summary + self.current_action.act(left.subtree_summary())
        } else {
            self.far_left_summary
        }
    }

    pub fn far_right_summary(&self) -> D::Summary {
        if let Some(node) = self.tree.node() {
            let right = if self.current_action.to_reverse() {
                &node.left
            } else {
                &node.right
            };
            self.current_action.act(right.subtree_summary()) + self.far_right_summary 
        } else {
            self.far_right_summary
        }
    }

    pub fn query_locator<L: Locator<D>>(&self, locator: &L) -> Option<locators::LocResult>
    where
        D::Value: Clone,
    {
        let node = self.tree.node()?;

        // deal with reversals
        let mut right = &node.right;
        let mut left = &node.left;
        if self.current_action.to_reverse() {
            std::mem::swap(&mut left, &mut right);
        }

        let direction = locator.locate(
            self.far_left_summary + self.current_action.act(left.subtree_summary()),
            &self.current_action.act(node.node_value.clone()),
            self.current_action.act(right.subtree_summary()) + self.far_right_summary
        );

        Some(direction)
    }
}









/// Returns the accumulated values on the locator's segment
/// Do not use with splay trees - it might mess up the complexity,
/// because it uses go_up().
///
/// Instead, use the specific [`SomeTree::segment_summary`]
pub fn segment_summary<D: Data, T, L>(tree: &basic_tree::BasicTree<D, T>, locator: L) -> D::Summary
where
    L: Locator<D>,
    D::Value: Clone,
{
    use trees::*;
    use locators::LocResult::*;

    let mut walker = ImmDownBasicWalker::new(tree);
    while let Some(direction) = walker.query_locator(&locator) {
        match direction {
            GoLeft => {
                walker.go_left();
            },
            GoRight => {
                walker.go_right();
            },
            Accept => {
                // TODO: ugly
                let current_node_summary = walker.current_action.act(
                    D::to_summary(&walker.tree.node().unwrap().node_value)
                );
                let mut left_walker = walker.clone();
                let mut right_walker = walker;
                left_walker.go_left().unwrap();
                right_walker.go_right().unwrap();

                let left_half = segment_summary_on_suffix(left_walker, locator.clone());
                let right_half = segment_summary_on_prefix(right_walker, locator);
                return left_half
                    + current_node_summary
                    + right_half;
            },
        }
    }
    // Empty segment case 
    Default::default()
}

/// Returns the summary of the segment in the current tree,
/// provided the segment is a suffix.
fn segment_summary_on_suffix<D:Data, T, L>(
    mut walker: ImmDownBasicWalker<D, T>,
    locator: L
) -> D::Summary
where
L: Locator<D>,
D::Value: Clone,
{
    use trees::*;
    use locators::LocResult::*;
    let mut result: D::Summary = Default::default();

    while let Some(direction) = walker.query_locator(&locator) {
        match direction {
            GoLeft => panic!("Inconsistent locator"),
            GoRight => {
                walker.go_right().expect(suddenly_empty)
            },
            Accept => {
                let extra = walker.go_left_extra().expect(suddenly_empty);
                result = extra + result;
            },
        }
    }
    result
}

/// Returns the summary of the segment in the current tree,
/// provided the segment is a suffix.
fn segment_summary_on_prefix<D:Data, T, L>(
    mut walker: ImmDownBasicWalker<D, T>,
    locator: L
) -> D::Summary
where
L: Locator<D>,
D::Value: Clone,
{
    use trees::*;
    use locators::LocResult::*;
    let mut result: D::Summary = Default::default();

    while let Some(direction) = walker.query_locator(&locator) {
        match direction {
            GoLeft => {
                walker.go_left().expect(suddenly_empty)
            },
            GoRight => panic!("Inconsistent locator"),
            Accept => {
                let extra = walker.go_right_extra().expect(suddenly_empty);
                result = result + extra;
            },
        }
    }
    result
}


/*

/// Returns the accumulated values on the locator's segment
/// Do not use with splay trees - it might mess up the complexity,
/// because it uses go_up().
///
/// Instead, use the specific [`SomeTree::segment_summary`]
pub fn segment_summary<D: Data, T, L>(mut tree: &basic_tree::BasicTree<D, T>, locator: L) -> D::Summary
where
    L: Locator<D>,
    D::Value: Clone,
{
    use trees::*;
    use LocResult::*;

    let mut left_summary: D::Summary = Default::default();
    let mut right_summary: D::Summary = Default::default();
    let mut current_action: D::Action = Default::default();

    while let Some(node) = tree.node() {
        current_action = current_action + *node.action();

        let direction = clone_locate(
            current_action,
            left_summary,
            &current_action.act(node.node_value.clone()),
            right_summary,
            &locator
        );
    
        match direction {
            GoRight => {
                left_summary = left_summary +
                    current_action.act(node.left.subtree_summary()
                        + D::to_summary(&node.node_value)
                    );
                tree = &node.right
            },
            GoLeft => {
                right_summary =
                    current_action.act(
                        D::to_summary(&node.node_value)
                        + node.right.subtree_summary()
                    )
                    + right_summary;
                tree = &node.left
            },

            // at this point, we split into the two sides
            Accept => {
                let left_node_summary = current_action.act(node.left.subtree_summary());
                let node_summary = current_action.act(D::to_summary(&node.node_value));
                let right_node_summary = current_action.act(node.right.subtree_summary());
                
                let first_half = segment_summary_on_suffix(
                    &node.left, 
                    current_action, 
                    left_summary, 
                    node_summary + right_node_summary + right_summary, 
                    locator.clone()
                );
                let second_half = segment_summary_on_prefix(
                    &node.right, 
                    current_action, 
                    left_summary + left_node_summary + node_summary, 
                    right_summary, locator
                );

                return first_half + node_summary + second_half;
            }
        }
    }

    // empty locator case
    Default::default()
}

fn segment_summary_on_suffix<D:Data, T, L>(
        mut tree: &basic_tree::BasicTree<D, T>, 
        mut current_action: D::Action, 
        mut left_summary: D::Summary,
        right_summary: D::Summary,
        locator: L
    ) -> D::Summary
where
    L: Locator<D>,
    D::Value: Clone,
{
    use trees::*;
    use LocResult::*;

    let mut current_summary: D::Summary = Default::default();

    while let Some(node) = tree.node() {
        current_action = current_action + *node.action();
        let direction = clone_locate(
            current_action,
            left_summary,
            &current_action.act(node.node_value.clone()),
            current_summary + right_summary,
            &locator
        );
    
        match direction {
            GoRight => {
                left_summary = left_summary +
                    current_action.act(node.left.subtree_summary()
                        + D::to_summary(&node.node_value)
                    );
                tree = &node.right
            }
            GoLeft => panic!("inconsistent locator"),

            Accept => {
                current_summary =
                    current_action.act(
                        D::to_summary(&node.node_value)
                        + node.right.subtree_summary())
                    + current_summary;
                
                tree = &node.left;
            }
        }
    }
    // when finished
    current_summary
}

fn segment_summary_on_prefix<D:Data, T, L>(
        mut tree: &basic_tree::BasicTree<D, T>, 
        mut current_action: D::Action, 
        left_summary: D::Summary,
        mut right_summary: D::Summary,
        locator: L
    ) -> D::Summary
where
    L: Locator<D>,
    D::Value: Clone,
{
    use trees::*;
    use LocResult::*;

    let mut current_summary: D::Summary = Default::default();

    while let Some(node) = tree.node() {
        current_action = current_action + *node.action();
        let direction = locator.locate(
            left_summary + current_summary,
            &current_action.act(node.node_value.clone()),
            right_summary
        );
    
        match direction {
            GoRight => panic!("inconsistent locator"),
            GoLeft => {
                right_summary = current_action.act(
                        D::to_summary(&node.node_value)
                        + node.right.subtree_summary()
                    ) + right_summary;
                tree = &node.left;
            }

            Accept => {
                current_summary = current_summary
                    + current_action.act(
                        node.left.subtree_summary()
                        + D::to_summary(&node.node_value)
                    );
                
                tree = &node.right;
            }
        }
    }
    // when finished
    current_summary
}

*/