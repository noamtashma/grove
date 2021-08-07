//! A module for examples of possible instantiations for [`Data::Value`], [`Data::Summary`],
//! [`Data::Action`] and [`Data`] itself.
//!
//! Hopefully also some useful common ones.
//!
//! For example, [`Unit`] for instantiations without  actions or without summaries.

use super::*;

/// A trait for summary instances which keep track of the size of segments.
pub trait SizedSummary {
    /// The size of the segment
    fn size(self) -> usize;
}

// TODO: consider retiring this and just requiring Value: Ord instead.

/// A trait for values that are keyed by a key type `Key`. When using keyed values, we assume
/// that all of the elements in the tree are in sorted order.
///
/// For example, when storing integers in sorted order, use the `Ordered`
/// struct, and now you can use binary search to find specific elements /
/// specify the edges of the segments you want to act upon.
///
/// Smaller values go on the left.
pub trait Keyed<Key>
where
    Key: std::cmp::Ord
{
    // TODO: is it possible to switch to `impl Borrow<Self::Key> + '_` or something similar?
    /// Gets the key associated with a value
    fn get_key(&self) -> &Key;
}

impl<T: Ord> Keyed<T> for T {
    fn get_key(&self) -> &Self {
        &self
    }
}

// Some common instantiations and examples

/// [`Data`] instance for just plain values.
pub type PlainData<V> = (V, Unit, Unit);

/// [`Data`] instance for plain values with segment size information, so that they can be accessed.
pub type SizeData<V> = (V, Size, Unit);

/// A Data marker for a standard set of summaries and actions used for numbers. Specifically,
/// one can reverse or add a constant to a whole segment at once, and one can query
/// the maximum, minimum, size and sum of a whole segment at once.
pub type StdNum = (I, NumSummary, RevAffineAction);

// ----------------- particular summaries and actions -------------------
// from here, each struct is packaged into its own internal module.
// mostly in order to reduce clutter / separate the different structs.

pub use unit::*;
mod unit {
    pub use super::*;
    /// Summary or Action placeholder when no action or no summary is needed.
    #[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Default, PartialOrd, Ord)]
    pub struct Unit {}

    impl<V> Acts<V> for Unit {
        fn act_inplace(&self, _ref: &mut V) {}
    }

    impl Add for Unit {
        type Output = Unit;
        fn add(self, _b: Unit) -> Unit {
            Unit {}
        }
    }

    impl Action for Unit {
        fn is_identity(self) -> bool {
            self == Default::default()
        }
    }

    impl<V> ToSummary<Unit> for V {
        fn to_summary(&self) -> Unit {
            Unit {}
        }
    }
}

pub use size::*;
mod size {
    use super::*;
    /// Storing the size of a subtree.
    #[derive(PartialEq, Eq, Clone, Copy, Debug)]
    pub struct Size {
        /// The size of a subtree
        pub size: usize,
    }

    impl Add for Size {
        type Output = Size;
        fn add(self, b: Size) -> Size {
            Size {
                size: self.size + b.size,
            }
        }
    }

    impl Default for Size {
        fn default() -> Size {
            Size { size: 0 }
        }
    }

    impl SizedSummary for Size {
        fn size(self) -> usize {
            self.size
        }
    }

    impl<V> ToSummary<Size> for V {
        fn to_summary(&self) -> Size {
            Size { size: 1 }
        }
    }
}

pub use rev_action::*;
mod rev_action {
    use super::*;
    /// Actions that either reverses a segment or keeps it as it is
    #[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
    pub struct RevAction {
        /// Whether to reverse the segment
        pub to_reverse: bool,
    }

    impl std::ops::Add for RevAction {
        type Output = RevAction;
        fn add(self, b: RevAction) -> RevAction {
            RevAction {
                to_reverse: self.to_reverse != b.to_reverse,
            }
        }
    }

    impl Default for RevAction {
        fn default() -> Self {
            RevAction { to_reverse: false }
        }
    }

    impl Action for RevAction {
        fn is_identity(self) -> bool {
            self == Default::default()
        }

        fn to_reverse(self) -> bool {
            self.to_reverse
        }
    }

    impl Acts<Unit> for RevAction {
        fn act_inplace(&self, _val: &mut Unit) {}
    }

    impl Acts<I> for RevAction {
        fn act_inplace(&self, _val: &mut I) {}
    }

    impl Acts<Size> for RevAction {
        fn act_inplace(&self, _val: &mut Size) {}
    }

    impl Acts<NumSummary> for RevAction {
        fn act_inplace(&self, _val: &mut NumSummary) {}
    }
}

pub use add_action::*;
mod add_action {
   use super::*; 
    /// An action for adding a constant to all values in a segment.
    #[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
    pub struct AddAction {
        /// The amount to be added
        pub add: I,
    }

    impl std::ops::Add for AddAction {
        type Output = Self;
        fn add(self, other: Self) -> Self {
            AddAction {
                add: self.add + other.add,
            }
        }
    }

    impl Default for AddAction {
        fn default() -> Self {
            AddAction { add: 0 }
        }
    }

    impl Action for AddAction {
        fn is_identity(self) -> bool {
            self == Default::default()
        }
    }

    impl Acts<Unit> for AddAction {
        fn act_inplace(&self, _val: &mut Unit) {}
    }

    impl Acts<I> for AddAction {
        fn act_inplace(&self, val: &mut I) {
            *val += self.add;
        }
    }

    impl Acts<NumSummary> for AddAction {
        fn act_inplace(&self, summary: &mut NumSummary) {
            summary.max = summary.max.map(|max: I| max + self.add);
            summary.min = summary.min.map(|max: I| max + self.add);
            summary.sum += self.add * summary.size;
        }
    }
}

type I = i32;

pub use num_summary::*;
mod num_summary {
    use super::*;
    /// A standard numerical summary
    #[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
    pub struct NumSummary {
        /// The maximum of all values in the segment. [`None`] is the segment is empty.
        pub max: Option<I>,
        /// The minimum of all values in the segment. [`None`] is the segment is empty.
        pub min: Option<I>,
        /// The size of the segment.
        pub size: I,
        /// The sum of all values in the segment.
        pub sum: I,
    }

    impl Add for NumSummary {
        type Output = Self;
        fn add(self, other: Self) -> Self {
            NumSummary {
                max: match (self.max, other.max) {
                    (Some(a), Some(b)) => Some(std::cmp::max(a, b)),
                    (Some(a), _) => Some(a),
                    (_, b) => b,
                },
                min: match (self.min, other.min) {
                    (Some(a), Some(b)) => Some(std::cmp::min(a, b)),
                    (Some(a), _) => Some(a),
                    (_, b) => b,
                },
                size: self.size + other.size,
                sum: self.sum + other.sum,
            }
        }
    }

    impl Default for NumSummary {
        fn default() -> NumSummary {
            NumSummary {
                max: None,
                min: None,
                size: 0,
                sum: 0,
            }
        }
    }

    impl SizedSummary for NumSummary {
        fn size(self) -> usize {
            self.size as usize
        }
    }

    impl ToSummary<NumSummary> for I {
        fn to_summary(&self) -> NumSummary {
            NumSummary {
                max: Some(*self),
                min: Some(*self),
                size: 1,
                sum: *self,
            }
        }
    }
}

pub use rev_add_action::*;
mod rev_add_action {
    use super::*;
    /// Actions of reversals and adding a constant
    #[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
    pub struct RevAddAction {
        /// whether to reverse the segment.
        pub to_reverse: RevAction,
        /// A constant to add to all the values in the segment.
        pub add: AddAction,
    }

    impl Add for RevAddAction {
        type Output = Self;
        fn add(self, other: Self) -> Self {
            RevAddAction {
                to_reverse: self.to_reverse + other.to_reverse,
                add: self.add + other.add,
            }
        }
    }

    impl Default for RevAddAction {
        fn default() -> Self {
            RevAddAction {
                to_reverse: RevAction::default(),
                add: AddAction::default(),
            }
        }
    }

    impl Action for RevAddAction {
        fn is_identity(self) -> bool {
            self == Default::default()
        }

        fn to_reverse(self) -> bool {
            self.to_reverse.to_reverse()
        }
    }

    impl<T> Acts<T> for RevAddAction
    where
        RevAction: Acts<T>,
        AddAction: Acts<T>,
    {
        fn act_inplace(&self, val: &mut T) {
            self.to_reverse.act_inplace(val);
            self.add.act_inplace(val);
        }
    }
}

pub use rev_affine_action::*;
mod rev_affine_action {
    use super::*;
    /// Actions of reversals, adding a constant, and multiplying by a constant.
    #[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
    pub struct RevAffineAction {
        /// Whether to reverse the segment.
        pub to_reverse: bool,
        /// A constant to multiply all the values in the segment with.
        pub mul: I,
        /// A constant to add to all the values in the segment.
        pub add: I,
    }

    impl Action for RevAffineAction {
        fn is_identity(self) -> bool {
            self == Default::default()
        }

        fn to_reverse(self) -> bool {
            self.to_reverse
        }
    }

    impl Add for RevAffineAction {
        type Output = Self;
        fn add(self, other: Self) -> Self {
            RevAffineAction {
                to_reverse: self.to_reverse ^ other.to_reverse,
                mul: self.mul * other.mul,
                add: self.add + self.mul * other.add,
            }
        }
    }

    impl Default for RevAffineAction {
        fn default() -> Self {
            RevAffineAction {
                to_reverse: false,
                mul: 1,
                add: 0,
            }
        }
    }

    impl Acts<I> for RevAffineAction {
        fn act_inplace(&self, val: &mut I) {
            *val *= self.mul;
            *val += self.add;
        }
    }

    impl Acts<NumSummary> for RevAffineAction {
        fn act_inplace(&self, summary: &mut NumSummary) {
            if self.mul < 0 {
                std::mem::swap(&mut summary.min, &mut summary.max);
            }
            summary.max = summary.max.map(|max: I| max * self.mul);
            summary.min = summary.min.map(|max: I| max * self.mul);
            summary.sum *= self.mul;

            summary.max = summary.max.map(|max: I| max + self.add);
            summary.min = summary.min.map(|max: I| max + self.add);
            summary.sum += self.add * summary.size;
        }
    }
}

pub use poly_num::*;
mod poly_num {
    use super::*;
    /// A summary type that can sum up applications of polynomials.
    /// That is, for a segment that has value a_i, is can compute
    /// (for example) \sum_i a_i*P(i) for any polynomial P of degree at most D-1.
    ///
    /// Not the most efficient it could be - meant for small values of D.
    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
    pub struct PolyNum<const D: usize> {
        // Contains the amount of elements in this segment
        size: usize,
        // Contains the moments of the segment.
        // The k'th moment is the sum of i^k*a_i for i in 0..size.
        // In other words, it's the same as `self.apply_poly(x^k)`.
        moments: [I; D],
    }

    impl<const D: usize> PolyNum<D> {
        /// Sums up the polynomial on these values, starting with index 0.
        /// That is, if this represents a segment with values `a_0, ..., a_k`,
        /// this returns `P(0)*a_0 + P(1)*a_1 + ... P(k)a_k`.
        pub fn apply_poly(&self, poly: &[I; D]) -> I {
            let mut result = 0;
            for i in 0..D {
                result += poly[i] * self.moments[i];
            }
            result
        }

        /// Shifts this segment right `shift` units to be off-balance, i.e, not start at 0.
        /// TODO: better explanation.
        pub fn shift(&self, shift: I) -> Self {
            let mut moments: [I; D] = [0; D];

            let mut powers = [0; D];
            powers[0] = 1;

            for deg in 0..D {
                // sum up (x+self.size)^deg on `rhs` and add to the result
                let mut sum: i64 = 0;
                // there can be some cancellation here, so we sum up using a bigger type.
                for i in 0..=deg {
                    sum += (powers[i] as i64) * (self.moments[i] as i64);
                }
                moments[deg] = sum as I;

                if deg >= D - 1 {
                    break; // skip multiplying the polynomial by (x+shift) one too many times
                }

                // multiply `powers` by (x+shift)
                for i in (0..=deg).rev() {
                    powers[i + 1] += powers[i];
                    powers[i] *= shift;
                }
            }

            PolyNum {
                size: self.size,
                moments,
            }
        }
    }

    impl<const D: usize> Default for PolyNum<D> {
        fn default() -> Self {
            PolyNum {
                size: 0,
                moments: [0; D],
            }
        }
    }

    impl<const D: usize> SizedSummary for PolyNum<D> {
        fn size(self) -> usize {
            self.size
        }
    }

    impl<const D: usize> Add for PolyNum<D> {
        type Output = PolyNum<D>;

        fn add(self, rhs: Self) -> Self::Output {
            let mut moments = [0; D];
            for (i, m) in rhs.shift(self.size as I).moments.iter().enumerate() {
                moments[i] = self.moments[i] + m;
            }

            PolyNum {
                size: self.size + rhs.size,
                moments,
            }
        }
    }

    impl<const D: usize> ToSummary<PolyNum<D>> for I {
        fn to_summary(&self) -> PolyNum<D> {
            let mut moments = [0; D];
            if D > 0 {
                moments[0] = *self;
            }
            PolyNum { size: 1, moments }
        }
    }

    impl<const D: usize> Acts<PolyNum<D>> for AddAction {
        fn act_inplace(&self, summary: &mut PolyNum<D>) {
            // inefficient power-and-add method for computing
            // consecutive power-sums.
            let singleton: PolyNum<D> = self.add.to_summary();
            let mut power_sums_summary = PolyNum::default();
            for j in (0..usize::BITS).rev() {
                power_sums_summary = power_sums_summary + power_sums_summary;
                if (summary.size() >> j) & 1 == 1 {
                    power_sums_summary = power_sums_summary + singleton;
                }
            }

            // add the correct power-sums to `summary`'s own sums.
            for i in 0..D {
                summary.moments[i] += power_sums_summary.moments[i];
            }
        }
    }

    impl<const D: usize> Acts<PolyNum<D>> for RevAction {
        fn act_inplace(&self, summary: &mut PolyNum<D>) {
            if !self.to_reverse {
                return;
            }

            summary.moments = summary.shift(1 - (summary.size as I)).moments;

            for i in 0..D {
                if i % 2 == 1 {
                    summary.moments[i] *= -1;
                }
            }
        }
    }

    impl<const D: usize> Acts<PolyNum<D>> for RevAffineAction {
        fn act_inplace(&self, summary: &mut PolyNum<D>) {
            // first: mul action
            for i in 0..D {
                summary.moments[i] *= self.mul;
            }
            let action = RevAddAction {
                to_reverse: RevAction {
                    to_reverse: self.to_reverse,
                },
                add: AddAction { add: self.add },
            };
            action.act_inplace(summary);
        }
    }
}
