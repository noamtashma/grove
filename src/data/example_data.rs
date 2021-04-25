use super::*;
use std::marker::PhantomData;

/// Used for cases where no action or no summary is needed.
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Default, PartialOrd, Ord)]
pub struct Unit {}

impl Add for Unit {
	type Output = Unit;
	fn add(self, _b : Unit) -> Unit {
		Unit {}
	}
}

/// Storing the size of a subtree.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Size {
    pub size : usize
}

impl Add for Size {
    type Output = Size;
    fn add(self, b : Size) -> Size {
        Size {size : self.size + b.size}
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
struct SizeData<V> {phantom : PhantomData<V>}

impl<V> Data for SizeData<V> {
    type Action = Unit;
    type Summary = Size;
    type Value = V;
    const IDENTITY : Self::Action = Unit{};
    const EMPTY : Size = Size {size : 0};
    fn act(_ : Unit, b : Size) -> Size { b }
    fn to_summary(_val : &Self::Value) -> Self::Summary {
        Size {size : 1}
    }
}

/// actions in which action::Value keeps track of the size of subtrees.
pub trait SizedData : Data {
    /// The size of the subtree of the current node
    fn size(val : Self::Summary) -> usize;
}

impl<V : Eq + Copy> SizedData for SizeData<V> {
    fn size(val : Size) -> usize { val.size }
}




/// A Data marker for no data at all, except for straight values.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct NoAction<V> {
    phantom : PhantomData<V>,
}

impl<V : Eq + Copy> Data for NoAction<V> {
    type Summary = Unit;
    type Action = Unit;
    type Value = V;
    const IDENTITY : Self::Action = Unit{};

    const EMPTY : Unit = Unit{};

    fn to_summary(_val : &Self::Value) -> Self::Summary {
        Unit{}
    }
}


/// Actions that either reverse a segment or keep it as it is
#[derive(PartialEq, Eq, Clone, Copy)]
struct RevAction {
    to_reverse : bool,
}

impl std::ops::Add for RevAction {
    type Output = RevAction;
    fn add(self, b : RevAction) -> RevAction {
        RevAction {to_reverse : self.to_reverse != b.to_reverse}
    }
}


type I = i32;
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub struct NumSummary {
    pub max : Option<I>,
    pub min : Option<I>,
    pub size : I,
    pub sum : I,
}
impl Add for NumSummary {
    type Output = Self;
    fn add(self, other : Self) -> Self {
        NumSummary {
            max : match (self.max, other.max) {
                    (Some(a), Some(b)) => Some(std::cmp::max(a,b)),
                    (Some(a), _) => Some(a),
                    (_, b) => b,
                },
            min : match (self.min, other.min) {
                (Some(a), Some(b)) => Some(std::cmp::min(a,b)),
                (Some(a), _) => Some(a),
                (_, b) => b,
            },
            size : self.size + other.size,
            sum : self.sum + other.sum,
        }
    }
}

/// Actions of reversals and adding a constant
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct RevAddAction{
    pub to_reverse : bool,
    pub add : I,
}

impl Add for RevAddAction {
    type Output = Self;
    fn add(self, other : Self) -> Self {
        RevAddAction {
            to_reverse : self.to_reverse != other.to_reverse,
            add : self.add + other.add,
        }
    }
}

/// A Data marker for a standard set of summaries and actions used for numbers. Specifically,
/// one can reverse or add a constant to a whole segment at once, and one can query
/// the maximum, minimum, size and sum of a whole segment at once.
pub struct StdNum{}

impl Data for StdNum {
    type Value = I;
    type Summary = NumSummary;
    type Action = RevAddAction;

    const IDENTITY : Self::Action = RevAddAction { to_reverse : false, add : 0 };
    const EMPTY : Self::Summary = NumSummary {
        max : None,
        min : None,
        size : 0,
        sum : 0,
    };

    fn to_reverse(act : Self::Action) -> bool {
        act.to_reverse
    }

    fn to_summary(val : &I) -> Self::Summary {
        NumSummary {
            max : Some(*val),
            min : Some(*val),
            size : 1,
            sum : *val,
        }
    }

    fn act(action : Self::Action, summary : Self::Summary) -> Self::Summary {
        Self::Summary {
            max : summary.max.map(|max : I| { max + action.add }),
            min : summary.min.map(|min : I| { min + action.add }),
            size : summary.size,
            sum : summary.sum + action.add*summary.size,
        }
    }

    fn act_value(action : Self::Action, val : &mut I) {
        *val += action.add;
    }
}

impl SizedData for StdNum {
    fn size(summary : Self::Summary) -> usize {
        summary.size as usize
    }
}

impl Reverse for StdNum {
    fn internal_reverse(node : &mut crate::trees::basic_tree::BasicNode<Self>) {
        node.act(RevAddAction {to_reverse : true, add : 0});
    }
}



// TODO: consider retiring this and just requiring Value : Ord instead.
/// The convention is that smaller values go on the left
pub trait Keyed {
    type Key : std::cmp::Ord;
    fn get_key(&self) -> <Self as Keyed>::Key;
}