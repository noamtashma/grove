use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;
use void::ResultVoidExt;

// TODO: switch to `NonNull` when rust 1.53 arrives.
/// A Telescope
/// This struct is used to allow recursively reborrowing mutable references in a dynamic
/// but safe way.
pub struct Telescope<'a, T: ?Sized> {
    head: *mut T,
    vec: Vec<*mut T>,
    phantom: PhantomData<&'a mut T>,
}

// TODO: consider converting the pointers to values without checking for null values.
// it's supposed to work, since the pointers only ever come from references.

// these aren't ever supposed to happen. but since we touch unsafe code, we might as well
// have clear error message when we `expect()`
pub const NO_VALUE_ERROR: &str = "invariant violated: telescope can't be empty";
pub const NULL_POINTER_ERROR: &str = "error! somehow got null pointer";

impl<'a, T: ?Sized> Telescope<'a, T> {
    pub fn new(r: &'a mut T) -> Self {
        Telescope {
            head: r as *mut T,
            vec: vec![],
            phantom: PhantomData,
        }
    }

    pub fn size(&self) -> usize {
        self.vec.len() + 1
    }

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
    pub fn extend<F: for<'b> FnOnce(&'b mut T) -> &'b mut T>(&mut self, func: F) {
        self.extend_result(|r| Ok(func(r))).void_unwrap()
    }

    /// Same as [`Self::extend`], but allows the function to return an error value.
    pub fn extend_result<E, F>(&mut self, func: F) -> Result<(), E>
    where
        F: for<'b> FnOnce(&'b mut T) -> Result<&'b mut T, E>,
    {
        self.extend_result_precise(|r, _phantom| func(r))
    }

    /// Same as [`Self::extend`], but allows the function to return an error value,
    /// and also tells the inner function that `'a : 'b` using a phantom argument.
    pub fn extend_result_precise<E, F>(&mut self, func: F) -> Result<(), E>
    where
        F: for<'b> FnOnce(&'b mut T, PhantomData<&'b &'a ()>) -> Result<&'b mut T, E>,
    {
        // The compiler has to be told explicitly that the lifetime is `'a`.
        // Otherwise the lifetime is left unconstrained.
        // It probably doesn't matter much in practice, since we specifically require `func` to be able to work
        // with any lifetime, and the references are converted to pointers immediately.
        // However, that is the "most correct" lifetime - its actual lifetime may be anything up to `'a`,
        // depending on whether the user will pop it earlier than that.
        let head_ref: &'a mut T = unsafe { self.head.as_mut() }.expect(NULL_POINTER_ERROR);

        match func(head_ref, PhantomData) {
            Ok(p) => {
                self.push(p);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// This function maps the top of the telescope. It's similar to [`Self::extend`], but
    /// it doesn't keep the previous reference. See [`Self::extend`] for more details.
    pub fn map<F: for<'b> FnOnce(&'b mut T) -> &'b mut T>(&mut self, func: F) {
        self.map_result(|r| Ok(func(r))).void_unwrap()
    }

    /// Same as [`Self::map`], but allows the function to return an error value.
    pub fn map_result<E, F>(&mut self, func: F) -> Result<(), E>
    where
        F: for<'b> FnOnce(&'b mut T) -> Result<&'b mut T, E>,
    {
        self.map_result_precise(|r, _| func(r))
    }

    /// Same as [`Self::map`], but allows the function to return an error value,
    /// and also tells the inner function that `'a : 'b` using a phantom argument.
    pub fn map_result_precise<E, F>(&mut self, func: F) -> Result<(), E>
    where
        F: for<'b> FnOnce(&'b mut T, PhantomData<&'b &'a ()>) -> Result<&'b mut T, E>,
    {
        // The compiler has to be told explicitly that the lifetime is `'a`.
        // Otherwise the lifetime is left unconstrained.
        // It probably doesn't matter much in practice, since we specifically require `func` to be able to work
        // with any lifetime, and the references are converted to pointers immediately.
        // However, that is the "most correct" lifetime - its actual lifetime may be anything up to `'a`,
        // depending on whether the user will pop it earlier than that.
        let head_ref: &'a mut T = unsafe { self.head.as_mut() }.expect(NULL_POINTER_ERROR);

        match func(head_ref, PhantomData) {
            Ok(p) => {
                self.head = p as *mut T;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Push another reference to the telescope, unrelated to the current one.
    /// `tel.push(ref)` is morally equivalent to `tel.extend_result_precise(move |_, _| { Ok(ref) })`.
    /// However, you might have some trouble making the anonymous function conform to the
    /// right type.
    pub fn push(&mut self, r: &'a mut T) {
        self.vec.push(self.head);
        self.head = r as *mut T;

        /* alternative definition using a call to `self.extend_result_precise`.
        // in order to name 'x, replace the signature with:
        // pub fn push<'x>(&'x mut self, r : &'a mut T) {
        // this is used in order to tell the closure to conform to the right type
        fn helper<'a,'x, T : ?Sized, F> (f : F) -> F where
                F : for<'b> FnOnce(&'b mut T, PhantomData<&'b &'a ()>)
                -> Result<&'b mut T, void::Void> + 'x
            { f }

        self.extend_result_precise(
            helper::<'a,'x>(move |_, _phantom| { Ok(r) })
        ).void_unwrap();
        */
    }

    /// Lets the user use the last reference for some time, and discards it completely.
    /// After the user uses it, the next time they inspect the telescope, it won't be there.
    pub fn pop(&mut self) -> Option<&mut T> {
        let res = unsafe { self.head.as_mut() }.expect(NULL_POINTER_ERROR);
        self.head = self.vec.pop()?; // We can't pop the original reference. In that case, Return None.

        Some(res)
    }

    /// Discards the telescope and returns the last reference.
    /// The difference between this and using [`Self::pop`] are:
    /// * This will consume the telescope
    /// * [`Self::pop`] will never pop the first original reference, because that would produce an
    ///   invalid telescope. [`Self::into_ref`] will.
    pub fn into_ref(self) -> &'a mut T {
        unsafe { self.head.as_mut() }.expect(NULL_POINTER_ERROR)
    }

    /// Gets the [`std::ptr::NonNull`] pointer that is i'th from the top of the telescope.
    /// The intended usage is for using the pointers as the inputs to `ptr_eq`.
    /// However, using the pointers themselves (which requires `unsafe`) will almost definitely
    /// break rust's guarantees.
    pub fn get_nonnull(&self, i: usize) -> std::ptr::NonNull<T> {
        let ptr = if i == 0 {
            self.head
        } else {
            self.vec[self.vec.len() - i]
        };
        std::ptr::NonNull::new(ptr).expect(NULL_POINTER_ERROR)
    }
}

impl<'a, T: ?Sized> Deref for Telescope<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.head.as_ref() }.expect(NULL_POINTER_ERROR)
    }
}

impl<'a, T: ?Sized> DerefMut for Telescope<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.head.as_mut() }.expect(NULL_POINTER_ERROR)
    }
}

impl<'a, T: ?Sized> From<&'a mut T> for Telescope<'a, T> {
    fn from(r: &'a mut T) -> Self {
        Self::new(r)
    }
}
