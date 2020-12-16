pub mod example_data;

// this trait represents the data that will be stored inside the tree.
// the data can include: keys, values, indices, heights, sizes, sums maximums and minimums of subtrees, actions to be performed on the subtrees,
// and whatever your heart desires for your data structure needs.
pub trait Data {
	// rebuild the associated data from the previous data and the sons.
	fn rebuild_data<'a>(&'a mut self, left : Option<&'a Self>, right : Option<&'a Self>);
	// clear the current actions in order for the user to access the node safely
	fn access<'a>(&'a mut self, left : Option<&'a mut Self>, right : Option<&'a mut Self>);

	// these two functions should be implemented if you want to be able to reverse suvtrees of your tree.
	// this function should return whether you would like to reverse your subtree
	// and zero it back - calling to_reverse() twice should always return false the second time.

	// it doesn't matter in which function the actual effect of the reverse happens,
	// however, you can only pick one
	fn to_reverse(&mut self) -> bool {
		false
	}

	// this function should flip the bit of whether you'll want to reverse your data
	fn reverse(&mut self) {
		panic!("this struct doesn't implement reversal")
	}
}

// marker trait for Data that implement reverse
pub trait Reverse : Data {}

// TODO - trait RevData
// need to consider the design