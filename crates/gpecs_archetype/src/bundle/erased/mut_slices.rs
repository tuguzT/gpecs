use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_component::{
    erased::{ErasedComponentMutSlice, ErasedComponentSlice},
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType, WithComponentId},
    },
};
use gpecs_soa_erased::{
    BufferOffsetsFrom, BufferOffsetsFromLayout, CovariantFieldLayouts, ErasedSoaMutSlices,
    ErasedSoaMutSlicesIter,
    ptr::slice::{CastConst, MutSliceItemPtr},
    soa::{
        field::{
            FieldLayouts, FieldLayoutsItem, FieldLayoutsIter, FieldLayoutsOutput, FieldLayoutsOwned,
        },
        traits::SoaContext,
    },
};

use crate::{
    bundle::{
        Bundle, BundleSlicesMut,
        erased::{
            ErasedBundleMutSlicePtrs, ErasedBundleSlices, ErasedBundleSlicesIter,
            error::DowncastError,
            traits::{ErasedArchetypeIterator, ErasedArchetypeKind, IntoErasedArchetypeIterator},
        },
    },
    erased::ErasedArchetypeView,
};

pub struct ErasedBundleMutSlices<'a, D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    inner: ErasedSoaMutSlices<'a, D, P>,
}

impl<'a, D, P> ErasedBundleMutSlices<'a, D, P>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_inner(inner: ErasedSoaMutSlices<'a, D, P>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundleMutSlicePtrs<D, P>) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { inner.as_mut_unchecked() };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub fn into_inner(self) -> ErasedSoaMutSlices<'a, D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedBundleMutSlicePtrs<D, P> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundleMutSlicePtrs::from_inner(inner) }
    }
}

impl<D, P> ErasedBundleMutSlices<'_, D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> &[P::Item] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(&mut self) -> &mut [P::Item] {
        let Self { inner } = self;
        inner.as_mut_buffer()
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

impl<'a, D, P> ErasedBundleMutSlices<'_, D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn iter(
        &'a self,
    ) -> ErasedBundleSlicesIter<'a, FieldLayoutsIter<'a, D>, CastConst<P>, BufferOffsetsFromLayout>
    {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicesIter::from_inner(inner) }
    }

    #[inline]
    pub fn iter_mut(
        &'a mut self,
    ) -> ErasedBundleMutSlicesIter<'a, FieldLayoutsIter<'a, D>, P, BufferOffsetsFromLayout> {
        let Self { inner } = self;

        let inner = inner.iter_mut();
        unsafe { ErasedBundleMutSlicesIter::from_inner(inner) }
    }
}

impl<'a, D, P> ErasedBundleMutSlices<'a, D, P>
where
    D: ErasedArchetypeKind,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn downcast<B>(
        self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<BundleSlicesMut<'a, B>, DowncastError<Self>>
    where
        B: Bundle,
    {
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let slices = self
            .into_ptrs()
            .downcast::<B>(components)
            .map_err(|error| error.map_value(into_self))?;

        let slices = unsafe { B::CONTEXT.mut_slice_ptrs_to_mut_slices(slices) };
        Ok(slices)
    }
}

impl<D, P> ErasedBundleMutSlices<'_, D, P>
where
    D: ErasedArchetypeKind + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_layouts()
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentSlice<'_, CastConst<P>>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }

    #[inline]
    pub fn get_mut(&mut self, component_id: ComponentId) -> Option<ErasedComponentMutSlice<'_, P>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter_mut().nth(index)
    }
}

impl<D, P> Debug for ErasedBundleMutSlices<'_, D, P>
where
    D: Debug + ?Sized,
    P: MutSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        f.debug_struct("ErasedBundleMutSlices")
            .field("inner", &inner)
            .finish()
    }
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleMutSlices<'_, D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentSlice<'a, CastConst<P>>;
    type IntoIter =
        ErasedBundleSlicesIter<'a, FieldLayoutsIter<'a, D>, CastConst<P>, BufferOffsetsFromLayout>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P> IntoIterator for &'a mut ErasedBundleMutSlices<'_, D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentMutSlice<'a, P>;
    type IntoIter =
        ErasedBundleMutSlicesIter<'a, FieldLayoutsIter<'a, D>, P, BufferOffsetsFromLayout>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, D, P> IntoIterator for ErasedBundleMutSlices<'a, D, P>
where
    D: IntoErasedArchetypeIterator,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentMutSlice<'a, P>;
    type IntoIter = ErasedBundleMutSlicesIter<'a, D::IntoIter, P, BufferOffsetsFromLayout>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleMutSlicesIter::from_inner(inner) }
    }
}

impl<'a, D, P> From<ErasedBundleMutSlices<'a, D, P>> for ErasedBundleSlices<'a, D, CastConst<P>>
where
    P: MutSliceItemPtr,
{
    #[inline]
    fn from(slices: ErasedBundleMutSlices<'a, D, P>) -> Self {
        let inner = slices.into_inner();
        let inner = inner.into();
        unsafe { Self::from_inner(inner) }
    }
}

impl<'a, D, P> FieldLayouts<'a> for ErasedBundleMutSlices<'_, D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedBundleMutSlices<'_, D, P>
where
    D: CovariantFieldLayouts + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}

pub struct ErasedBundleMutSlicesIter<'a, D, P, F>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    inner: ErasedSoaMutSlicesIter<'a, D, P, F>,
}

impl<'a, D, P, F> ErasedBundleMutSlicesIter<'a, D, P, F>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn from_inner(inner: ErasedSoaMutSlicesIter<'a, D, P, F>) -> Self {
        Self { inner }
    }
}

impl<D, P, F> ErasedBundleMutSlicesIter<'_, D, P, F>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> &[P::Item] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(&mut self) -> &mut [P::Item] {
        let Self { inner } = self;
        inner.as_mut_buffer()
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

impl<'a, D, P, F> ErasedBundleMutSlicesIter<'_, D, P, F>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
    F: BufferOffsetsFrom<FieldLayoutsItem<'a, D>> + Clone,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleMutSlicesIter<'a, FieldLayoutsIter<'a, D>, P, F> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleMutSlicesIter::from_inner(inner) }
    }
}

impl<'a, D, P, F> IntoIterator for &'a ErasedBundleMutSlicesIter<'_, D, P, F>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
    F: BufferOffsetsFrom<FieldLayoutsItem<'a, D>> + Clone,
{
    type Item = ErasedComponentMutSlice<'a, P>;
    type IntoIter = ErasedBundleMutSlicesIter<'a, FieldLayoutsIter<'a, D>, P, F>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P, F> Debug for ErasedBundleMutSlicesIter<'_, D, P, F>
where
    D: FieldLayoutsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr<Item: Debug>,
    F: for<'a> BufferOffsetsFrom<FieldLayoutsItem<'a, D>> + Clone,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<'a, D, P, F> Iterator for ErasedBundleMutSlicesIter<'a, D, P, F>
where
    D: ErasedArchetypeIterator + ?Sized,
    P: MutSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    type Item = ErasedComponentMutSlice<'a, P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_layouts().into_iter().next()?.component_id();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentMutSlice::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<D, P, F> ExactSizeIterator for ErasedBundleMutSlicesIter<'_, D, P, F>
where
    D: ErasedArchetypeIterator + ExactSizeIterator + ?Sized,
    P: MutSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<D, P, F> FusedIterator for ErasedBundleMutSlicesIter<'_, D, P, F>
where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized,
    P: MutSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
}

impl<'a, D, P, F> FieldLayouts<'a> for ErasedBundleMutSlicesIter<'_, D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_layouts()
    }
}

impl<D, P, F> CovariantFieldLayouts for ErasedBundleMutSlicesIter<'_, D, P, F>
where
    D: CovariantFieldLayouts + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}
