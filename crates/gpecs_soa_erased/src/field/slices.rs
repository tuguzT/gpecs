use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
};

use crate::{error::check_align, soa::traits::FieldDescriptor};

use super::{
    ErasedFieldPtr, ErasedFieldRef, ErasedFieldSlicePtr,
    assert::{check_into_layout, check_slice_buffer_len},
    error::{ErasedFieldSlicePtrError, IntoValueError},
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
    pub fn new(
        desc: FieldDescriptor,
        buffer: &'a [u8],
        len: usize,
    ) -> Result<Self, ErasedFieldSlicePtrError> {
        let ptr = buffer.as_ptr();
        check_align(buffer.as_ptr(), desc.layout())?;
        check_slice_buffer_len(buffer.len(), desc.layout().size(), len)?;

        Ok(Self {
            desc,
            ptr,
            len,
            phantom: PhantomData,
        })
    }

    #[inline]
    #[track_caller]
    pub unsafe fn new_unchecked(desc: FieldDescriptor, buffer: &'a [u8], len: usize) -> Self {
        if cfg!(debug_assertions) {
            return Self::new(desc, buffer, len).expect("incorrect inputs");
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
    pub unsafe fn into<T>(self) -> Result<&'a [T], IntoValueError<Self>> {
        let Self { desc, .. } = self;
        let me = check_into_layout::<T, _>(desc.layout(), self)?;
        let Self { ptr, len, .. } = me;

        let data = ptr.cast();
        Ok(unsafe { slice::from_raw_parts(data, len) })
    }

    #[inline]
    pub unsafe fn cast<T>(&self) -> Result<&[T], IntoValueError<&Self>> {
        let Self { desc, .. } = *self;
        let me = check_into_layout::<T, _>(desc.layout(), self)?;
        let Self { ptr, len, .. } = *me;

        let data = ptr.cast();
        Ok(unsafe { slice::from_raw_parts(data, len) })
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

impl AsRef<[u8]> for ErasedFieldSlice<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.buffer()
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
        let Self { start, end, .. } = *self;
        end - start
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
        let Self {
            desc,
            buffer,
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
            desc,
            buffer,
            ref mut end,
            ..
        } = *self;

        *end -= offset;
        let count = *end * desc.layout().size();
        unsafe { buffer.as_ptr().add(count) }
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
        let Self {
            desc,
            buffer,
            start,
            end,
            marker,
        } = *self;
        Self {
            desc,
            buffer,
            start,
            end,
            marker,
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

        let Self { desc, .. } = *self;
        let ptr = unsafe { self.post_inc_start(1) };
        let buffer = unsafe { slice::from_raw_parts(ptr, desc.layout().size()) };
        let item = unsafe { ErasedFieldRef::new_unchecked(desc, buffer) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = ErasedFieldSliceIter::len(self);
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        ErasedFieldSliceIter::len(&self)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= ErasedFieldSliceIter::len(self) {
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
        if ErasedFieldSliceIter::is_empty(&self) {
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
        let len = ErasedFieldSliceIter::len(&self);
        let ptr = buffer.as_ptr();
        let mut acc = init;
        let mut i = 0;
        loop {
            // SAFETY: the loop iterates `i in 0..len`, which always is in bounds of
            // the slice allocation
            let item = unsafe {
                let data = ptr.add(i * desc.layout().size());
                let buffer = slice::from_raw_parts(data, desc.layout().size());
                ErasedFieldRef::new_unchecked(desc, buffer)
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
        let n = ErasedFieldSliceIter::len(self);
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
        let n = ErasedFieldSliceIter::len(self);
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

        let Self { desc, .. } = *self;
        let ptr = unsafe { self.pre_dec_end(1) };
        let buffer = unsafe { slice::from_raw_parts(ptr, desc.layout().size()) };
        let item = unsafe { ErasedFieldRef::new_unchecked(desc, buffer) };
        Some(item)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= ErasedFieldSliceIter::len(self) {
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
