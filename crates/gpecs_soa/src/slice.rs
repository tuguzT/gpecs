use core::{
    fmt::{self, Debug},
    marker::PhantomData,
    ops::{Index, IndexMut},
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
pub unsafe trait SoaSliceIndex<T>
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

unsafe impl<T, U, V, I> SoaSliceIndex<SoaSlice<T, U, V>> for I
where
    I: slice::SliceIndex<[T]> + slice::SliceIndex<[U]> + slice::SliceIndex<[V]> + Clone,
    for<'any> <I as slice::SliceIndex<[T]>>::Output: 'any,
    for<'any> <I as slice::SliceIndex<[U]>>::Output: 'any,
    for<'any> <I as slice::SliceIndex<[V]>>::Output: 'any,
{
    type Ref<'a> = (
        &'a <I as slice::SliceIndex<[T]>>::Output,
        &'a <I as slice::SliceIndex<[U]>>::Output,
        &'a <I as slice::SliceIndex<[V]>>::Output,
    )
    where
        SoaSlice<T, U, V>: 'a;

    type RefMut<'a> = (
        &'a mut <I as slice::SliceIndex<[T]>>::Output,
        &'a mut <I as slice::SliceIndex<[U]>>::Output,
        &'a mut <I as slice::SliceIndex<[V]>>::Output,
    )
    where
        SoaSlice<T, U, V>: 'a;

    fn get(self, slice: &SoaSlice<T, U, V>) -> Option<Self::Ref<'_>> {
        let (t_slice, u_slice, v_slice) = slice.as_slices();
        let t_output = t_slice.get(self.clone())?;
        let u_output = u_slice.get(self.clone())?;
        let v_output = v_slice.get(self)?;
        Some((t_output, u_output, v_output))
    }

    fn get_mut(self, slice: &mut SoaSlice<T, U, V>) -> Option<Self::RefMut<'_>> {
        let (t_slice, u_slice, v_slice) = slice.as_mut_slices();
        let t_output = t_slice.get_mut(self.clone())?;
        let u_output = u_slice.get_mut(self.clone())?;
        let v_output = v_slice.get_mut(self)?;
        Some((t_output, u_output, v_output))
    }

    fn index(self, slice: &SoaSlice<T, U, V>) -> Self::Ref<'_> {
        let (t_slice, u_slice, v_slice) = slice.as_slices();
        let t_output = t_slice.index(self.clone());
        let u_output = u_slice.index(self.clone());
        let v_output = v_slice.index(self);
        (t_output, u_output, v_output)
    }

    fn index_mut(self, slice: &mut SoaSlice<T, U, V>) -> Self::RefMut<'_> {
        let (t_slice, u_slice, v_slice) = slice.as_mut_slices();
        let t_output = t_slice.index_mut(self.clone());
        let u_output = u_slice.index_mut(self.clone());
        let v_output = v_slice.index_mut(self);
        (t_output, u_output, v_output)
    }

    type ConstPtr = (
        *const <I as slice::SliceIndex<[T]>>::Output,
        *const <I as slice::SliceIndex<[U]>>::Output,
        *const <I as slice::SliceIndex<[V]>>::Output,
    );

    type MutPtr = (
        *mut <I as slice::SliceIndex<[T]>>::Output,
        *mut <I as slice::SliceIndex<[U]>>::Output,
        *mut <I as slice::SliceIndex<[V]>>::Output,
    );

    unsafe fn get_unchecked(self, slice: *const SoaSlice<T, U, V>) -> Self::ConstPtr {
        let slice = unsafe { &*slice };
        let (t_slice, u_slice, v_slice) = slice.as_slices();

        unsafe {
            let t_output = t_slice.get_unchecked(self.clone());
            let u_output = u_slice.get_unchecked(self.clone());
            let v_output = v_slice.get_unchecked(self);
            (t_output, u_output, v_output)
        }
    }

    unsafe fn get_unchecked_mut(self, slice: *mut SoaSlice<T, U, V>) -> Self::MutPtr {
        let slice = unsafe { &mut *slice };
        let (t_slice, u_slice, v_slice) = slice.as_mut_slices();

        unsafe {
            let t_output = t_slice.get_unchecked_mut(self.clone());
            let u_output = u_slice.get_unchecked_mut(self.clone());
            let v_output = v_slice.get_unchecked_mut(self);
            (t_output, u_output, v_output)
        }
    }
}
