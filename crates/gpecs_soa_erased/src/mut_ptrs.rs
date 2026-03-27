use core::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::FusedIterator,
    mem::MaybeUninit,
    ptr,
};

use crate::{
    CovariantFieldDescriptors, ErasedSoa, ErasedSoaMutRefs, ErasedSoaPtrs, ErasedSoaPtrsIter,
    ErasedSoaRefs,
    assert::{assert_descriptors, check_downcast},
    dangling::{Dangling, dangling},
    data::{ErasedMutPtr, ErasedPtr},
    error::{
        DowncastError, InsufficientAlignError, PtrsError, check_offset, check_ptr_align,
        check_sufficient_align, check_sufficient_len,
    },
    layout::bytes_to_items,
    ptr::slice::{CastConstPtr, MutSliceItemPtr},
    soa::{
        field::{
            BufferOffset, BufferOffsets, FieldDescriptor, FieldDescriptors, FieldDescriptorsIter,
            FieldDescriptorsOutput, FieldDescriptorsOwned, buffer_offsets,
        },
        traits::{AllocSoa, AllocSoaContext, MutPtrs, RawSoaContext},
    },
    storage::AlignedStorage,
};

pub struct ErasedSoaMutPtrs<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    buffer: *mut [P::Item],
    capacity: usize,
    offset: usize,
    descriptors: D,
}

impl<D, P> ErasedSoaMutPtrs<D, P>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn new_unchecked(
        descriptors: D,
        buffer: *mut [P::Item],
        capacity: usize,
        offset: usize,
    ) -> Self {
        Self {
            buffer,
            capacity,
            offset,
            descriptors,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (D, *mut [P::Item], usize, usize) {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = self;
        (descriptors, buffer, capacity, offset)
    }

    #[inline]
    pub fn cast_const(self) -> ErasedSoaPtrs<D, CastConstPtr<P>> {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = self;

        let ptr = buffer.cast_const();
        unsafe { ErasedSoaPtrs::new_unchecked(descriptors, ptr, capacity, offset) }
    }

    #[inline]
    pub unsafe fn deref<'a>(self) -> ErasedSoaRefs<'a, D, CastConstPtr<P>> {
        unsafe { self.cast_const().deref() }
    }

    #[inline]
    pub unsafe fn deref_mut<'a>(self) -> ErasedSoaMutRefs<'a, D, P> {
        unsafe { ErasedSoaMutRefs::from_ptrs(self) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { offset, .. } = self;

        let offset = unsafe { offset.unchecked_add(count) };
        Self { offset, ..self }
    }
}

impl<D, P> ErasedSoaMutPtrs<D, P>
where
    D: FieldDescriptorsOwned,
    P: MutSliceItemPtr,
{
    #[inline]
    #[expect(clippy::not_unsafe_ptr_arg_deref, reason = "false positive")]
    pub fn new(
        descriptors: D,
        buffer: *mut [P::Item],
        capacity: usize,
        offset: usize,
    ) -> Result<Self, PtrsError> {
        check_offset(offset, capacity)?;

        let mut offsets = buffer_offsets(descriptors.field_descriptors(), capacity);
        offsets.by_ref().try_for_each(|offset| {
            check_sufficient_align(offset?.desc.layout(), Layout::new::<P::Item>())
                .map_err(PtrsError::from)
        })?;

        let layout = offsets.into_layout();
        check_ptr_align(buffer.cast(), layout)?;

        let buffer_layout = Layout::array::<P::Item>(buffer.len())?;
        check_sufficient_len(buffer_layout.size(), layout.size())?;

        let me = unsafe { Self::new_unchecked(descriptors, buffer, capacity, offset) };
        Ok(me)
    }

    #[inline]
    pub fn dangling(descriptors: D) -> Result<Self, InsufficientAlignError> {
        let Dangling { addr, capacity } = dangling::<_, P::Item>(descriptors.field_descriptors())?;

        let data = ptr::without_provenance_mut(addr);
        let buffer = ptr::slice_from_raw_parts_mut(data, 0);

        let me = unsafe { Self::new_unchecked(descriptors, buffer, capacity, 0) };
        Ok(me)
    }
}

impl<D, P> ErasedSoaMutPtrs<D, P>
where
    D: FieldDescriptorsOwned,
    P: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn downcast<T>(
        self,
        context: &T::Context,
    ) -> Result<MutPtrs<'_, T>, DowncastError<Self>>
    where
        T: AllocSoa + ?Sized,
    {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = self;

        let actual = descriptors.field_descriptors();
        let expected = context.field_descriptors();
        if let Err(error) = check_downcast(actual, expected, capacity) {
            return Err(DowncastError::new(self, error));
        }

        let ptrs = unsafe { context.ptrs_from_buffer_mut(buffer.cast(), capacity) };
        let ptrs = unsafe { context.ptrs_add_mut(ptrs, offset) };
        Ok(ptrs)
    }
}

impl<D, P> ErasedSoaMutPtrs<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [P::Item] {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> *mut [P::Item] {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { capacity, .. } = *self;
        capacity
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { offset, .. } = *self;
        offset
    }

    #[inline]
    pub fn descriptors(&self) -> &D {
        let Self { descriptors, .. } = self;
        descriptors
    }
}

impl<'a, D, P> ErasedSoaMutPtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    #[track_caller]
    pub unsafe fn offset_from<'e, E>(
        &'a self,
        origin: &'e ErasedSoaPtrs<E, CastConstPtr<P>>,
    ) -> isize
    where
        E: FieldDescriptors<'e> + ?Sized,
    {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = *self;

        assert_eq!(buffer.cast_const(), origin.as_buffer());
        assert_eq!(capacity, origin.capacity());
        assert_descriptors(descriptors.field_descriptors(), origin.field_descriptors());

        let offset = offset.cast_signed();
        let origin_offset = origin.offset().cast_signed();
        offset.wrapping_sub(origin_offset)
    }
}

impl<'a, D, P> ErasedSoaMutPtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedSoaPtrsIter<FieldDescriptorsIter<'a, D>, CastConstPtr<P>> {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = *self;

        let descriptors = descriptors.field_descriptors().into_iter();
        let inner = buffer_offsets(descriptors, capacity);
        unsafe { ErasedSoaPtrsIter::new_unchecked(inner, buffer, offset) }
    }

    #[inline]
    pub fn iter_mut(&'a mut self) -> ErasedSoaMutPtrsIter<FieldDescriptorsIter<'a, D>, P> {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = *self;

        let descriptors = descriptors.field_descriptors().into_iter();
        let inner = buffer_offsets(descriptors, capacity);
        unsafe { ErasedSoaMutPtrsIter::new_unchecked(inner, buffer, offset) }
    }
}

impl<D, P> ErasedSoaMutPtrs<D, P>
where
    D: FieldDescriptorsOwned + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    #[track_caller]
    pub unsafe fn swap<E>(&mut self, with: &mut ErasedSoaMutPtrs<E, P>)
    where
        E: FieldDescriptorsOwned + ?Sized,
    {
        let n = assert_descriptors(self.field_descriptors(), with.field_descriptors());

        let mut this = self.iter_mut();
        let mut with = with.iter_mut();
        for _ in 0..n {
            let this = unsafe { this.next_unchecked() };
            let with = unsafe { with.next_unchecked() };
            unsafe { this.swap(with) }
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_forward<'e, E>(
        &mut self,
        src: &'e ErasedSoaPtrs<E, CastConstPtr<P>>,
        count: usize,
    ) where
        E: FieldDescriptors<'e> + ?Sized,
    {
        let n = assert_descriptors(self.field_descriptors(), src.field_descriptors());

        let mut dst = self.iter_mut();
        let mut src = src.iter();
        for _ in 0..n {
            let dst = unsafe { dst.next_unchecked() };
            let src = unsafe { src.next_unchecked() };
            unsafe { dst.copy_from(src, count) }
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_backward<'e, E>(
        &mut self,
        src: &'e ErasedSoaPtrs<E, CastConstPtr<P>>,
        count: usize,
    ) where
        E: FieldDescriptors<'e> + ?Sized,
    {
        #[inline]
        fn rec<D, E, P>(
            dst_iter: &mut ErasedSoaMutPtrsIter<D, P>,
            src_iter: &mut ErasedSoaPtrsIter<E, CastConstPtr<P>>,
            n: usize,
            count: usize,
        ) where
            D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
            E: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
            P: MutSliceItemPtr,
        {
            if n == 0 {
                return;
            }
            let dst = unsafe { dst_iter.next_unchecked() };
            let src = unsafe { src_iter.next_unchecked() };

            rec(dst_iter, src_iter, n - 1, count);
            unsafe { dst.copy_from(src, count) }
        }

        let n = assert_descriptors(self.field_descriptors(), src.field_descriptors());
        let mut dst = self.iter_mut();
        let mut src = src.iter();

        rec(&mut dst, &mut src, n, count);
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping<'e, E>(
        &mut self,
        src: &'e ErasedSoaPtrs<E, CastConstPtr<P>>,
        count: usize,
    ) where
        E: FieldDescriptors<'e> + ?Sized,
    {
        let n = assert_descriptors(self.field_descriptors(), src.field_descriptors());

        let mut dst = self.iter_mut();
        let mut src = src.iter();
        for _ in 0..n {
            let dst = unsafe { dst.next_unchecked() };
            let src = unsafe { src.next_unchecked() };
            unsafe { dst.copy_from_nonoverlapping(src, count) }
        }
    }
}

impl<D, P, U> ErasedSoaMutPtrs<D, P>
where
    D: FieldDescriptorsOwned + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
{
    #[inline]
    #[track_caller]
    pub unsafe fn write<T, E>(&mut self, value: ErasedSoa<T, E, P::Ptrs>)
    where
        T: AlignedStorage<Item = U>,
        E: FieldDescriptorsOwned,
        for<'a, 'b> FieldDescriptorsOutput<'a, E>: FieldDescriptors<'b>,
    {
        let src = value.as_ptrs();
        unsafe { self.copy_from_nonoverlapping(&src, 1) };

        drop(src);
        let _ = value.into_parts();
    }
}

impl<D, P> Debug for ErasedSoaMutPtrs<D, P>
where
    D: Debug + ?Sized,
    P: MutSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            buffer,
            capacity,
            offset,
            descriptors,
            ..
        } = self;

        f.debug_struct("ErasedSoaMutPtrs")
            .field("buffer", buffer)
            .field("capacity", capacity)
            .field("offset", offset)
            .field("descriptors", &descriptors)
            .finish()
    }
}

impl<D, P> Clone for ErasedSoaMutPtrs<D, P>
where
    D: Clone,
    P: MutSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            ref descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = *self;

        let descriptors = descriptors.clone();
        unsafe { Self::new_unchecked(descriptors, buffer, capacity, offset) }
    }
}

impl<D, P> Copy for ErasedSoaMutPtrs<D, P>
where
    D: Copy,
    P: MutSliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedSoaMutPtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedPtr<CastConstPtr<P>>;
    type IntoIter = ErasedSoaPtrsIter<FieldDescriptorsIter<'a, D>, CastConstPtr<P>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P> IntoIterator for &'a mut ErasedSoaMutPtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedMutPtr<P>;
    type IntoIter = ErasedSoaMutPtrsIter<FieldDescriptorsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<D, P> IntoIterator for ErasedSoaMutPtrs<D, P>
where
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
    P: MutSliceItemPtr,
{
    type Item = ErasedMutPtr<P>;
    type IntoIter = ErasedSoaMutPtrsIter<D::IntoIter, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            buffer,
            capacity,
            offset,
            ..
        } = self;

        let descriptors = descriptors.into_iter();
        let inner = buffer_offsets(descriptors, capacity);
        unsafe { ErasedSoaMutPtrsIter::new_unchecked(inner, buffer, offset) }
    }
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedSoaMutPtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { descriptors, .. } = self;
        descriptors.field_descriptors()
    }
}

impl<D, P> CovariantFieldDescriptors for ErasedSoaMutPtrs<D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        D::upcast_field_descriptors(from)
    }
}

pub struct ErasedSoaMutPtrsIter<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    buffer: *mut [P::Item],
    offset: usize,
    inner: BufferOffsets<D>,
}

impl<D, P> ErasedSoaMutPtrsIter<D, P>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn new_unchecked(
        inner: BufferOffsets<D>,
        buffer: *mut [P::Item],
        offset: usize,
    ) -> Self {
        Self {
            buffer,
            offset,
            inner,
        }
    }
}

impl<D, P> ErasedSoaMutPtrsIter<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [P::Item] {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> *mut [P::Item] {
        let Self { buffer, .. } = *self;
        buffer
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { inner, .. } = self;
        inner.capacity()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { offset, .. } = *self;
        offset
    }

    #[inline]
    pub fn descriptors(&self) -> &D {
        let Self { inner, .. } = self;
        inner.as_inner()
    }
}

impl<D, P> ErasedSoaMutPtrsIter<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    unsafe fn field_ptr_from_buffer_offset(&self, offset: BufferOffset) -> ErasedMutPtr<P> {
        let BufferOffset { desc, offset, .. } = offset;
        let index = bytes_to_items::<P::Item>(offset);

        let Self { buffer, offset, .. } = *self;
        let ptr = unsafe { P::from_slice(buffer, index) };
        unsafe { ErasedMutPtr::from_parts(desc.layout(), ptr).add(offset) }
    }
}

impl<D, P> ErasedSoaMutPtrsIter<D, P>
where
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn next_unchecked(&mut self) -> ErasedMutPtr<P> {
        let Self { inner, .. } = self;

        let offset = unsafe { inner.next_unchecked() };
        unsafe { self.field_ptr_from_buffer_offset(offset) }
    }
}

impl<'a, D, P> ErasedSoaMutPtrsIter<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub(super) fn entries(&'a self) -> ErasedSoaMutPtrsIter<FieldDescriptorsIter<'a, D>, P> {
        let Self {
            ref inner,
            buffer,
            offset,
            ..
        } = *self;

        let layout = inner.layout();
        let capacity = inner.capacity();
        let fields = inner.as_inner().field_descriptors().into_iter();

        let inner = unsafe { BufferOffsets::from_parts(layout, capacity, fields) };
        unsafe { ErasedSoaMutPtrsIter::new_unchecked(inner, buffer, offset) }
    }
}

impl<D, P> Debug for ErasedSoaMutPtrsIter<D, P>
where
    D: FieldDescriptorsOwned + ?Sized,
    P: MutSliceItemPtr + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = self.entries();
        f.debug_list().entries(entries).finish()
    }
}

impl<D, P> Clone for ErasedSoaMutPtrsIter<D, P>
where
    D: Clone,
    P: MutSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            ref inner,
            buffer,
            offset,
            ..
        } = *self;

        let inner = inner.clone();
        unsafe { Self::new_unchecked(inner, buffer, offset) }
    }
}

impl<D, P> Iterator for ErasedSoaMutPtrsIter<D, P>
where
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedMutPtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner, .. } = self;

        let offset = inner.next()?;
        let offset = unsafe { offset.unwrap_unchecked() };
        let item = unsafe { self.field_ptr_from_buffer_offset(offset) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner, .. } = self;
        inner.size_hint()
    }
}

impl<D, P> ExactSizeIterator for ErasedSoaMutPtrsIter<D, P>
where
    D: ExactSizeIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }
}

impl<D, P> FusedIterator for ErasedSoaMutPtrsIter<D, P>
where
    D: FusedIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: MutSliceItemPtr,
{
}

impl<'a, D, P> FieldDescriptors<'a> for ErasedSoaMutPtrsIter<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { inner, .. } = self;
        inner.as_inner().field_descriptors()
    }
}

impl<D, P> CovariantFieldDescriptors for ErasedSoaMutPtrsIter<D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        D::upcast_field_descriptors(from)
    }
}
