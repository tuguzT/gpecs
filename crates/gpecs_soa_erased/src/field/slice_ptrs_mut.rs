use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr::{self, NonNull},
    slice,
};

use crate::soa::traits::FieldDescriptor;

use super::{
    ErasedFieldMutPtr, ErasedFieldPtr, ErasedFieldSlice, ErasedFieldSliceMut, ErasedFieldSlicePtr,
    ErasedFieldSlicePtrIter,
    assert::{check_buffer_align, check_layout, check_slice_buffer_len},
    error::{ErasedFieldSliceError, IntoValueError},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedFieldSliceMutPtr {
    desc: FieldDescriptor,
    ptr: *mut u8,
    len: usize,
}

impl ErasedFieldSliceMutPtr {
    #[inline]
    #[track_caller]
    pub fn new(
        desc: FieldDescriptor,
        buffer: *mut [u8],
        len: usize,
    ) -> Result<Self, ErasedFieldSliceError> {
        let ptr = buffer.cast();
        check_buffer_align(ptr, desc.layout())?;
        check_slice_buffer_len(buffer.len(), desc.layout().size(), len)?;

        Ok(Self { desc, ptr, len })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: *mut [u8], len: usize) -> Self {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer, len).expect("incorrect inputs");
        }

        let ptr = buffer.cast();
        Self { desc, ptr, len }
    }

    #[inline]
    pub fn from<T>(ptr: *mut [T]) -> Self {
        let len = ptr.len();
        let desc = FieldDescriptor::of::<T>();
        let buffer = ptr::slice_from_raw_parts_mut(ptr.cast(), desc.layout().size() * len);
        unsafe { Self::new_unchecked(desc, buffer, len) }
    }

    #[inline]
    pub fn into<T>(self) -> Result<*mut [T], IntoValueError<Self>> {
        let me = check_layout::<T, _>(self.desc.layout(), self)?;
        let Self { ptr, len, .. } = me;
        Ok(ptr::slice_from_raw_parts_mut(ptr.cast(), len))
    }

    #[inline]
    pub fn cast_const(self) -> ErasedFieldSlicePtr {
        let Self { desc, ptr, len } = self;
        let buffer = ptr::slice_from_raw_parts(ptr.cast_const(), desc.layout().size() * len);
        unsafe { ErasedFieldSlicePtr::new_unchecked(desc, buffer, len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedFieldSlice<'a> {
        let Self { desc, ptr, len } = self;
        let buffer = unsafe { slice::from_raw_parts(ptr, desc.layout().size() * len) };
        unsafe { ErasedFieldSlice::new_unchecked(desc, buffer, len) }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedFieldSliceMut<'a> {
        let Self { desc, ptr, len } = self;
        let buffer = unsafe { slice::from_raw_parts_mut(ptr, desc.layout().size() * len) };
        unsafe { ErasedFieldSliceMut::new_unchecked(desc, buffer, len) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { len, .. } = *self;
        len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn descriptor(&self) -> FieldDescriptor {
        let Self { desc, .. } = *self;
        desc
    }

    #[inline]
    pub fn buffer(&self) -> *mut [u8] {
        let Self { desc, ptr, len } = *self;
        ptr::slice_from_raw_parts_mut(ptr, desc.layout().size() * len)
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut u8 {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldMutPtr {
        let Self { desc, ptr, .. } = *self;
        let buffer = ptr::slice_from_raw_parts_mut(ptr, desc.layout().size());
        unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, *mut [u8], usize) {
        let Self { desc, ptr, len } = self;
        let buffer = ptr::slice_from_raw_parts_mut(ptr, desc.layout().size() * len);
        (desc, buffer, len)
    }

    #[inline]
    pub fn iter(&self) -> ErasedFieldSlicePtrIter {
        let Self { desc, ptr, len } = *self;
        let buffer = ptr::slice_from_raw_parts(ptr, desc.layout().size() * len);
        let slice = unsafe { ErasedFieldSlicePtr::new_unchecked(desc, buffer, len) };
        ErasedFieldSlicePtrIter::new(slice)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> ErasedFieldSliceMutPtrIter {
        let Self { desc, ptr, len } = *self;
        let buffer = ptr::slice_from_raw_parts_mut(ptr, desc.layout().size() * len);
        let slice = unsafe { ErasedFieldSliceMutPtr::new_unchecked(desc, buffer, len) };
        ErasedFieldSliceMutPtrIter::new(slice)
    }
}

impl IntoIterator for &ErasedFieldSliceMutPtr {
    type Item = ErasedFieldPtr;
    type IntoIter = ErasedFieldSlicePtrIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for &mut ErasedFieldSliceMutPtr {
    type Item = ErasedFieldMutPtr;
    type IntoIter = ErasedFieldSliceMutPtrIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl IntoIterator for ErasedFieldSliceMutPtr {
    type Item = ErasedFieldMutPtr;
    type IntoIter = ErasedFieldSliceMutPtrIter;

    fn into_iter(self) -> Self::IntoIter {
        ErasedFieldSliceMutPtrIter::new(self)
    }
}

pub struct ErasedFieldSliceMutPtrIter {
    desc: FieldDescriptor,
    buffer: NonNull<u8>,
    start: usize,
    end: usize,
}

impl ErasedFieldSliceMutPtrIter {
    #[inline]
    pub(super) fn new(slice: ErasedFieldSliceMutPtr) -> Self {
        let (desc, buffer, end) = slice.into_parts();
        let buffer = NonNull::new(buffer as *mut _).expect("slice ptr should be nonnull");

        Self {
            desc,
            buffer,
            start: 0,
            end,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { start, end, .. } = *self;
        end - start
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn as_slice(&self) -> ErasedFieldSliceMutPtr {
        let Self { desc, buffer, .. } = *self;
        let len = self.len();
        let buffer = ptr::slice_from_raw_parts_mut(buffer.as_ptr(), len * desc.layout().size());
        unsafe { ErasedFieldSliceMutPtr::new_unchecked(desc, buffer, len) }
    }

    #[inline]
    unsafe fn post_inc_start(&mut self, offset: usize) -> *mut u8 {
        let Self {
            buffer,
            ref desc,
            ref mut start,
            ..
        } = *self;

        let count = *start * desc.layout().size();
        let ptr = unsafe { buffer.as_ptr().add(count) };
        *start += offset;
        ptr
    }

    #[inline]
    unsafe fn pre_dec_end(&mut self, offset: usize) -> *mut u8 {
        let Self {
            buffer,
            ref desc,
            ref mut end,
            ..
        } = *self;

        *end -= offset;
        let count = *end * desc.layout().size();
        let ptr = unsafe { buffer.as_ptr().add(count) };
        ptr
    }
}

impl Debug for ErasedFieldSliceMutPtrIter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (desc, buffer, len) = self.as_slice().into_parts();
        f.debug_struct("ErasedFieldSliceMutPtrIter")
            .field("desc", &desc)
            .field("buffer", &buffer)
            .field("len", &len)
            .finish()
    }
}

impl Clone for ErasedFieldSliceMutPtrIter {
    fn clone(&self) -> Self {
        let Self {
            desc,
            buffer,
            start,
            end,
        } = *self;
        Self {
            desc,
            buffer,
            start,
            end,
        }
    }
}

#[allow(clippy::while_let_on_iterator)]
impl Iterator for ErasedFieldSliceMutPtrIter {
    type Item = ErasedFieldMutPtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if ErasedFieldSliceMutPtrIter::is_empty(self) {
            return None;
        }

        let Self { desc, .. } = *self;
        let ptr = unsafe { self.post_inc_start(1) };
        let buffer = ptr::slice_from_raw_parts_mut(ptr, desc.layout().size());
        let ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) };
        Some(ptr)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = ErasedFieldSliceMutPtrIter::len(self);
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        ErasedFieldSliceMutPtrIter::len(&self)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= ErasedFieldSliceMutPtrIter::len(self) {
            self.start = self.end;
            return None;
        }

        unsafe {
            self.post_inc_start(n);
        }
        self.next()
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        if ErasedFieldSliceMutPtrIter::is_empty(&self) {
            return init;
        }

        // this implementation consists of the following optimizations compared to the
        // default implementation:
        // - do-while loop, as is llvm's preferred loop shape,
        //   see https://releases.llvm.org/16.0.0/docs/LoopTerminology.html#more-canonical-loops
        // - bumps an index instead of a pointer since the latter case inhibits
        //   some optimizations, see #111603
        // - avoids Option wrapping/matching
        let Self { desc, buffer, .. } = self;
        let len = ErasedFieldSliceMutPtrIter::len(&self);
        let ptr = buffer.as_ptr();
        let mut acc = init;
        let mut i = 0;
        loop {
            // SAFETY: the loop iterates `i in 0..len`, which always is in bounds of
            // the slice allocation
            let item = unsafe {
                let data = ptr.add(i * desc.layout().size());
                let buffer = ptr::slice_from_raw_parts_mut(data, desc.layout().size());
                ErasedFieldMutPtr::new_unchecked(desc, buffer)
            };
            acc = f(acc, item);
            // SAFETY: `i` can't overflow since it'll only reach usize::MAX if the
            // slice had that length, in which case we'll break out of the loop
            // after the increment
            i = unsafe { i.unchecked_add(1) };
            if i == len {
                break;
            }
        }
        acc
    }

    #[inline]
    fn for_each<F>(mut self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        while let Some(x) = self.next() {
            f(x);
        }
    }

    #[inline]
    fn all<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        while let Some(x) = self.next() {
            if !f(x) {
                return false;
            }
        }
        true
    }

    #[inline]
    fn any<F>(&mut self, mut f: F) -> bool
    where
        Self: Sized,
        F: FnMut(Self::Item) -> bool,
    {
        while let Some(x) = self.next() {
            if f(x) {
                return true;
            }
        }
        false
    }

    #[inline]
    fn find<P>(&mut self, mut predicate: P) -> Option<Self::Item>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool,
    {
        while let Some(x) = self.next() {
            if predicate(&x) {
                return Some(x);
            }
        }
        None
    }

    #[inline]
    fn find_map<B, F>(&mut self, mut f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> Option<B>,
    {
        while let Some(x) = self.next() {
            if let Some(y) = f(x) {
                return Some(y);
            }
        }
        None
    }

    #[inline]
    fn position<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        Self: Sized,
        P: FnMut(Self::Item) -> bool,
    {
        let n = ErasedFieldSliceMutPtrIter::len(self);
        let mut i = 0;
        while let Some(x) = self.next() {
            if predicate(x) {
                assert!(i < n);
                return Some(i);
            }
            i += 1;
        }
        None
    }

    #[inline]
    fn rposition<P>(&mut self, mut predicate: P) -> Option<usize>
    where
        P: FnMut(Self::Item) -> bool,
        Self: Sized + ExactSizeIterator + DoubleEndedIterator,
    {
        let n = ErasedFieldSliceMutPtrIter::len(self);
        let mut i = n;
        while let Some(x) = self.next_back() {
            i -= 1;
            if predicate(x) {
                assert!(i < n);
                return Some(i);
            }
        }
        None
    }
}

impl DoubleEndedIterator for ErasedFieldSliceMutPtrIter {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if ErasedFieldSliceMutPtrIter::is_empty(self) {
            return None;
        }

        let Self { desc, .. } = *self;
        let ptr = unsafe { self.pre_dec_end(1) };
        let buffer = ptr::slice_from_raw_parts_mut(ptr, desc.layout().size());
        let ptr = unsafe { ErasedFieldMutPtr::new_unchecked(desc, buffer) };
        Some(ptr)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= ErasedFieldSliceMutPtrIter::len(self) {
            self.end = self.start;
            return None;
        }

        unsafe {
            self.pre_dec_end(n);
        }
        self.next_back()
    }
}

impl ExactSizeIterator for ErasedFieldSliceMutPtrIter {
    fn len(&self) -> usize {
        ErasedFieldSliceMutPtrIter::len(self)
    }
}

impl FusedIterator for ErasedFieldSliceMutPtrIter {}

#[inline]
pub fn field_slice_from_raw_parts_mut(
    data: ErasedFieldMutPtr,
    len: usize,
) -> ErasedFieldSliceMutPtr {
    let (desc, data) = data.into_parts();
    let buffer = ptr::slice_from_raw_parts_mut(data.cast(), len * desc.layout().size());
    unsafe { ErasedFieldSliceMutPtr::new_unchecked(desc, buffer, len) }
}
