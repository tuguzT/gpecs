use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_component::{
    erased::ErasedComponentSlice,
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType, WithComponentId},
    },
};
use gpecs_soa_erased::{
    BufferOffsetsFrom, BufferOffsetsFromSelf, BufferOffsetsOf, CovariantFieldLayouts,
    ErasedSoaSlices, ErasedSoaSlicesIter,
    ptr::slice::ConstSliceItemPtr,
    soa::{
        field::{FieldLayouts, FieldLayoutsItem, FieldLayoutsOutput, FieldLayoutsOwned},
        traits::SoaContext,
    },
};

use crate::{
    bundle::{
        Bundle, BundleSlices,
        erased::{
            ErasedBundleSlicePtrs,
            error::DowncastError,
            traits::{ErasedArchetypeIterator, ErasedArchetypeKind, IntoErasedArchetypeIterator},
        },
    },
    erased::ErasedArchetypeView,
};

pub struct ErasedBundleSlices<'a, D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    inner: ErasedSoaSlices<'a, D, P>,
}

impl<'a, D, P> ErasedBundleSlices<'a, D, P>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_inner(inner: ErasedSoaSlices<'a, D, P>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundleSlicePtrs<D, P>) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { inner.as_ref_unchecked() };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub fn into_inner(self) -> ErasedSoaSlices<'a, D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedBundleSlicePtrs<D, P> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundleSlicePtrs::from_inner(inner) }
    }
}

impl<D, P> ErasedBundleSlices<'_, D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> &[P::Item] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { inner } = self;
        inner.capacity()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { inner } = self;
        inner.offset()
    }

    #[inline]
    pub fn layouts(&self) -> &D {
        let Self { inner } = self;
        inner.layouts()
    }

    #[inline]
    pub fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a, D, P> ErasedBundleSlices<'_, D, P>
where
    D: FieldLayouts<'a, OutputItem: BufferOffsetsFromSelf, OutputIter: ErasedArchetypeIterator>
        + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn iter(
        &'a self,
    ) -> ErasedBundleSlicesIter<'a, D::OutputIter, P, BufferOffsetsOf<D::OutputItem>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicesIter::from_inner(inner) }
    }
}

impl<'a, D, P> ErasedBundleSlices<'a, D, P>
where
    D: ErasedArchetypeKind,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn downcast<B>(
        self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<BundleSlices<'a, B>, DowncastError<Self>>
    where
        B: Bundle,
    {
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let slices = self
            .into_ptrs()
            .downcast::<B>(components)
            .map_err(|error| error.map_value(into_self))?;

        let slices = unsafe { B::CONTEXT.slice_ptrs_to_slices(slices) };
        Ok(slices)
    }
}

impl<D, P> ErasedBundleSlices<'_, D, P>
where
    D: ErasedArchetypeKind + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_layouts()
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentSlice<'_, P>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }
}

impl<D, P> Debug for ErasedBundleSlices<'_, D, P>
where
    D: Debug + ?Sized,
    P: ConstSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        f.debug_struct("ErasedBundleSlices")
            .field("inner", &inner)
            .finish()
    }
}

impl<D, P> Clone for ErasedBundleSlices<'_, D, P>
where
    D: Clone,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        unsafe { Self::from_inner(inner) }
    }
}

impl<D, P> Copy for ErasedBundleSlices<'_, D, P>
where
    D: Copy,
    P: ConstSliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleSlices<'_, D, P>
where
    D: FieldLayouts<'a, OutputItem: BufferOffsetsFromSelf, OutputIter: ErasedArchetypeIterator>
        + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentSlice<'a, P>;
    type IntoIter = ErasedBundleSlicesIter<'a, D::OutputIter, P, BufferOffsetsOf<D::OutputItem>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P> IntoIterator for ErasedBundleSlices<'a, D, P>
where
    D: IntoErasedArchetypeIterator<Item: BufferOffsetsFromSelf>,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentSlice<'a, P>;
    type IntoIter = ErasedBundleSlicesIter<'a, D::IntoIter, P, BufferOffsetsOf<D::Item>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleSlicesIter::from_inner(inner) }
    }
}

impl<'a, D, P> FieldLayouts<'a> for ErasedBundleSlices<'_, D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Output = D::Output;
    type OutputIter = D::OutputIter;
    type OutputItem = D::OutputItem;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedBundleSlices<'_, D, P>
where
    D: CovariantFieldLayouts + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}

pub struct ErasedBundleSlicesIter<'a, D, P, F>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    inner: ErasedSoaSlicesIter<'a, D, P, F>,
}

impl<'a, D, P, F> ErasedBundleSlicesIter<'a, D, P, F>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn from_inner(inner: ErasedSoaSlicesIter<'a, D, P, F>) -> Self {
        Self { inner }
    }
}

impl<D, P, F> ErasedBundleSlicesIter<'_, D, P, F>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> &[P::Item] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        let Self { inner } = self;
        inner.capacity()
    }

    #[inline]
    pub fn offset(&self) -> usize {
        let Self { inner } = self;
        inner.offset()
    }

    #[inline]
    pub fn slice_len(&self) -> usize {
        let Self { inner } = self;
        inner.slice_len()
    }

    #[inline]
    pub fn layouts(&self) -> &D {
        let Self { inner, .. } = self;
        inner.layouts()
    }
}

impl<'a, D, P, F> ErasedBundleSlicesIter<'_, D, P, F>
where
    D: FieldLayouts<'a, OutputIter: ErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
    F: BufferOffsetsFrom<D::OutputItem> + Clone,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleSlicesIter<'a, D::OutputIter, P, F> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicesIter::from_inner(inner) }
    }
}

impl<'a, D, P, F> IntoIterator for &'a ErasedBundleSlicesIter<'_, D, P, F>
where
    D: FieldLayouts<'a, OutputIter: ErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
    F: BufferOffsetsFrom<D::OutputItem> + Clone,
{
    type Item = ErasedComponentSlice<'a, P>;
    type IntoIter = ErasedBundleSlicesIter<'a, D::OutputIter, P, F>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P, F> Debug for ErasedBundleSlicesIter<'_, D, P, F>
where
    D: FieldLayoutsOwned<OutputIter: ErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr<Item: Debug>,
    F: for<'a> BufferOffsetsFrom<FieldLayoutsItem<'a, D>> + Clone,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<D, P, F> Clone for ErasedBundleSlicesIter<'_, D, P, F>
where
    D: Clone,
    P: ConstSliceItemPtr,
    F: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<'a, D, P, F> Iterator for ErasedBundleSlicesIter<'a, D, P, F>
where
    D: ErasedArchetypeIterator + ?Sized,
    P: ConstSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    type Item = ErasedComponentSlice<'a, P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_layouts().into_iter().next()?.component_id();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentSlice::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<D, P, F> ExactSizeIterator for ErasedBundleSlicesIter<'_, D, P, F>
where
    D: ErasedArchetypeIterator + ExactSizeIterator + ?Sized,
    P: ConstSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<D, P, F> FusedIterator for ErasedBundleSlicesIter<'_, D, P, F>
where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized,
    P: ConstSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
}

impl<'a, D, P, F> FieldLayouts<'a> for ErasedBundleSlicesIter<'_, D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Output = D::Output;
    type OutputIter = D::OutputIter;
    type OutputItem = D::OutputItem;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_layouts()
    }
}

impl<D, P, F> CovariantFieldLayouts for ErasedBundleSlicesIter<'_, D, P, F>
where
    D: CovariantFieldLayouts + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}
