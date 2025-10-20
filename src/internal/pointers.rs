use core::{
    mem,
    ops::{Deref, DerefMut},
    ptr::{addr_of, addr_of_mut},
};

use crate::internal::PeekIter;

#[cfg(target_pointer_width = "64")]
pub(crate) type HalfUsize = u32;
#[cfg(target_pointer_width = "32")]
pub(crate) type HalfUsize = u16;
#[cfg(target_pointer_width = "16")]
pub(crate) type HalfUsize = u8;

const _: () = assert!(HalfUsize::BITS == usize::BITS / 2);

#[repr(transparent)]
pub(crate) struct UniquePtr<T: ?Sized>(*mut T);

impl<T: ?Sized> UniquePtr<T> {
    /// Create a [`UniquePtr`]. Semantically it's equivalent to
    /// creating a `&mut T`, meaning that during the lifetime of [`UniquePtr`]
    /// its UB to create and use another [`UniquePtr`] to the same memory location.
    /// # Safety
    /// Caller guarantees that the pointer is valid for casting to &mut during the
    /// lifespan of self.
    #[inline]
    pub(crate) const unsafe fn new(pointer: *mut T) -> Self {
        Self(pointer)
    }

    /// Extracts the internal pointer without consuming self.
    /// Be careful to not use it in a way that invalidates the [`Self::new`] safety conditions.
    #[inline]
    pub(crate) const fn as_ptr(&self) -> *mut T {
        self.0
    }

    // Extracts the internal pointer
    #[inline]
    pub(crate) const fn into_ptr(self) -> *mut T {
        self.0
    }

    /// Creates a [`UniqueOwningPtr`] from this [`UniquePtr`].
    ///
    /// # Safety:
    /// Caller guarantees all preconditions of [`UniqueOwningPtr::new`]
    #[inline]
    pub(crate) const unsafe fn into_owning_ptr(self) -> UniqueOwningPtr<T> {
        // SAFETY: Caller guaranteed that it's safe
        unsafe { UniqueOwningPtr::new(self.0) }
    }
}

impl<T: ?Sized> Deref for UniquePtr<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<T: ?Sized> DerefMut for UniquePtr<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

#[repr(transparent)]
pub(crate) struct UniqueOwningPtr<T: ?Sized>(*mut T);

impl<T: ?Sized> UniqueOwningPtr<T> {
    /// Create a [`UniqueOwningPtr`]. Semantically it's equivalent to
    /// having an owned `T`, meaning that during the lifetime of [`UniqueOwningPtr`]
    /// its UB to access it in any other way than through [`UniqueOwningPtr`].
    /// # Safety
    /// Caller guarantees that the pointer is valid for casting to &mut during the
    /// lifespan of self, and that it's safe to drop the pointee or read the owned T.
    #[inline]
    pub(crate) const unsafe fn new(pointer: *mut T) -> Self {
        Self(pointer)
    }

    /// Extract the raw *mut T without causing the pointee to be dropped
    #[inline]
    pub(crate) const fn into_ptr(self) -> *mut T {
        let ptr = self.0;
        mem::forget(self); // do not drop the T when self is dropped
        ptr
    }

    #[inline]
    pub(crate) fn read(self) -> T
    where
        T: Sized,
    {
        // SAFETY: Caller guaranteed that it's safe
        unsafe { self.into_ptr().read() }
    }
}

impl<T: ?Sized> Drop for UniqueOwningPtr<T> {
    fn drop(&mut self) {
        // SAFETY: Caller guaranteed that it's safe
        unsafe {
            self.0.drop_in_place();
        }
    }
}

impl<IT: Iterator> UniqueOwningPtr<PeekIter<IT>> {
    #[inline]
    pub(crate) fn into_last_item(self) -> IT::Item {
        #[allow(clippy::unneeded_field_pattern)]
        const _CHECK: () = {
            // We rely on PeekIter only having 2 fields and not implementing drop itself
            let PeekIter { item: _, iter: _ } = PeekIter::new(0, core::iter::empty());
            assert!(!mem::needs_drop::<PeekIter<core::iter::Empty<usize>>>());
        };

        let p = self.into_ptr();
        unsafe {
            let item = addr_of!((*p).item).read();
            addr_of_mut!((*p).iter).drop_in_place();
            item
        }
    }
}

impl<T: ?Sized> Deref for UniqueOwningPtr<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl<T: ?Sized> DerefMut for UniqueOwningPtr<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0 }
    }
}

#[inline]
pub(crate) fn addr_from_ref<T>(reference: &T) -> usize {
    ptr_to_usize(reference)
}

#[rustversion::since(1.84)]
#[inline]
pub(crate) fn ptr_to_usize<T>(p: *const T) -> usize {
    p.addr()
}

#[rustversion::before(1.84)]
#[inline]
pub(crate) fn ptr_to_usize<T>(p: *const T) -> usize {
    // exposes provenance because pointer to usize transmute (what addr() does)
    // is not safe to implement in user code (at least that's how I read the docs)
    p as *const () as usize
}

#[rustversion::since(1.87)]
#[inline]
/// # Safety
/// Caller guarantees that `old_item` > `old_base` and they are in the same allocation, and that
/// `new_base`'s allocation contains `new_base+(old_item-old_base)`
pub(crate) unsafe fn rebase_ptr<T: ?Sized>(
    old_base: *mut T, old_item: *mut T, new_base: *mut T,
) -> *mut T {
    debug_assert!(old_base.cast::<()>() <= old_item.cast::<()>());
    unsafe { new_base.byte_add(old_item.byte_offset_from_unsigned(old_base)) }
}

#[rustversion::before(1.87)]
#[inline]
/// # Safety
/// Caller guarantees that old_item > old_base and they are in the same allocation, and that
/// new_base's allocation contains new_base+(old_item-old_base)
pub(crate) unsafe fn rebase_ptr<T>(
    old_base: *mut T, old_item: *mut T, new_base: *mut T,
) -> *mut T {
    debug_assert!(old_base.cast::<()>() <= old_item.cast::<()>());
    unsafe {
        new_base
            .cast::<u8>()
            .offset(old_item.cast::<u8>().offset_from(old_base.cast::<u8>()))
            .cast::<T>()
    }
}
