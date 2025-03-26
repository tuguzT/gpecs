use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
};

use crate::traits::FieldDescriptor;

use super::{
    assert::{assert_buffer_align, assert_layout, assert_slice_buffer_len},
    ErasedFieldPtr, ErasedFieldRef, ErasedFieldSlicePtr,
};

#[derive(Clone, Copy)]
pub struct ErasedFieldSlice<'a> {
    desc: FieldDescriptor,
    ptr: *const u8,
    len: usize,
    phantom: PhantomData<&'a [u8]>,
}

impl<'a> ErasedFieldSlice<'a> {
    #[inline]
    #[track_caller]
    pub fn new(desc: FieldDescriptor, buffer: &'a [u8], len: usize) -> Self {
        assert_slice_buffer_len(buffer.len(), desc.layout().size(), len);
        assert_buffer_align(buffer.as_ptr(), desc.layout().align());

        let ptr = buffer.as_ptr();
        Self {
            desc,
            ptr,
            len,
            phantom: PhantomData,
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: &'a [u8], len: usize) -> Self {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer, len);
        }

        let ptr = buffer.as_ptr();
        Self {
            desc,
            ptr,
            len,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(ptr: &'a [T]) -> Self {
        let len = ptr.len();
        let desc = FieldDescriptor::of::<T>();
        let data = ptr.as_ptr().cast();
        let buffer = unsafe { slice::from_raw_parts(data, desc.layout().size() * len) };
        unsafe { Self::new_unchecked(desc, buffer, len) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn into<T>(self) -> &'a [T] {
        let Self { desc, ptr, len, .. } = self;
        assert_layout::<T>(desc.layout());

        let data = ptr.cast();
        unsafe { slice::from_raw_parts(data, len) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn cast<T>(&self) -> &[T] {
        let Self { desc, ptr, len, .. } = *self;
        assert_layout::<T>(desc.layout());

        let data = ptr.cast();
        unsafe { slice::from_raw_parts(data, len) }
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
    pub fn buffer(&self) -> &[u8] {
        let Self { desc, ptr, len, .. } = *self;
        unsafe { slice::from_raw_parts(ptr, desc.layout().size() * len) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptr, .. } = *self;
        ptr
    }

    #[inline]
    pub fn as_field_slice_ptr(&self) -> ErasedFieldSlicePtr {
        let Self { desc, ptr, len, .. } = *self;
        let buffer = unsafe { slice::from_raw_parts(ptr, desc.layout().size() * len) };
        unsafe { ErasedFieldSlicePtr::new_unchecked(desc, buffer, len) }
    }

    #[inline]
    pub fn as_field_ptr(&self) -> ErasedFieldPtr {
        let Self { desc, ptr, .. } = *self;
        let buffer = ptr::slice_from_raw_parts(ptr, desc.layout().size());
        unsafe { ErasedFieldPtr::new_unchecked(desc, buffer) }
    }

    #[inline]
    pub fn into_buffer(self) -> &'a [u8] {
        let (_, buffer, _) = self.into_parts();
        buffer
    }

    #[inline]
    pub fn into_parts(self) -> (FieldDescriptor, &'a [u8], usize) {
        let Self { desc, ptr, len, .. } = self;
        let buffer = unsafe { slice::from_raw_parts(ptr, desc.layout().size() * len) };
        (desc, buffer, len)
    }

    #[inline]
    pub fn iter(&self) -> ErasedFieldSliceIter<'_> {
        let Self { desc, ptr, len, .. } = *self;
        let buffer = unsafe { slice::from_raw_parts(ptr, desc.layout().size() * len) };
        let slice = unsafe { ErasedFieldSlice::new_unchecked(desc, buffer, len) };
        ErasedFieldSliceIter::new(slice)
    }
}

impl Debug for ErasedFieldSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { desc, len, .. } = self;
        let buffer = &self.buffer();
        f.debug_struct("ErasedFieldSlice")
            .field("desc", desc)
            .field("buffer", buffer)
            .field("len", len)
            .finish()
    }
}

impl<'a> IntoIterator for &'a ErasedFieldSlice<'_> {
    type Item = ErasedFieldRef<'a>;
    type IntoIter = ErasedFieldSliceIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for ErasedFieldSlice<'a> {
    type Item = ErasedFieldRef<'a>;
    type IntoIter = ErasedFieldSliceIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ErasedFieldSliceIter::new(self)
    }
}

pub struct ErasedFieldSliceIter<'a> {
    desc: FieldDescriptor,
    buffer: NonNull<u8>,
    start: usize,
    end: usize,
    marker: PhantomData<&'a [u8]>,
}

impl<'a> ErasedFieldSliceIter<'a> {
    #[inline]
    pub(super) fn new(slice: ErasedFieldSlice<'a>) -> Self {
        let (desc, buffer, end) = slice.into_parts();
        let buffer = NonNull::new(buffer.as_ptr().cast_mut()).expect("slice ptr should be nonnull");

        Self {
            desc,
            buffer,
            start: 0,
            end,
            marker: PhantomData,
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
    pub fn as_slice(&self) -> ErasedFieldSlice<'_> {
        let Self { desc, buffer, .. } = *self;
        let len = self.len();
        let buffer = unsafe { slice::from_raw_parts(buffer.as_ptr(), len * desc.layout().size()) };
        unsafe { ErasedFieldSlice::new_unchecked(desc, buffer, len) }
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

impl Debug for ErasedFieldSliceIter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (desc, buffer, len) = self.as_slice().into_parts();
        f.debug_struct("ErasedFieldSliceIter")
            .field("desc", &desc)
            .field("buffer", &buffer)
            .field("len", &len)
            .finish()
    }
}

impl Clone for ErasedFieldSliceIter<'_> {
    fn clone(&self) -> Self {
        Self {
            desc: self.desc.clone(),
            buffer: self.buffer.clone(),
            start: self.start.clone(),
            end: self.end.clone(),
            marker: self.marker.clone(),
        }
    }
}

#[allow(clippy::while_let_on_iterator)]
impl<'a> Iterator for ErasedFieldSliceIter<'a> {
    type Item = ErasedFieldRef<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if ErasedFieldSliceIter::is_empty(self) {
            return None;
        }

        let ptr = unsafe { self.post_inc_start(1) };
        let buffer = unsafe { slice::from_raw_parts(ptr, self.desc.layout().size()) };
        let item = unsafe { ErasedFieldRef::new_unchecked(self.desc, buffer) };
        Some(item)
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
        if ErasedFieldSliceIter::is_empty(&self) {
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
                let buffer = slice::from_raw_parts(data, self.desc.layout().size());
                ErasedFieldRef::new_unchecked(self.desc, buffer)
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

impl DoubleEndedIterator for ErasedFieldSliceIter<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if ErasedFieldSliceIter::is_empty(self) {
            return None;
        }

        let ptr = unsafe { self.pre_dec_end(1) };
        let buffer = unsafe { slice::from_raw_parts(ptr, self.desc.layout().size()) };
        let item = unsafe { ErasedFieldRef::new_unchecked(self.desc, buffer) };
        Some(item)
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

impl ExactSizeIterator for ErasedFieldSliceIter<'_> {
    fn len(&self) -> usize {
        ErasedFieldSliceIter::len(self)
    }
}

impl FusedIterator for ErasedFieldSliceIter<'_> {}
