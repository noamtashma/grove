pub mod basic_data;

// this trait represents the data that will be stored inside the tree.
// the data can include: keys, values, indices, heights, sizes, sums maximums and minimums of subtrees, actions to be performed on the subtrees,
// and whatever your heart desires for your data structure needs.
pub trait Data {
	// rebuild the associated data from the previous data and the sons.
	fn rebuild_data<'a>(&'a mut self, left : Option<&'a Self>, right : Option<&'a Self>);
	// clear the current actions in order for the user to access the node safely
	fn access<'a>(&'a mut self, left : Option<&'a mut Self>, right : Option<&'a mut Self>);
}

// TODO - trait RevData
// need to consider the design