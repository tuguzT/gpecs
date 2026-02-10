use core::{
    alloc::Layout,
    fmt::{self, Debug},
    iter::FusedIterator,
    marker::PhantomData,
    mem::MaybeUninit,
    ptr,
};

use crate::{
    erased::{
        CovariantFieldDescriptors, ErasedSoaPtrs, ErasedSoaPtrsIter, ErasedSoaRefs,
        ErasedSoaRefsMut,
        assert::{assert_descriptors, check_into_value},
        dangling::{Dangling, dangling},
        error::{ErasedSoaIntoValueError, ErasedSoaPtrsError, check_offset},
    },
    error::{
        InsufficientAlignError, check_ptr_align, check_sufficient_align, check_sufficient_len,
    },
    field::{ErasedFieldMutPtr, ErasedFieldPtr},
    slice_item_ptr::{CastConstPtr, MutSliceItemPtr},
    soa::{
        field::{
            BufferOffset, BufferOffsets, FieldDescriptor, FieldDescriptors, FieldDescriptorsIter,
            FieldDescriptorsOwned, buffer_offsets,
        },
        traits::{AllocSoa, AllocSoaContext, MutPtrs, RawSoaContext},
    },
    storage::AddressableUnit,
};

pub struct ErasedSoaMutPtrs<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    phantom: PhantomData<P>,
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
            phantom: PhantomData,
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
    pub unsafe fn deref_mut<'a>(self) -> ErasedSoaRefsMut<'a, D, P> {
        unsafe { ErasedSoaRefsMut::from_mut_ptrs(self) }
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
    ) -> Result<Self, ErasedSoaPtrsError> {
        let mut offsets = buffer_offsets(descriptors.field_descriptors(), capacity);
        offsets.by_ref().try_for_each(|offset| {
            check_sufficient_align(offset?.desc.layout(), Layout::new::<P::Item>())
                .map_err(ErasedSoaPtrsError::from)
        })?;

        let layout = offsets.into_layout();
        check_sufficient_len(buffer.len() * size_of::<P::Item>(), layout.size())?;
        check_ptr_align(buffer.cast(), layout)?;
        check_offset(offset, capacity)?;

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
    P: MutSliceItemPtr<Item = MaybeUninit<u8>>,
{
    #[inline]
    pub unsafe fn try_into<T>(
        self,
        context: &T::Context,
    ) -> Result<MutPtrs<'_, T>, ErasedSoaIntoValueError<Self>>
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

        let result = check_into_value(descriptors.field_descriptors(), context.field_descriptors());
        if let Err(error) = result {
            return Err(ErasedSoaIntoValueError::new(self, error));
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

impl<'a, D, P, U> ErasedSoaMutPtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
    U: AddressableUnit,
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

impl<D, P, U> ErasedSoaMutPtrs<D, P>
where
    D: FieldDescriptorsOwned + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
    U: AddressableUnit,
{
    #[inline]
    #[track_caller]
    pub unsafe fn swap<E>(&mut self, with: &mut ErasedSoaMutPtrs<E, P>)
    where
        E: FieldDescriptorsOwned + ?Sized,
    {
        assert_descriptors(self.field_descriptors(), with.field_descriptors());

        // TODO: rewrite this loop without zip
        for (this, with) in itertools::zip_eq(self, with) {
            unsafe { this.swap(with) }
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_forward<'e, E>(
        &mut self,
        from: &'e ErasedSoaPtrs<E, CastConstPtr<P>>,
        count: usize,
    ) where
        E: FieldDescriptors<'e> + ?Sized,
    {
        assert_descriptors(self.field_descriptors(), from.field_descriptors());

        for (this, from) in itertools::zip_eq(self, from) {
            unsafe { this.copy_from(from, count) }
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_backward<'e, E>(
        &mut self,
        from: &'e ErasedSoaPtrs<E, CastConstPtr<P>>,
        count: usize,
    ) where
        E: FieldDescriptors<'e> + ?Sized,
    {
        assert_descriptors(self.field_descriptors(), from.field_descriptors());

        #[inline]
        #[expect(clippy::items_after_statements)]
        fn rec<I, P, U>(iter: I, count: usize)
        where
            I: IntoIterator<Item = (ErasedFieldMutPtr<P>, ErasedFieldPtr<CastConstPtr<P>>)>,
            P: MutSliceItemPtr<Item = MaybeUninit<U>>,
            U: AddressableUnit,
        {
            let mut iter = iter.into_iter();
            let Some((to, from)) = iter.next() else {
                return;
            };

            rec(iter, count);
            unsafe { to.copy_from(from, count) }
        }

        rec(itertools::zip_eq(self, from), count);
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping<'e, E>(
        &mut self,
        from: &'e ErasedSoaPtrs<E, CastConstPtr<P>>,
        count: usize,
    ) where
        E: FieldDescriptors<'e> + ?Sized,
    {
        assert_descriptors(self.field_descriptors(), from.field_descriptors());

        for (this, from) in itertools::zip_eq(self, from) {
            unsafe { this.copy_from_nonoverlapping(from, count) }
        }
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

impl<'a, D, P, U> IntoIterator for &'a ErasedSoaMutPtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
    U: AddressableUnit,
{
    type Item = ErasedFieldPtr<CastConstPtr<P>>;
    type IntoIter = ErasedSoaPtrsIter<FieldDescriptorsIter<'a, D>, CastConstPtr<P>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P, U> IntoIterator for &'a mut ErasedSoaMutPtrs<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
    U: AddressableUnit,
{
    type Item = ErasedFieldMutPtr<P>;
    type IntoIter = ErasedSoaMutPtrsIter<FieldDescriptorsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<D, P, U> IntoIterator for ErasedSoaMutPtrs<D, P>
where
    D: IntoIterator<Item: AsRef<FieldDescriptor>>,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
    U: AddressableUnit,
{
    type Item = ErasedFieldMutPtr<P>;
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
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        D::upcast_field_descriptors(from)
    }
}

pub struct ErasedSoaMutPtrsIter<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    phantom: PhantomData<P>,
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
            phantom: PhantomData,
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
}

impl<'a, D, P, U> ErasedSoaMutPtrsIter<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
    U: AddressableUnit,
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

impl<D, P, U> Debug for ErasedSoaMutPtrsIter<D, P>
where
    D: FieldDescriptorsOwned + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>> + Debug,
    U: AddressableUnit,
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

impl<D, P, U> Iterator for ErasedSoaMutPtrsIter<D, P>
where
    D: Iterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
    U: AddressableUnit,
{
    type Item = ErasedFieldMutPtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref mut inner,
            buffer,
            offset,
            ..
        } = *self;

        let field_ptr = {
            let BufferOffset { desc, offset, .. } = unsafe { inner.next()?.unwrap_unchecked() };
            let index = offset.div_ceil(size_of::<U>());
            let ptr = unsafe { P::from_slice(buffer, index) };
            unsafe { ErasedFieldMutPtr::from_parts(desc, ptr) }
        };

        let item = unsafe { field_ptr.add(offset) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner, .. } = self;
        inner.size_hint()
    }
}

impl<D, P, U> ExactSizeIterator for ErasedSoaMutPtrsIter<D, P>
where
    D: ExactSizeIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
    U: AddressableUnit,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner, .. } = self;
        inner.len()
    }
}

impl<D, P, U> FusedIterator for ErasedSoaMutPtrsIter<D, P>
where
    D: FusedIterator<Item: AsRef<FieldDescriptor>> + ?Sized,
    P: MutSliceItemPtr<Item = MaybeUninit<U>>,
    U: AddressableUnit,
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
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        D::upcast_field_descriptors(from)
    }
}
