use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;

struct Telescope<'a, T> {
	vec : Vec<*mut T>,
	phantom : PhantomData<&'a mut T>,
}

const no_value_error : &'static str = "invariant violated: telescope can't be empty";
const null_ptr_error : &'static str = "error! somehow got null pointer";

impl<'a, T> Telescope<'a, T> {
	fn new(r : &'a mut T) -> Self {
		Telescope{ vec: vec![r as *mut T], phantom : PhantomData}
	}
	
	fn reborrow<F : FnMut(&'a mut T) -> &'a mut T>(&mut self, func : &mut F) {
		let len = self.vec.len();
		if len == 0 {
			panic!(no_value_error);
		}
		unsafe {
			let r = self.vec[len-1].as_mut().expect(no_value_error);
			self.vec.push(func(r) as *mut T);
		}
	}
	
	fn push(&mut self, r : &'a mut T) {
		self.vec.push(r as *mut T);
	}
	
	// lets the user use the last reference for some time, and discards it completely.
	// after the user uses it, the next time they inspect the telescope, it won't be there.
	fn pop(&mut self) -> Option<&mut T> {
		let len = self.vec.len();
		if len == 0 {
			panic!(no_value_error);
		}
		if len == 1 {
			return None;
		}
		else {
			unsafe {
				let r : *mut T = self.vec.pop().expect(no_value_error);
				return Some(r.as_mut().expect(null_ptr_error));
			}
		}
	}
}

impl<'a, T> Deref for Telescope<'a, T> {
	type Target = T;
	fn deref(&self) -> &T {
		let len = self.vec.len();
		if len == 0 {
			panic!(no_value_error);
		}
		unsafe {
			return self.vec[len-1].as_ref().expect(null_ptr_error);
		}
	}
}

impl<'a, T> DerefMut for Telescope<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		let len = self.vec.len();
		if len == 0 {
			panic!(no_value_error);
		}
		unsafe {
			return self.vec[len-1].as_mut().expect(null_ptr_error);
		}
	}
}