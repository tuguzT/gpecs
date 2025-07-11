use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr::{self, NonNull},
    slice,
};

use crate::soa::traits::FieldDescriptor;

use super::{
    ErasedFieldPtr, ErasedFieldSlice, ErasedFieldSliceMutPtr,
    assert::{check_buffer_align, check_layout, check_slice_buffer_len},
    error::{ErasedFieldSliceError, IntoValueError},
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedFieldSlicePtr {
    desc: FieldDescriptor,
    ptr: *const u8,
    len: usize,
}

impl ErasedFieldSlicePtr {
    #[inline]
    #[track_caller]
    pub fn new(
        desc: FieldDescriptor,
        buffer: *const [u8],
        len: usize,
    ) -> Result<Self, ErasedFieldSliceError> {
        let ptr = buffer.cast();
        check_buffer_align(ptr, desc.layout())?;
        check_slice_buffer_len(buffer.len(), desc.layout().size(), len)?;

        Ok(Self { desc, ptr, len })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: *const [u8], len: usize) -> Self {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer, len).expect("incorrect inputs");
        }

        let ptr = buffer.cast();
        Self { desc, ptr, len }
    }

    #[inline]
    pub fn from<T>(ptr: *const [T]) -> Self {
        let len = ptr.len();
        let desc = FieldDescriptor::of::<T>();
        let buffer = ptr::slice_from_raw_parts(ptr.cast(), desc.layout().size() * len);
        unsafe { Self::new_unchecked(desc, buffer, len) }
    }

    #[inline]
    pub fn into<T>(self) -> Result<*const [T], IntoValueError<Self>> {
        let me = check_layout::<T, _>(self.desc.layout(), self)?;
        let Self { ptr, len, .. } = me;
        Ok(ptr::slice_from_raw_parts(ptr.cast(), len))
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedFieldSliceMutPtr {
        let Self { desc, ptr, len } = self;
        let buffer = ptr::slice_from_raw_parts_mut(ptr.cast_mut(), desc.layout().size() * len);
        unsafe { ErasedFieldSliceMutPtr::new_unchecked(desc, buffer, len) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedFieldSlice<'a> {
        let Self { desc, ptr, len } = self;
        let buffer = unsafe { slice::from_raw_parts(ptr, desc.layout().size() * len) };
        unsafe { ErasedFieldSlice::new_unchecked(desc, buffer, len) }
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
    pub fn buffer(&self) -> *const [u8] {
        let Self { desc, ptr, len } = *self;
        ptr::slice_from_raw_parts(ptr, len * desc.layout().size())
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let Self { desc, ptr, .. } = *self;
        let buffer = ptr::slice_from_raw_parts(ptr, desc.layout().size());
        unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, *const [u8], usize) {
        let Self { desc, ptr, len } = self;
        let buffer = ptr::slice_from_raw_parts(ptr, len * desc.layout().size());
        (desc, buffer, len)
    }

    #[inline]
    pub fn iter(&self) -> ErasedFieldSlicePtrIter {
        let Self { desc, ptr, len } = *self;
        let buffer = ptr::slice_from_raw_parts(ptr, len * desc.layout().size());
        let slice = unsafe { ErasedFieldSlicePtr::new_unchecked(desc, buffer, len) };
        ErasedFieldSlicePtrIter::new(slice)
    }
}

impl IntoIterator for &ErasedFieldSlicePtr {
    type Item = ErasedFieldPtr;
    type IntoIter = ErasedFieldSlicePtrIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for ErasedFieldSlicePtr {
    type Item = ErasedFieldPtr;
    type IntoIter = ErasedFieldSlicePtrIter;

    fn into_iter(self) -> Self::IntoIter {
        ErasedFieldSlicePtrIter::new(self)
    }
}

pub struct ErasedFieldSlicePtrIter {
    desc: FieldDescriptor,
    buffer: NonNull<u8>,
    start: usize,
    end: usize,
}

impl ErasedFieldSlicePtrIter {
    #[inline]
    pub(super) fn new(slice: ErasedFieldSlicePtr) -> Self {
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
    pub fn as_slice(&self) -> ErasedFieldSlicePtr {
        let Self { desc, buffer, .. } = *self;
        let len = self.len();
        let buffer = ptr::slice_from_raw_parts(buffer.as_ptr(), len * desc.layout().size());
        unsafe { ErasedFieldSlicePtr::new_unchecked(desc, buffer, len) }
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

impl Debug for ErasedFieldSlicePtrIter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (desc, buffer, len) = self.as_slice().into_parts();
        f.debug_struct("ErasedFieldSlicePtrIter")
            .field("desc", &desc)
            .field("buffer", &buffer)
            .field("len", &len)
            .finish()
    }
}

impl Clone for ErasedFieldSlicePtrIter {
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
impl<'a> Iterator for ErasedFieldSlicePtrIter {
    type Item = ErasedFieldPtr;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if ErasedFieldSlicePtrIter::is_empty(self) {
            return None;
        }

        let Self { desc, .. } = *self;
        let ptr = unsafe { self.post_inc_start(1) };
        let buffer = ptr::slice_from_raw_parts(ptr, desc.layout().size());
        let ptr = unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) };
        Some(ptr)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = ErasedFieldSlicePtrIter::len(self);
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        ErasedFieldSlicePtrIter::len(&self)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= ErasedFieldSlicePtrIter::len(self) {
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
        if ErasedFieldSlicePtrIter::is_empty(&self) {
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
        let len = ErasedFieldSlicePtrIter::len(&self);
        let ptr = buffer.as_ptr();
        let mut acc = init;
        let mut i = 0;
        loop {
            // SAFETY: the loop iterates `i in 0..len`, which always is in bounds of
            // the slice allocation
            let item = unsafe {
                let data = ptr.add(i * desc.layout().size());
                let buffer = ptr::slice_from_raw_parts(data, desc.layout().size());
                ErasedFieldPtr::new_unchecked(desc, buffer)
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
        let n = ErasedFieldSlicePtrIter::len(self);
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
        let n = ErasedFieldSlicePtrIter::len(self);
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

impl DoubleEndedIterator for ErasedFieldSlicePtrIter {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if ErasedFieldSlicePtrIter::is_empty(self) {
            return None;
        }

        let Self { desc, .. } = *self;
        let ptr = unsafe { self.pre_dec_end(1) };
        let buffer = ptr::slice_from_raw_parts(ptr, desc.layout().size());
        let ptr = unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) };
        Some(ptr)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= ErasedFieldSlicePtrIter::len(self) {
            self.end = self.start;
            return None;
        }

        unsafe {
            self.pre_dec_end(n);
        }
        self.next_back()
    }
}

impl ExactSizeIterator for ErasedFieldSlicePtrIter {
    fn len(&self) -> usize {
        ErasedFieldSlicePtrIter::len(self)
    }
}

impl FusedIterator for ErasedFieldSlicePtrIter {}

#[inline]
pub fn field_slice_from_raw_parts(data: ErasedFieldPtr, len: usize) -> ErasedFieldSlicePtr {
    let (desc, data) = data.into_parts();
    let buffer = ptr::slice_from_raw_parts(data.cast(), len * desc.layout().size());
    unsafe { ErasedFieldSlicePtr::new_unchecked(desc, buffer, len) }
}
