use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr::{self, NonNull},
    slice,
};

use crate::traits::FieldDescriptor;

use super::{
    assert::{assert_buffer_align, assert_layout, assert_slice_buffer_len},
    ErasedFieldPtr, ErasedFieldSlice, ErasedFieldSliceMutPtr,
};

#[derive(Debug, Clone, Copy)]
pub struct ErasedFieldSlicePtr {
    desc: FieldDescriptor,
    // all the data is stored inline in a single buffer
    buffer: *const [u8],
}

impl ErasedFieldSlicePtr {
    #[inline]
    #[track_caller]
    pub fn new(desc: FieldDescriptor, buffer: *const [u8]) -> Self {
        assert_slice_buffer_len(buffer.len(), desc.layout().size());
        assert_buffer_align(buffer.cast(), desc.layout().align());

        Self { desc, buffer }
    }

    #[inline]
    pub fn from<T>(ptr: *const [T]) -> Self {
        let desc = FieldDescriptor::of::<T>();
        let buffer = ptr::slice_from_raw_parts(ptr.cast(), desc.layout().size() * ptr.len());
        Self::new(desc, buffer)
    }

    #[inline]
    #[track_caller]
    pub fn into<T>(self) -> *const [T] {
        let len = self.len();
        let Self { desc, buffer } = self;
        assert_layout::<T>(desc.layout());

        ptr::slice_from_raw_parts(buffer.cast(), len)
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedFieldSliceMutPtr {
        let Self { desc, buffer } = self;
        ErasedFieldSliceMutPtr::new(desc, buffer.cast_mut())
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedFieldSlice<'a> {
        let Self { desc, buffer } = self;
        let buffer = unsafe { slice::from_raw_parts(buffer.cast(), buffer.len()) };
        ErasedFieldSlice::new(desc, buffer)
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { desc, buffer } = *self;
        buffer.len().checked_div(desc.layout().size()).unwrap_or(0)
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
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { buffer, .. } = self;
        buffer.cast()
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let Self { desc, buffer } = *self;
        let buffer = ptr::slice_from_raw_parts(buffer.cast(), desc.layout().size());
        ErasedFieldPtr::new(desc, buffer)
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, *const [u8]) {
        let Self { desc, buffer } = self;
        (desc, buffer)
    }

    #[inline]
    pub fn iter(&self) -> ErasedFieldSlicePtrIter {
        let Self { desc, buffer, .. } = *self;
        let slice = ErasedFieldSlicePtr::new(desc, buffer);
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
        let end = slice.len();
        let (desc, buffer) = slice.into_parts();
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
        self.end - self.start
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn as_slice(&self) -> ErasedFieldSlicePtr {
        let len = self.len() * self.desc.layout().size();
        let buffer = ptr::slice_from_raw_parts(self.buffer.as_ptr(), len);
        ErasedFieldSlicePtr::new(self.desc, buffer)
    }

    #[inline]
    unsafe fn post_inc_start(&mut self, offset: usize) -> *mut u8 {
        let count = self.start * self.desc.layout().size();
        let ptr = unsafe { self.buffer.as_ptr().add(count) };

        self.start += offset;
        ptr
    }

    #[inline]
    unsafe fn pre_dec_end(&mut self, offset: usize) -> *mut u8 {
        self.end -= offset;

        let count = self.end * self.desc.layout().size();
        let ptr = unsafe { self.buffer.as_ptr().add(count) };
        ptr
    }
}

impl Debug for ErasedFieldSlicePtrIter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (desc, buffer) = self.as_slice().into_parts();
        f.debug_struct("ErasedFieldSlicePtrIter")
            .field("desc", &desc)
            .field("buffer", &buffer)
            .finish()
    }
}

impl Clone for ErasedFieldSlicePtrIter {
    fn clone(&self) -> Self {
        Self {
            desc: self.desc.clone(),
            buffer: self.buffer.clone(),
            start: self.start.clone(),
            end: self.end.clone(),
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

        let ptr = unsafe { self.post_inc_start(1) };
        let buffer = ptr::slice_from_raw_parts(ptr, self.desc.layout().size());
        Some(ErasedFieldPtr::new(self.desc, buffer))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len() {
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
        // this implementation consists of the following optimizations compared to the
        // default implementation:
        // - do-while loop, as is llvm's preferred loop shape,
        //   see https://releases.llvm.org/16.0.0/docs/LoopTerminology.html#more-canonical-loops
        // - bumps an index instead of a pointer since the latter case inhibits
        //   some optimizations, see #111603
        // - avoids Option wrapping/matching
        if ErasedFieldSlicePtrIter::is_empty(&self) {
            return init;
        }
        let mut acc = init;
        let mut i = 0;
        let len = self.len();
        let ptr = self.buffer.as_ptr();
        loop {
            // SAFETY: the loop iterates `i in 0..len`, which always is in bounds of
            // the slice allocation
            let item = unsafe {
                let data = ptr.add(i * self.desc.layout().size());
                let buffer = ptr::slice_from_raw_parts(data, self.desc.layout().size());
                ErasedFieldPtr::new(self.desc, buffer)
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
        let n = self.len();
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
        let n = self.len();
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

        let ptr = unsafe { self.pre_dec_end(1) };
        let buffer = ptr::slice_from_raw_parts(ptr, self.desc.layout().size());
        Some(ErasedFieldPtr::new(self.desc, buffer))
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len() {
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
