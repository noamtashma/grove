use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;

pub struct Telescope<'a, T> {
	vec : Vec<*mut T>,
	phantom : PhantomData<&'a mut T>,
}

const NO_VALUE_ERROR : &str = "invariant violated: telescope can't be empty";
const NULL_POINTER_ERROR : &str = "error! somehow got null pointer";

impl<'a, T> Telescope<'a, T> {
	pub fn new(r : &'a mut T) -> Self {
		Telescope{ vec: vec![r as *mut T], phantom : PhantomData}
	}

	pub fn size(&self) -> usize {
		self.vec.len()
	}
	
	// if the type was just
	// (&mut self, func : &mut dyn FnMut(&'a mut T) -> &'a mut T)
	// then this function would be unsafe
	// because on the call, we could leak the reference outside,
	// and then immediately pop the telescope to get another reference to it.

	// however, this type ensures no leaking is possible, since the function that leaks
	// can't guarantee that the reference given to it will live for any length of time.
	pub fn extend<F : for<'b> FnMut(&'b mut T) -> &'b mut T>(&mut self, func : &mut F) {
		match self.extend_result::<(), _>(&mut |r| Ok(func(r))) {
			Ok(()) => (),
			Err(()) => panic!("unexpected error in error-less reborrow"),
		}
	}

	pub fn extend_result
		<E,
		F : for<'b> FnMut(&'b mut T) -> Result<&'b mut T, E>>
			(&mut self, func : &mut F) -> Result<(),E> {

		let len = self.vec.len();
		if len == 0 {
			panic!(NO_VALUE_ERROR);
		}
		unsafe {
			let r = self.vec[len-1].as_mut().expect(NO_VALUE_ERROR);
			match func(r) {
				Ok(p) => self.vec.push(p as *mut T),
				Err(e) => return Err(e),
			}
		}
		return Ok(());
	}
	
	pub fn push(&mut self, r : &'a mut T) {
		self.vec.push(r as *mut T);
	}
	
	// lets the user use the last reference for some time, and discards it completely.
	// after the user uses it, the next time they inspect the telescope, it won't be there.
	pub fn pop(&mut self) -> Option<&mut T> {
		let len = self.vec.len();
		if len == 0 {
			panic!(NO_VALUE_ERROR);
		};
		if len == 1 {
			return None;
		}
		else {
			unsafe {
				let r : *mut T = self.vec.pop().expect(NO_VALUE_ERROR);
				return Some(r.as_mut().expect(NULL_POINTER_ERROR));
			}
		}
	}

	// discards the telescope and returns the last reference
	// the difference between this and using pop() are:
	// this will consume the telescope
	// you can use this to remove the original reference
	pub fn into_ref(mut self) -> &'a mut T {
		let len = self.vec.len();
		if len == 0 {
			panic!(NO_VALUE_ERROR);
		};
		unsafe {
			let r : *mut T = self.vec.pop().expect(NO_VALUE_ERROR);
			return r.as_mut().expect(NULL_POINTER_ERROR);
		}
	}
}

impl<'a, T> Deref for Telescope<'a, T> {
	type Target = T;
	fn deref(&self) -> &T {
		let len = self.vec.len();
		if len == 0 {
			panic!(NO_VALUE_ERROR);
		}
		unsafe {
			return self.vec[len-1].as_ref().expect(NULL_POINTER_ERROR);
		}
	}
}

impl<'a, T> DerefMut for Telescope<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		let len = self.vec.len();
		if len == 0 {
			panic!(NO_VALUE_ERROR);
		}
		unsafe {
			return self.vec[len-1].as_mut().expect(NULL_POINTER_ERROR);
		}
	}
}