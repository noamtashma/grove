use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;
use void::ResultVoidExt;

/// A Telescope
/// This struct is used to allow recursively reborrow mutable references in a dynamic
/// but safe way.
pub struct Telescope<'a, T : ?Sized> {
	head : *mut T,
	vec : Vec<*mut T>,
	phantom : PhantomData<&'a mut T>,
}

// these aren't ever supposed to happen. but since we touch unsafe code, we might as well
// have clear error message when we `expect()`
pub const NO_VALUE_ERROR : &str = "invariant violated: telescope can't be empty";
pub const NULL_POINTER_ERROR : &str = "error! somehow got null pointer";

impl<'a, T : ?Sized> Telescope<'a, T> {
	pub fn new(r : &'a mut T) -> Self {
		Telescope{ head : r as *mut T, vec: vec![], phantom : PhantomData}
	}

	pub fn size(&self) -> usize {
		self.vec.len() + 1
	}

	// possible extention: using a special trait, create a version of `extend`
	// that accepts function of the sort
	// `for<'b> where 'a : 'b, fn(&'b mut T) -> &'b mut T`
	// which is syntactically impossible to type.

	/// This function extends the telescope one time. That meaans, if the latest layer
	/// of the telescope is a reference `ref`, then this call extends the telescope
	/// and the new latest layer will have the reference `ref2 = func(ref)`.
	/// After this call, the telescope will expose the new `ref2`, and `ref`
	/// will be frozen (As it is borrowed by `ref2`), until this layer is
	/// popped off.
	/// 
	/// # Safety:
	/// The type ensures no leaking is possible, since `func` can't guarantee that
	/// the reference given to it will live for any length of time, so it can't leak it anywhere.
	/// It can only use it inside the function, and use it to return a new reference, which is the
	/// intended usage.
	///
	/// Altenatively, if the type was just
	/// ```rust,ignore
	/// fn extend<'a, F : FnOnce(&'a mut T) -> &'a mut T>(&mut self, func : F)
	/// ```
	/// then this function would be unsafe,
	/// because `func` could leak the reference outside, and then the caller could immediately
	/// pop the telescope to get another copy of the same reference.
	///
	/// We could use
	/// ```rust,ignore
	/// fn extend<'a, F : FnOnce(&'a mut T) -> &'a mut T>(&'a mut self, func : F)
	/// ```
	///
	/// But that would invalidate the whole point of using the telescope - You couldn't extend
	/// more than once, and it couldn't be any better than a regular mutable reference.
	///
	pub fn extend<F : for<'b> FnOnce(&'b mut T) -> &'b mut T>(&mut self, func : F) {
		self.extend_result(|r| Ok(func(r))).void_unwrap()
	}

	/// Same as [`Self::extend`], but allows the function to return an error value.
	pub fn extend_result<E, F>(&mut self, func : F) -> Result<(),E> where
		F : for<'b> FnOnce(&'b mut T) -> Result<&'b mut T, E>
	{
		// The compiler has to be told explicitly that the lifetime is `'a`.
		// It probably doesn't matter much in practice, since we specifically require `func` to be able to work
		// with any lifetime, and the references are converted to pointers immediately.
		// however, that is the correct lifetime.
		let head_ref : &'a mut T = unsafe {
			self.head.as_mut()
		}.expect(NULL_POINTER_ERROR);

		match func(head_ref) {
			Ok(p) => self.push(p),
			Err(e) => return Err(e),
		}
		return Ok(());
	}
	
	/// Push another reference, unrelated to the current one.
	/// `tel.push(ref)` would be equivalent to `tel.extend(|prev| { Ok(ref) })`, but that
	/// doesn't compile since [`Self::extend`]'s type may require `'b` to be larger than `'a`.
	pub fn push(&mut self, r : &'a mut T) {
		self.vec.push(self.head);
		self.head = r as *mut T;
	}
	
	/// Lets the user use the last reference for some time, and discards it completely.
	/// After the user uses it, the next time they inspect the telescope, it won't be there.
	pub fn pop(&mut self) -> Option<&mut T> {
		let res = unsafe {
			self.head.as_mut()
		}.expect(NULL_POINTER_ERROR);
		self.head = self.vec.pop()?; // We can't pop the original reference. In that case, Return None.

		Some(res)
	}
	

	/// Discards the telescope and returns the last reference.
	/// The difference between this and using [`Self::pop`] are:
	/// * This will consume the telescope
	/// * [`Self::pop`] will never pop the first original reference. [`Self::into_ref`] will.
	pub fn into_ref(self) -> &'a mut T {
		return unsafe {
			self.head.as_mut()
		}.expect(NULL_POINTER_ERROR);
	}
}

impl<'a, T : ?Sized> Deref for Telescope<'a, T> {
	type Target = T;
	fn deref(&self) -> &T {
		unsafe {
			self.head.as_ref()
		}.expect(NULL_POINTER_ERROR)
	}
}

impl<'a, T : ?Sized> DerefMut for Telescope<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe {
			self.head.as_mut()
		}.expect(NULL_POINTER_ERROR)
	}
}

impl<'a, T : ?Sized> From<&'a mut T> for Telescope<'a, T> {
	fn from(r : &'a mut T) -> Self {
		Self::new(r)
	}
}