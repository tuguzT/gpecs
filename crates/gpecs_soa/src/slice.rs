use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
};

use crate::ptr::{
    min_size_of, ptrs, slice_from_len_in_bytes, slice_from_len_in_bytes_mut, slice_from_raw_parts,
    slice_from_raw_parts_mut, to_len,
};

#[repr(transparent)]
pub struct SoaSlice<T, U, V> {
    phantom: PhantomData<(T, U, V)>,
    buffer: [u8],
}

impl<T, U, V> SoaSlice<T, U, V> {
    #[inline]
    pub const fn len(&self) -> usize {
        match self.buffer_capacity() {
            0 => 0,
            _ => unsafe { ptr::read(self.as_ptr().cast()) },
        }
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub const fn buffer_capacity(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        if min_size_of::<T, U, V>() == 0 {
            usize::MAX
        } else {
            let len_in_bytes = self.buffer_capacity();
            to_len::<T, U, V>(len_in_bytes)
        }
    }

    #[inline]
    pub const fn as_ptr(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.buffer.as_mut_ptr()
    }

    #[inline]
    pub fn as_ptrs(&self) -> (*const T, *const U, *const V) {
        let ptr = self.as_ptr().cast_mut();
        let len = self.capacity();

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = ptrs::<T, U, V>(ptr, len);
            (t_ptr, u_ptr, v_ptr)
        }
    }

    #[inline]
    pub fn as_mut_ptrs(&mut self) -> (*mut T, *mut U, *mut V) {
        let ptr = self.as_mut_ptr();
        let len = self.capacity();

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = ptrs::<T, U, V>(ptr, len);
            (t_ptr, u_ptr, v_ptr)
        }
    }

    #[inline]
    pub fn as_slices(&self) -> (&[T], &[U], &[V]) {
        let (t_data, u_data, v_data) = self.as_ptrs();
        let len = self.len();

        unsafe {
            let t_slice = slice::from_raw_parts(t_data, len);
            let u_slice = slice::from_raw_parts(u_data, len);
            let v_slice = slice::from_raw_parts(v_data, len);
            (t_slice, u_slice, v_slice)
        }
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> (&mut [T], &mut [U], &mut [V]) {
        let (t_data, u_data, v_data) = self.as_mut_ptrs();
        let len = self.len();

        unsafe {
            let t_slice = slice::from_raw_parts_mut(t_data, len);
            let u_slice = slice::from_raw_parts_mut(u_data, len);
            let v_slice = slice::from_raw_parts_mut(v_data, len);
            (t_slice, u_slice, v_slice)
        }
    }

    #[inline]
    pub fn get<I>(&self, index: I) -> Option<I::Ref<'_>>
    where
        I: SoaSliceIndex<Self>,
    {
        index.get(self)
    }

    #[inline]
    pub fn get_mut<I>(&mut self, index: I) -> Option<I::RefMut<'_>>
    where
        I: SoaSliceIndex<Self>,
    {
        index.get_mut(self)
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked<I>(&self, index: I) -> I::ConstPtr
    where
        I: SoaSliceIndex<Self>,
    {
        unsafe { index.get_unchecked(self) }
    }

    #[inline]
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn get_unchecked_mut<I>(&mut self, index: I) -> I::MutPtr
    where
        I: SoaSliceIndex<Self>,
    {
        unsafe { index.get_unchecked_mut(self) }
    }

    #[inline]
    pub fn index<I>(&self, index: I) -> I::Ref<'_>
    where
        I: SoaSliceIndex<Self>,
    {
        index.index(self)
    }

    #[inline]
    pub fn index_mut<I>(&mut self, index: I) -> I::RefMut<'_>
    where
        I: SoaSliceIndex<Self>,
    {
        index.index_mut(self)
    }
}

impl<T, U, V> Debug for SoaSlice<T, U, V>
where
    T: Debug,
    U: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (t_slice, u_slice, v_slice) = self.as_slices();
        f.debug_struct("MultiSlice")
            .field("t_slice", &t_slice)
            .field("u_slice", &u_slice)
            .field("v_slice", &v_slice)
            .finish()
    }
}

impl<T, U, V> Default for &SoaSlice<T, U, V> {
    fn default() -> Self {
        let data = NonNull::dangling().as_ptr();
        unsafe { from_len_in_bytes(data, 0) }
    }
}

impl<T, U, V> Default for &mut SoaSlice<T, U, V> {
    fn default() -> Self {
        let data = NonNull::dangling().as_ptr();
        unsafe { from_len_in_bytes_mut(data, 0) }
    }
}

impl<T, U, V> Drop for SoaSlice<T, U, V> {
    fn drop(&mut self) {
        if self.is_empty() {
            return;
        }

        let (t_slice, u_slice, v_slice) = self.as_mut_slices();
        unsafe {
            ptr::drop_in_place(t_slice);
            ptr::drop_in_place(u_slice);
            ptr::drop_in_place(v_slice);
        }
    }
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn from_raw_parts<'slice, T, U, V>(
    data: *const u8,
    capacity: usize,
) -> &'slice SoaSlice<T, U, V> {
    unsafe { &*slice_from_raw_parts(data, capacity) }
}

#[allow(clippy::missing_safety_doc)]
#[inline]
pub unsafe fn from_raw_parts_mut<'slice, T, U, V>(
    data: *mut u8,
    capacity: usize,
) -> &'slice mut SoaSlice<T, U, V> {
    unsafe { &mut *slice_from_raw_parts_mut(data, capacity) }
}

#[inline]
pub(crate) unsafe fn from_len_in_bytes<'slice, T, U, V>(
    data: *const u8,
    len_in_bytes: usize,
) -> &'slice SoaSlice<T, U, V> {
    unsafe { &*slice_from_len_in_bytes(data, len_in_bytes) }
}

#[inline]
pub(crate) unsafe fn from_len_in_bytes_mut<'slice, T, U, V>(
    data: *mut u8,
    len_in_bytes: usize,
) -> &'slice mut SoaSlice<T, U, V> {
    unsafe { &mut *slice_from_len_in_bytes_mut(data, len_in_bytes) }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe trait SoaSliceIndex<T>: private_slice_index::Sealed
where
    T: ?Sized,
{
    type Ref<'a>
    where
        T: 'a;

    type RefMut<'a>
    where
        T: 'a;

    fn get(self, slice: &T) -> Option<Self::Ref<'_>>;

    fn get_mut(self, slice: &mut T) -> Option<Self::RefMut<'_>>;

    fn index(self, slice: &T) -> Self::Ref<'_>;

    fn index_mut(self, slice: &mut T) -> Self::RefMut<'_>;

    type ConstPtr;

    type MutPtr;

    unsafe fn get_unchecked(self, slice: *const T) -> Self::ConstPtr;

    unsafe fn get_unchecked_mut(self, slice: *mut T) -> Self::MutPtr;
}

unsafe impl<T, U, V> SoaSliceIndex<SoaSlice<T, U, V>> for usize {
    type Ref<'a> = (&'a T, &'a U, &'a V)
    where
        SoaSlice<T, U, V>: 'a;

    type RefMut<'a> = (&'a mut T, &'a mut U, &'a mut V)
    where
        SoaSlice<T, U, V>: 'a;

    fn get(self, slice: &SoaSlice<T, U, V>) -> Option<Self::Ref<'_>> {
        if self >= slice.len() {
            return None;
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.get_unchecked(slice);
            Some((&*t_ptr, &*u_ptr, &*v_ptr))
        }
    }

    fn get_mut(self, slice: &mut SoaSlice<T, U, V>) -> Option<Self::RefMut<'_>> {
        if self >= slice.len() {
            return None;
        }

        unsafe {
            let (t_ptr, u_ptr, v_ptr) = self.get_unchecked_mut(slice);
            Some((&mut *t_ptr, &mut *u_ptr, &mut *v_ptr))
        }
    }

    fn index(self, slice: &SoaSlice<T, U, V>) -> Self::Ref<'_> {
        match self.get(slice) {
            Some(value) => value,
            None => slice_index_usize_fail(slice.len(), self),
        }
    }

    fn index_mut(self, slice: &mut SoaSlice<T, U, V>) -> Self::RefMut<'_> {
        let len = slice.len();
        match self.get_mut(slice) {
            Some(value) => value,
            None => slice_index_usize_fail(len, self),
        }
    }

    type ConstPtr = (*const T, *const U, *const V);

    type MutPtr = (*mut T, *mut U, *mut V);

    unsafe fn get_unchecked(self, slice: *const SoaSlice<T, U, V>) -> Self::ConstPtr {
        unsafe {
            debug_assert!(self < (*slice).len());
        }

        let buffer = slice as *const [u8];
        let ptr = buffer as _;
        let len = to_len::<T, U, V>(buffer.len());
        unsafe {
            let (t_ptr, u_ptr, v_ptr) = ptrs::<T, U, V>(ptr, len);
            (t_ptr.add(self), u_ptr.add(self), v_ptr.add(self))
        }
    }

    unsafe fn get_unchecked_mut(self, slice: *mut SoaSlice<T, U, V>) -> Self::MutPtr {
        unsafe {
            debug_assert!(self < (*slice).len());
        }

        let buffer = slice as *mut [u8];
        let ptr = buffer as _;
        let len = to_len::<T, U, V>(buffer.len());
        unsafe {
            let (t_ptr, u_ptr, v_ptr) = ptrs::<T, U, V>(ptr, len);
            (t_ptr.add(self), u_ptr.add(self), v_ptr.add(self))
        }
    }
}

#[cold]
#[inline(never)]
#[track_caller]
fn slice_index_usize_fail(len: usize, index: usize) -> ! {
    panic!("index out of bounds: the len is {len} but the index is {index}")
}

mod private_slice_index {
    use core::ops::{
        Bound, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
    };

    pub trait Sealed {}

    impl Sealed for usize {}

    impl Sealed for Range<usize> {}

    impl Sealed for RangeFrom<usize> {}

    impl Sealed for RangeFull {}

    impl Sealed for RangeTo<usize> {}

    impl Sealed for RangeInclusive<usize> {}

    impl Sealed for RangeToInclusive<usize> {}

    impl Sealed for (Bound<usize>, Bound<usize>) {}
}
