use core::{
    alloc::{Layout, LayoutError},
    error::Error,
    fmt::{self, Debug, Display},
    iter::FusedIterator,
    marker::PhantomData,
    num::NonZeroUsize,
    ptr, slice,
};

use itertools::{EitherOrBoth::Both, Itertools};

use crate::{
    CovariantFieldLayouts, ErasedSoaMutPtrs, ErasedSoaMutRefs, ErasedSoaMutRefsIter, ErasedSoaPtrs,
    ErasedSoaRefs, ErasedSoaRefsIter,
    assert::check_downcast,
    data::{Erased, ErasedMutRef, ErasedRef, error::FromLayoutDataError, try_clone_from_slice},
    error::{
        DowncastError, FromFieldsLayoutsError, FromLayoutsValueError, FromLayoutsValueErrorKind,
        FromStorageFieldsLayoutsError, FromStorageValueError, FromValueError, FromValueErrorKind,
        InsufficientAlignError, IterOrFieldLenMismatchError, LenMismatchError,
        check_from_layouts_value, check_from_storage_value, check_from_value, check_layout,
        check_len, check_sufficient_align,
    },
    layout::bytes_to_items,
    ptr::slice::SliceItemPtrs,
    soa::{
        field::{
            BufferOffset, BufferOffsets, FieldLayouts, FieldLayoutsItem, FieldLayoutsIter,
            FieldLayoutsOutput, FieldLayoutsOwned, RawBufferOffsets, buffer_offsets,
        },
        layout::WithLayout,
        traits::{
            AllocSoa, AllocSoaContext, ReadSoaContext, Refs, RefsMut, Soa, SoaRead, SoaWrite,
            WriteSoaContext,
        },
    },
    storage::{AlignedStorage, AlignedStorageFromLayout},
};

#[cfg(feature = "alloc")]
pub type BoxedErasedSoa<P, I = Layout> =
    ErasedSoa<crate::storage::BoxedAlignedUninitStorage, alloc::boxed::Box<[I]>, P>;

pub struct ErasedSoa<T, D, P>
where
    D: ?Sized,
{
    phantom: PhantomData<P>,
    storage: T,
    layouts: D,
}

impl<T, D, P> ErasedSoa<T, D, P> {
    #[inline]
    pub unsafe fn from_parts(storage: T, layouts: D) -> Self {
        Self {
            phantom: PhantomData,
            storage,
            layouts,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (T, D) {
        let Self {
            storage, layouts, ..
        } = self;
        (storage, layouts)
    }
}

impl<T, D, P> ErasedSoa<T, D, P>
where
    D: ?Sized,
{
    #[inline]
    pub fn layouts(&self) -> &D {
        let Self { layouts, .. } = self;
        layouts
    }
}

impl<T, D, P> ErasedSoa<T, D, P>
where
    T: AlignedStorage,
    D: ?Sized,
{
    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { storage, .. } = self;
        storage.layout()
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T::Item {
        let Self { storage, .. } = self;
        storage.as_ptr().cast()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T::Item {
        let Self { storage, .. } = self;
        storage.as_mut_ptr().cast()
    }

    #[inline]
    pub fn as_buffer(&self) -> &[T::Item] {
        let Self { storage, .. } = self;
        storage.as_slice()
    }

    #[inline]
    pub fn as_mut_buffer(&mut self) -> &mut [T::Item] {
        let Self { storage, .. } = self;
        storage.as_mut_slice()
    }
}

impl<T, D, P> ErasedSoa<T, D, P>
where
    T: AlignedStorage<Item: Clone>,
    D: FieldLayoutsOwned,
{
    #[inline]
    pub fn try_from_storage_fields_layouts<I, F>(
        mut storage: T,
        fields: I,
        layouts: D,
    ) -> Result<Self, FromStorageFieldsLayoutsError>
    where
        I: IntoIterator<Item = F>,
        F: AsRef<[T::Item]>,
    {
        let mut offsets = buffer_offsets(layouts.field_layouts(), 1);
        offsets.by_ref().try_for_each(|offset| {
            check_sufficient_align(offset?.desc.layout(), Layout::new::<T::Item>())
                .map_err(FromStorageFieldsLayoutsError::from)
        })?;

        let layout = storage.layout();
        let expected_layout = offsets.into_buffer_layout();
        check_layout(layout, expected_layout)?;

        write_clone_of_fields(storage.as_mut_slice(), fields, layouts.field_layouts())?;

        let me = unsafe { Self::from_parts(storage, layouts) };
        Ok(me)
    }
}

impl<T, D, P> ErasedSoa<T, D, P>
where
    T: AlignedStorage,
    D: FieldLayoutsOwned,
{
    #[inline]
    pub unsafe fn downcast<'a, V, R>(
        self,
        context: &'a V::Context,
    ) -> Result<R, DowncastError<Self>>
    where
        V: AllocSoa + SoaRead<'a, R> + ?Sized,
    {
        let Self {
            ref layouts,
            ref storage,
            ..
        } = self;

        let layouts = layouts.field_layouts();
        if let Err(error) = check_downcast(layouts, context.field_layouts(), 1) {
            return Err(DowncastError::new(self, error));
        }

        let layout = match context.buffer_layout(1) {
            Ok(layout) => layout,
            Err(error) => return Err(DowncastError::new(self, error.into())),
        };
        if let Err(error) = check_len(layout.size(), storage.layout().size()) {
            return Err(DowncastError::new(self, error.into()));
        }

        let Self { storage, .. } = self;
        let value = unsafe {
            let src = context.ptrs_from_buffer(storage.as_ptr().cast(), 1);
            context.read(src)
        };
        Ok(value)
    }
}

impl<'a, T, D, P> ErasedSoa<T, D, P>
where
    T: AlignedStorage,
    D: FieldLayouts<'a> + ?Sized,
    P: SliceItemPtrs<Item = T::Item>,
{
    #[inline]
    pub fn as_ptrs(&'a self) -> ErasedSoaPtrs<D::Output, P::Const> {
        let (ptrs, _) = self.as_ptrs_with_layouts();
        ptrs
    }

    #[inline]
    pub fn as_ptrs_with_layouts(&'a self) -> (ErasedSoaPtrs<D::Output, P::Const>, &'a D) {
        let Self {
            ref storage,
            ref layouts,
            ..
        } = *self;

        let ptrs = {
            let layouts = layouts.field_layouts();
            let buffer = storage.as_slice();
            unsafe { ErasedSoaPtrs::new_unchecked(layouts, buffer, 1, 0) }
        };
        (ptrs, layouts)
    }

    #[inline]
    pub fn as_mut_ptrs(&'a mut self) -> ErasedSoaMutPtrs<D::Output, P::Mut> {
        let (ptrs, _) = self.as_mut_ptrs_with_layouts();
        ptrs
    }

    #[inline]
    pub fn as_mut_ptrs_with_layouts(&'a mut self) -> (ErasedSoaMutPtrs<D::Output, P::Mut>, &'a D) {
        let Self {
            ref mut storage,
            ref layouts,
            ..
        } = *self;

        let ptrs = {
            let layouts = layouts.field_layouts();
            let buffer = storage.as_mut_slice();
            unsafe { ErasedSoaMutPtrs::new_unchecked(layouts, buffer, 1, 0) }
        };
        (ptrs, layouts)
    }

    #[inline]
    pub fn as_refs(&'a self) -> ErasedSoaRefs<'a, D::Output, P::Const> {
        let (refs, _) = self.as_refs_with_layouts();
        refs
    }

    #[inline]
    pub fn as_refs_with_layouts(&'a self) -> (ErasedSoaRefs<'a, D::Output, P::Const>, &'a D) {
        let (ptrs, layouts) = self.as_ptrs_with_layouts();
        let refs = unsafe { ptrs.as_ref_unchecked() };
        (refs, layouts)
    }

    #[inline]
    pub fn as_mut_refs(&'a mut self) -> ErasedSoaMutRefs<'a, D::Output, P::Mut> {
        let (refs, _) = self.as_mut_refs_with_layouts();
        refs
    }

    #[inline]
    pub fn as_mut_refs_with_layouts(
        &'a mut self,
    ) -> (ErasedSoaMutRefs<'a, D::Output, P::Mut>, &'a D) {
        let (ptrs, layouts) = self.as_mut_ptrs_with_layouts();
        let refs = unsafe { ptrs.as_mut_unchecked() };
        (refs, layouts)
    }

    #[inline]
    pub fn iter(&'a self) -> ErasedSoaRefsIter<'a, FieldLayoutsIter<'a, D>, P::Const> {
        self.as_refs().into_iter()
    }

    #[inline]
    pub fn iter_mut(&'a mut self) -> ErasedSoaMutRefsIter<'a, FieldLayoutsIter<'a, D>, P::Mut> {
        self.as_mut_refs().into_iter()
    }
}

impl<'a, T, D, P> ErasedSoa<T, D, P>
where
    T: AlignedStorage,
    D: FieldLayouts<'a, Output: FieldLayoutsOwned> + ?Sized,
    P: SliceItemPtrs<Item = T::Item>,
{
    #[inline]
    pub unsafe fn downcast_ref<'ctx, V>(
        &'a self,
        context: &'ctx V::Context,
    ) -> Result<Refs<'ctx, 'a, V>, DowncastError<&'a Self>>
    where
        V: AllocSoa + Soa<'a> + ?Sized,
    {
        let into_self = |_| self;

        let result = unsafe { self.as_refs().downcast::<V>(context) };
        let refs = result.map_err(|err| err.map_value(into_self))?;

        Ok(refs)
    }

    #[inline]
    pub unsafe fn downcast_mut<'ctx, V>(
        &'a mut self,
        context: &'ctx V::Context,
    ) -> Result<RefsMut<'ctx, 'a, V>, DowncastError<&'a mut Self>>
    where
        V: AllocSoa + Soa<'a> + ?Sized,
    {
        let ptr = ptr::from_mut(self);
        let into_self = |_| unsafe { ptr.as_mut_unchecked() };

        let result = unsafe { self.as_mut_refs().downcast::<V>(context) };
        let refs = result.map_err(|err| err.map_value(into_self))?;

        Ok(refs)
    }
}

impl<T, D, P> ErasedSoa<T, D, P>
where
    T: AlignedStorageFromLayout<Item: Clone>,
    D: FieldLayoutsOwned,
{
    #[inline]
    pub fn try_from_fields_layouts<I, F>(
        fields: I,
        layouts: D,
    ) -> Result<Self, FromFieldsLayoutsError<T::Error>>
    where
        I: IntoIterator<Item = F>,
        F: AsRef<[T::Item]>,
    {
        let mut offsets = buffer_offsets(layouts.field_layouts(), 1);
        offsets.by_ref().try_for_each(|offset| {
            check_sufficient_align(offset?.desc.layout(), Layout::new::<T::Item>())
                .map_err(FromFieldsLayoutsError::from)
        })?;

        let layout = offsets.into_buffer_layout();
        let mut storage = T::from_layout(layout).map_err(FromFieldsLayoutsError::FromLayout)?;

        write_clone_of_fields(storage.as_mut_slice(), fields, layouts.field_layouts())?;

        let me = unsafe { Self::from_parts(storage, layouts) };
        Ok(me)
    }
}

impl<T, D, P> ErasedSoa<T, D, P>
where
    T: AlignedStorageFromLayout<Item: Clone>,
{
    #[inline]
    pub fn try_from_fields_with_layouts<I, F, E>(
        fields_with_layouts: I,
    ) -> Result<Self, FromFieldsLayoutsError<T::Error>>
    where
        I: IntoIterator<Item = (F, E)>,
        F: AsRef<[T::Item]>,
        E: WithLayout,
        D: FromIterator<E>,
    {
        let (storage, layouts) = storage_from_fields_with_layouts(fields_with_layouts)?;
        let me = unsafe { Self::from_parts(storage, layouts) };
        Ok(me)
    }
}

impl<T, D, P> ErasedSoa<T, D, P>
where
    T: AlignedStorage,
{
    #[inline]
    pub fn try_from_storage_value<'a, V, W>(
        mut storage: T,
        context: &'a V::Context,
        value: W,
    ) -> Result<Self, FromStorageValueError<W>>
    where
        V: AllocSoa + SoaWrite<W> + ?Sized,
        D: FromIterator<FieldLayoutsItem<'a, V::Context, V>>,
    {
        let check = || {
            context.field_layouts().into_iter().try_for_each(|actual| {
                check_sufficient_align(actual.layout(), Layout::new::<T::Item>())
            })?;

            let expected_layout = context.buffer_layout(1)?;
            check_layout(storage.layout(), expected_layout).map_err(From::from)
        };
        let (value, ()) = check_from_storage_value(check, value)?;

        let layouts = context.field_layouts().into_iter().collect();
        unsafe {
            let dst = context.ptrs_from_buffer_mut(storage.as_mut_ptr().cast(), 1);
            context.write(dst, value);
        }

        let me = unsafe { Self::from_parts(storage, layouts) };
        Ok(me)
    }
}

impl<T, D, P> ErasedSoa<T, D, P>
where
    T: AlignedStorageFromLayout,
    D: FieldLayoutsOwned,
{
    #[inline]
    pub fn try_from_layouts_value<V, W>(
        layouts: D,
        context: &V::Context,
        value: W,
    ) -> Result<Self, FromLayoutsValueError<W, T::Error>>
    where
        V: AllocSoa + SoaWrite<W> + ?Sized,
    {
        let f = || {
            let mut offsets = buffer_offsets(layouts.field_layouts(), 1);
            for (field_index, item) in offsets
                .by_ref()
                .zip_longest(context.field_layouts())
                .enumerate()
            {
                let Both(offset, item) = item else {
                    let count = field_index + offsets.count();
                    let error = unsafe { LenMismatchError::new_unchecked(count, field_index) };
                    return Err(error.into());
                };

                let actual = offset?.desc.layout();
                check_layout(actual, item.layout())?;
                check_sufficient_align(actual, Layout::new::<T::Item>())?;
            }

            let layout = offsets.into_buffer_layout();
            let expected_layout = context.buffer_layout(1)?;
            check_layout(layout, expected_layout)?;

            T::from_layout(layout).map_err(FromLayoutsValueErrorKind::FromLayout)
        };
        let (value, mut storage) = check_from_layouts_value(f, value)?;

        unsafe {
            let dst = context.ptrs_from_buffer_mut(storage.as_mut_ptr().cast(), 1);
            context.write(dst, value);
        }

        let me = unsafe { Self::from_parts(storage, layouts) };
        Ok(me)
    }
}

impl<T, D, P> ErasedSoa<T, D, P>
where
    T: AlignedStorageFromLayout,
{
    #[inline]
    pub fn try_from<'a, V, W>(
        context: &'a V::Context,
        value: W,
    ) -> Result<Self, FromValueError<W, T::Error>>
    where
        V: AllocSoa + SoaWrite<W> + ?Sized,
        D: FromIterator<FieldLayoutsItem<'a, V::Context, V>>,
    {
        let f = || {
            context.field_layouts().into_iter().try_for_each(|actual| {
                check_sufficient_align(actual.layout(), Layout::new::<T::Item>())
            })?;

            let layout = context.buffer_layout(1)?;
            T::from_layout(layout).map_err(FromValueErrorKind::FromLayout)
        };
        let (value, mut storage) = check_from_value(f, value)?;

        let layouts = context.field_layouts().into_iter().collect();
        unsafe {
            let dst = context.ptrs_from_buffer_mut(storage.as_mut_ptr().cast(), 1);
            context.write(dst, value);
        }

        let me = unsafe { Self::from_parts(storage, layouts) };
        Ok(me)
    }
}

impl<T, D, P> ErasedSoa<T, D, P>
where
    T: AlignedStorage<Item: Clone>,
    D: IntoIterator<Item: WithLayout>,
    P: SliceItemPtrs<Item = T::Item>,
{
    #[inline]
    pub fn into_fields<F>(self) -> ErasedSoaIntoFields<T, D::IntoIter, F, P>
    where
        F: AlignedStorageFromLayout<Item = T::Item>,
    {
        let (storage, layouts) = self.into_parts();
        let offsets = buffer_offsets(layouts, 1);
        ErasedSoaIntoFields::new(storage, offsets)
    }
}

impl<T, D, P> Debug for ErasedSoa<T, D, P>
where
    T: AlignedStorage<Item: Debug>,
    D: FieldLayoutsOwned + ?Sized,
    P: SliceItemPtrs<Item = T::Item>,
    for<'a> FieldLayoutsIter<'a, D>: FieldLayoutsOwned,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields = &self.into_iter();
        f.debug_struct("ErasedSoa").field("fields", fields).finish()
    }
}

impl<T, D, P> AsRef<[T::Item]> for ErasedSoa<T, D, P>
where
    T: AlignedStorage,
    D: ?Sized,
{
    #[inline]
    fn as_ref(&self) -> &[T::Item] {
        self.as_buffer()
    }
}

impl<T, D, P> AsMut<[T::Item]> for ErasedSoa<T, D, P>
where
    T: AlignedStorage,
    D: ?Sized,
{
    #[inline]
    fn as_mut(&mut self) -> &mut [T::Item] {
        self.as_mut_buffer()
    }
}

impl<'a, T, D, P> IntoIterator for &'a ErasedSoa<T, D, P>
where
    T: AlignedStorage,
    D: FieldLayouts<'a> + ?Sized,
    P: SliceItemPtrs<Item = T::Item>,
{
    type Item = ErasedRef<'a, P::Const>;
    type IntoIter = ErasedSoaRefsIter<'a, FieldLayoutsIter<'a, D>, P::Const>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, D, P> IntoIterator for &'a mut ErasedSoa<T, D, P>
where
    T: AlignedStorage,
    D: FieldLayouts<'a> + ?Sized,
    P: SliceItemPtrs<Item = T::Item>,
{
    type Item = ErasedMutRef<'a, P::Mut>;
    type IntoIter = ErasedSoaMutRefsIter<'a, FieldLayoutsIter<'a, D>, P::Mut>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, D, P> IntoIterator for ErasedSoa<T, D, P>
where
    T: AlignedStorageFromLayout<Item: Clone>,
    D: IntoIterator<Item: WithLayout>,
    P: SliceItemPtrs<Item = T::Item>,
{
    type Item = Result<Erased<T, P>, FromLayoutDataError<T::Error>>;
    type IntoIter = ErasedSoaIntoFields<T, D::IntoIter, T, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.into_fields()
    }
}

impl<'a, T, D, P> FieldLayouts<'a> for ErasedSoa<T, D, P>
where
    D: FieldLayouts<'a> + ?Sized,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { layouts, .. } = self;
        layouts.field_layouts()
    }
}

impl<T, D, P> CovariantFieldLayouts for ErasedSoa<T, D, P>
where
    D: CovariantFieldLayouts + ?Sized,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}

pub struct ErasedSoaIntoFields<T, I, F, P>
where
    I: ?Sized,
{
    phantom: PhantomData<fn() -> (F, P)>,
    storage: T,
    offsets: BufferOffsets<I>,
}

impl<T, I, F, P> ErasedSoaIntoFields<T, I, F, P> {
    fn new(storage: T, offsets: BufferOffsets<I>) -> Self {
        Self {
            phantom: PhantomData,
            storage,
            offsets,
        }
    }
}

impl<T, I, F, P> ErasedSoaIntoFields<T, I, F, P>
where
    I: ?Sized,
{
    #[inline]
    pub fn layouts(&self) -> &I {
        let Self { offsets, .. } = self;
        offsets.as_inner()
    }
}

impl<T, I, F, P> Debug for ErasedSoaIntoFields<T, I, F, P>
where
    T: Debug,
    I: Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            storage, offsets, ..
        } = self;

        f.debug_struct("ErasedSoaIntoFields")
            .field("storage", storage)
            .field("offsets", &offsets)
            .finish()
    }
}

impl<T, I, F, P> Clone for ErasedSoaIntoFields<T, I, F, P>
where
    T: Clone,
    I: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self {
            storage, offsets, ..
        } = self;

        Self::new(storage.clone(), offsets.clone())
    }
}

impl<T, I, F, P> Iterator for ErasedSoaIntoFields<T, I, F, P>
where
    T: AlignedStorage<Item: Clone>,
    I: Iterator<Item: WithLayout> + ?Sized,
    F: AlignedStorageFromLayout<Item = T::Item>,
    P: SliceItemPtrs<Item = T::Item>,
{
    type Item = Result<Erased<F, P>, FromLayoutDataError<F::Error>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref storage,
            ref mut offsets,
            ..
        } = *self;

        let BufferOffset { desc, offset } = unsafe { offsets.next()?.unwrap_unchecked() };
        let layout = desc.layout();

        let offset = bytes_to_items::<T::Item>(offset);
        let len = bytes_to_items::<T::Item>(layout.size());
        let data = unsafe { storage.as_ptr().add(offset).cast() };
        let data = unsafe { slice::from_raw_parts(data, len) };

        let item = Erased::try_from_layout_data(layout, data);
        Some(item)
    }
}

impl<T, I, F, P> ExactSizeIterator for ErasedSoaIntoFields<T, I, F, P>
where
    T: AlignedStorage<Item: Clone>,
    I: ExactSizeIterator<Item: WithLayout> + ?Sized,
    F: AlignedStorageFromLayout<Item = T::Item>,
    P: SliceItemPtrs<Item = T::Item>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { offsets, .. } = self;
        offsets.len()
    }
}

impl<T, I, F, P> FusedIterator for ErasedSoaIntoFields<T, I, F, P>
where
    T: AlignedStorage<Item: Clone>,
    I: FusedIterator<Item: WithLayout> + ?Sized,
    F: AlignedStorageFromLayout<Item = T::Item>,
    P: SliceItemPtrs<Item = T::Item>,
{
}

impl<'a, T, I, F, P> FieldLayouts<'a> for ErasedSoaIntoFields<T, I, F, P>
where
    I: FieldLayouts<'a> + ?Sized,
{
    type Output = I::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        self.layouts().field_layouts()
    }
}

impl<T, I, F, P> CovariantFieldLayouts for ErasedSoaIntoFields<T, I, F, P>
where
    I: CovariantFieldLayouts + ?Sized,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        I::upcast_field_layouts(from)
    }
}

#[derive(Debug, Clone)]
enum WriteCopyOfFieldsError {
    LenMismatch(IterOrFieldLenMismatchError),
    InvalidLayout(LayoutError),
    InsufficientAlign(InsufficientAlignError),
}

impl From<IterOrFieldLenMismatchError> for WriteCopyOfFieldsError {
    #[inline]
    fn from(error: IterOrFieldLenMismatchError) -> Self {
        Self::LenMismatch(error)
    }
}

impl From<LayoutError> for WriteCopyOfFieldsError {
    #[inline]
    fn from(error: LayoutError) -> Self {
        Self::InvalidLayout(error)
    }
}

impl From<InsufficientAlignError> for WriteCopyOfFieldsError {
    #[inline]
    fn from(error: InsufficientAlignError) -> Self {
        Self::InsufficientAlign(error)
    }
}

impl Display for WriteCopyOfFieldsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LenMismatch(error) => Display::fmt(error, f),
            Self::InvalidLayout(error) => Display::fmt(error, f),
            Self::InsufficientAlign(error) => Display::fmt(error, f),
        }
    }
}

impl Error for WriteCopyOfFieldsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LenMismatch(error) => Some(error),
            Self::InvalidLayout(error) => Some(error),
            Self::InsufficientAlign(error) => Some(error),
        }
    }
}

impl From<WriteCopyOfFieldsError> for FromStorageFieldsLayoutsError {
    #[inline]
    fn from(error: WriteCopyOfFieldsError) -> Self {
        match error {
            WriteCopyOfFieldsError::LenMismatch(error) => Self::LenMismatch(error),
            WriteCopyOfFieldsError::InvalidLayout(error) => Self::InvalidLayout(error),
            WriteCopyOfFieldsError::InsufficientAlign(error) => Self::InsufficientAlign(error),
        }
    }
}

impl<T> From<WriteCopyOfFieldsError> for FromFieldsLayoutsError<T> {
    #[inline]
    fn from(error: WriteCopyOfFieldsError) -> Self {
        match error {
            WriteCopyOfFieldsError::LenMismatch(error) => Self::LenMismatch(error),
            WriteCopyOfFieldsError::InvalidLayout(error) => Self::InvalidLayout(error),
            WriteCopyOfFieldsError::InsufficientAlign(error) => Self::InsufficientAlign(error),
        }
    }
}

fn write_clone_of_fields<T, F, D>(
    dst: &mut [T],
    fields: F,
    layouts: D,
) -> Result<(), WriteCopyOfFieldsError>
where
    T: Clone,
    F: IntoIterator<Item: AsRef<[T]>>,
    D: IntoIterator<Item: WithLayout>,
{
    use IterOrFieldLenMismatchError::{FieldLenMismatch, IterLenMismatch};

    let mut layouts = layouts.into_iter();
    for (field_index, item) in buffer_offsets(&mut layouts, 1)
        .zip_longest(fields)
        .enumerate()
    {
        let Both(offset, src) = item else {
            let layouts_count = field_index + layouts.count();
            let error = unsafe { LenMismatchError::new_unchecked(layouts_count, field_index) };
            let error = IterLenMismatch(error).into();
            return Err(error);
        };

        let BufferOffset { desc, offset } = offset?;
        let layout = desc.layout();
        check_sufficient_align(layout, Layout::new::<T>())?;

        let offset = bytes_to_items::<T>(offset);
        let len = bytes_to_items::<T>(layout.size());
        let dst = &mut dst[offset..offset + len];
        try_clone_from_slice(dst, src.as_ref())
            .map_err(|error| FieldLenMismatch { error, field_index })?;
    }
    Ok(())
}

fn storage_from_fields_with_layouts<I, T, F, E, D>(
    fields_with_layouts: I,
) -> Result<(T, D), FromFieldsLayoutsError<T::Error>>
where
    T: AlignedStorageFromLayout<Item: Clone>,
    D: FromIterator<E>,
    I: IntoIterator<Item = (F, E)>,
    F: AsRef<[T::Item]>,
    E: WithLayout,
{
    use FromFieldsLayoutsError::FromLayout;
    use IterOrFieldLenMismatchError::FieldLenMismatch;

    let mut storage = T::from_layout(Layout::new::<()>()).map_err(FromLayout)?;
    let layouts = fields_with_layouts
        .into_iter()
        .enumerate()
        .map(|(field_index, (src, item))| {
            let layout = item.layout();
            check_sufficient_align(layout, Layout::new::<T::Item>())?;

            let mut state = RawBufferOffsets::from_parts(storage.layout(), 1, NonZeroUsize::MIN);
            let offset = state.next(layout)?;

            storage
                .set_layout(state.buffer_layout())
                .map_err(FromLayout)?;

            let offset = bytes_to_items::<T::Item>(offset);
            let len = bytes_to_items::<T::Item>(layout.size());
            let dst = &mut storage.as_mut_slice()[offset..offset + len];
            try_clone_from_slice(dst, src.as_ref())
                .map_err(|error| FieldLenMismatch { error, field_index })?;

            Ok(item)
        })
        .collect::<Result<_, FromFieldsLayoutsError<_>>>()?;

    Ok((storage, layouts))
}
