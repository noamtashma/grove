use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;

pub struct Telescope<'a, T : ?Sized> {
	vec : Vec<*mut T>,
	phantom : PhantomData<&'a mut T>,
}

// these aren't ever supposed to happen. but since we touch unsafe code, we might as well
// have clear error message when we `expect()`
pub const NO_VALUE_ERROR : &str = "invariant violated: telescope can't be empty";
pub const NULL_POINTER_ERROR : &str = "error! somehow got null pointer";

impl<'a, T : ?Sized> Telescope<'a, T> {
	pub fn new(r : &'a mut T) -> Self {
		Telescope{ vec: vec![r as *mut T], phantom : PhantomData}
	}

	pub fn size(&self) -> usize {
		self.vec.len()
	}
	
	/// This function extends the telescope one time. That menas, if the latest layer
	/// of the telescope is a reference `ref`, then this call extends the telescope
	/// and the new latest layer will have the reference `ref' = func(ref)`.
	/// After this call, the telescope will expose the new `ref'`, and `ref`
	/// will be frozen (As it is borrowed by `ref'`), until this layer is
	/// popped off.
	/// 
	/// # Safety:
	/// If the type was just
	/// `fn extend<'a, F : FnOnce(&'a mut T) -> &'a mut T>(&'b mut self, func : F) -> ()`
	/// then this function would be unsafe,
	/// because `func` could leak the reference outside, and then the caller could immediately
	/// pop the telescope to get another copy of the same reference.
	///
	/// We could use
	/// `fn extend<'a, F : FnOnce(&'a mut T) -> &'a mut T>(&'a mut self, func : F) -> ()`
	///
	/// But that would invalidate the whole point of using the telescope - You couldn't extend
	/// more than once, and it eouldn't be any better than a regular mutable reference.
	///
	/// However, the actual type ensures no leaking is possible, since the function that leaks
	/// can't guarantee that the reference given to it will live for any length of time.
	pub fn extend<F : for<'b> FnOnce(&'b mut T) -> &'b mut T>(&mut self, func : F) {
		match self.extend_result::<(), _>(|r| Ok(func(r))) {
			Ok(()) => (),
			Err(()) => panic!("unexpected error in error-less reborrow"),
		}
	}

	pub fn extend_result
		<E,
		F : for<'b> FnOnce(&'b mut T) -> Result<&'b mut T, E>>
			(&mut self, func : F) -> Result<(),E> {

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
	// you can use this to take the original reference. therefore, there is no None case.
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

impl<'a, T : ?Sized> Deref for Telescope<'a, T> {
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

impl<'a, T : ?Sized> DerefMut for Telescope<'a, T> {
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

impl<'a, T : ?Sized> From<&'a mut T> for Telescope<'a, T> {
	fn from(r : &'a mut T) -> Self {
		Self::new(r)
	}
}