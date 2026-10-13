use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
    ptr::NonNull,
};

use gpecs_component::{
    erased::ErasedComponentNonNullSlicePtr,
    registry::{
        ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType, WithComponentId},
    },
};
use gpecs_soa_erased::{
    BufferOffsetsFrom, BufferOffsetsFromSelf, BufferOffsetsOf, CovariantFieldLayouts,
    ErasedSoaNonNullSlicePtrs, ErasedSoaNonNullSlicePtrsIter,
    ptr::slice::{ConstPtr, MutPtr, NonNullSliceItemPtr},
    soa::{field::FieldLayouts, traits::SoaRawContext},
};
use gpecs_sparse::soa::field::{FieldLayoutsItem, FieldLayoutsOutput, FieldLayoutsOwned};

use crate::{
    bundle::{
        Bundle,
        erased::{
            ErasedBundleMutPtrs, ErasedBundleMutSlicePtrs, ErasedBundleNonNullPtrs,
            ErasedBundlePtrs, ErasedBundleSlicePtrs,
            error::DowncastError,
            traits::{ErasedArchetypeIterator, ErasedArchetypeKind, IntoErasedArchetypeIterator},
        },
        traits::BundleSliceNonNullPtrs,
    },
    erased::ErasedArchetypeView,
};

pub struct ErasedBundleNonNullSlicePtrs<D, P>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    inner: ErasedSoaNonNullSlicePtrs<D, P>,
}

impl<D, P> ErasedBundleNonNullSlicePtrs<D, P>
where
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_inner(inner: ErasedSoaNonNullSlicePtrs<D, P>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundleNonNullPtrs<D, P>, len: usize) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { ErasedSoaNonNullSlicePtrs::from_ptrs(inner, len) };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub fn into_inner(self) -> ErasedSoaNonNullSlicePtrs<D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedBundlePtrs<D, ConstPtr<P::Ptrs>> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }

    #[inline]
    pub fn into_mut_ptrs(self) -> ErasedBundleMutPtrs<D, MutPtr<P::Ptrs>> {
        let Self { inner } = self;

        let inner = inner.into_mut_ptrs();
        unsafe { ErasedBundleMutPtrs::from_inner(inner) }
    }

    #[inline]
    pub fn into_nonnull_ptrs(self) -> ErasedBundleNonNullPtrs<D, P> {
        let Self { inner } = self;

        let inner = inner.into_nonnull_ptrs();
        unsafe { ErasedBundleNonNullPtrs::from_inner(inner) }
    }

    #[inline]
    pub fn into_slice_ptrs(self) -> ErasedBundleSlicePtrs<D, ConstPtr<P::Ptrs>> {
        let Self { inner } = self;

        let inner = inner.into_slice_ptrs();
        unsafe { ErasedBundleSlicePtrs::from_inner(inner) }
    }

    #[inline]
    pub fn into_mut_slice_ptrs(self) -> ErasedBundleMutSlicePtrs<D, MutPtr<P::Ptrs>> {
        let Self { inner } = self;

        let inner = inner.into_mut_slice_ptrs();
        unsafe { ErasedBundleMutSlicePtrs::from_inner(inner) }
    }
}

impl<D, P> ErasedBundleNonNullSlicePtrs<D, P>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> NonNull<[P::Item]> {
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
    pub fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn layouts(&self) -> &D {
        let Self { inner } = self;
        inner.layouts()
    }
}

impl<'a, D, P> ErasedBundleNonNullSlicePtrs<D, P>
where
    D: FieldLayouts<'a, OutputItem: BufferOffsetsFromSelf, OutputIter: ErasedArchetypeIterator>
        + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn iter(
        &'a self,
    ) -> ErasedBundleNonNullSlicePtrsIter<D::OutputIter, P, BufferOffsetsOf<D::OutputItem>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleNonNullSlicePtrsIter::from_inner(inner) }
    }
}

impl<D, P> ErasedBundleNonNullSlicePtrs<D, P>
where
    D: ErasedArchetypeKind,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn downcast<B>(
        self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<BundleSliceNonNullPtrs<B>, DowncastError<Self>>
    where
        B: Bundle,
    {
        let len = self.len();
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs, len) };
        let ptrs = self
            .into_nonnull_ptrs()
            .downcast::<B>(components)
            .map_err(|error| error.map_value(into_self))?;

        let slices = B::CONTEXT.nonnull_slice_ptrs_from_raw_parts(ptrs, len);
        Ok(slices)
    }
}

impl<D, P> ErasedBundleNonNullSlicePtrs<D, P>
where
    D: ErasedArchetypeKind + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_layouts()
    }

    // #[inline]
    // pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentSlicePtr<CastConst<P>>> {
    //     let index = self.archetype().get_index_of(component_id)?;
    //     self.iter().nth(index)
    // }

    // #[inline]
    // pub fn get_mut(&mut self, component_id: ComponentId) -> Option<ErasedComponentMutSlicePtr<P>> {
    //     let index = self.archetype().get_index_of(component_id)?;
    //     self.iter_mut().nth(index)
    // }
}

impl<D, P> Debug for ErasedBundleNonNullSlicePtrs<D, P>
where
    D: Debug + ?Sized,
    P: NonNullSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        f.debug_struct("ErasedBundleNonNullSlicePtrs")
            .field("inner", &inner)
            .finish()
    }
}

impl<D, P> Clone for ErasedBundleNonNullSlicePtrs<D, P>
where
    D: Clone,
    P: NonNullSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        unsafe { Self::from_inner(inner) }
    }
}

impl<D, P> Copy for ErasedBundleNonNullSlicePtrs<D, P>
where
    D: Copy,
    P: NonNullSliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleNonNullSlicePtrs<D, P>
where
    D: FieldLayouts<'a, OutputItem: BufferOffsetsFromSelf, OutputIter: ErasedArchetypeIterator>
        + ?Sized,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedComponentNonNullSlicePtr<P>;
    type IntoIter =
        ErasedBundleNonNullSlicePtrsIter<D::OutputIter, P, BufferOffsetsOf<D::OutputItem>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> IntoIterator for ErasedBundleNonNullSlicePtrs<D, P>
where
    D: IntoErasedArchetypeIterator<Item: BufferOffsetsFromSelf>,
    P: NonNullSliceItemPtr,
{
    type Item = ErasedComponentNonNullSlicePtr<P>;
    type IntoIter = ErasedBundleNonNullSlicePtrsIter<D::IntoIter, P, BufferOffsetsOf<D::Item>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleNonNullSlicePtrsIter::from_inner(inner) }
    }
}

impl<'a, D, P> FieldLayouts<'a> for ErasedBundleNonNullSlicePtrs<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: NonNullSliceItemPtr,
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

impl<D, P> CovariantFieldLayouts for ErasedBundleNonNullSlicePtrs<D, P>
where
    D: CovariantFieldLayouts + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}

pub struct ErasedBundleNonNullSlicePtrsIter<D, P, F>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    inner: ErasedSoaNonNullSlicePtrsIter<D, P, F>,
}

impl<D, P, F> ErasedBundleNonNullSlicePtrsIter<D, P, F>
where
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn from_inner(inner: ErasedSoaNonNullSlicePtrsIter<D, P, F>) -> Self {
        Self { inner }
    }
}

impl<D, P, F> ErasedBundleNonNullSlicePtrsIter<D, P, F>
where
    D: ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> NonNull<[P::Item]> {
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

impl<'a, D, P, F> ErasedBundleNonNullSlicePtrsIter<D, P, F>
where
    D: FieldLayouts<'a, OutputIter: ErasedArchetypeIterator> + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::OutputItem> + Clone,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleNonNullSlicePtrsIter<D::OutputIter, P, F> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleNonNullSlicePtrsIter::from_inner(inner) }
    }
}

impl<'a, D, P, F> IntoIterator for &'a ErasedBundleNonNullSlicePtrsIter<D, P, F>
where
    D: FieldLayouts<'a, OutputIter: ErasedArchetypeIterator> + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::OutputItem> + Clone,
{
    type Item = ErasedComponentNonNullSlicePtr<P>;
    type IntoIter = ErasedBundleNonNullSlicePtrsIter<D::OutputIter, P, F>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P, F> Debug for ErasedBundleNonNullSlicePtrsIter<D, P, F>
where
    D: FieldLayoutsOwned<OutputIter: ErasedArchetypeIterator> + ?Sized,
    P: NonNullSliceItemPtr + Debug,
    F: for<'a> BufferOffsetsFrom<FieldLayoutsItem<'a, D>> + Clone,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<D, P, F> Clone for ErasedBundleNonNullSlicePtrsIter<D, P, F>
where
    D: Clone,
    P: NonNullSliceItemPtr,
    F: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<D, P, F> Iterator for ErasedBundleNonNullSlicePtrsIter<D, P, F>
where
    D: ErasedArchetypeIterator + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    type Item = ErasedComponentNonNullSlicePtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_layouts().into_iter().next()?.component_id();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentNonNullSlicePtr::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<D, P, F> ExactSizeIterator for ErasedBundleNonNullSlicePtrsIter<D, P, F>
where
    D: ErasedArchetypeIterator + ExactSizeIterator + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<D, P, F> FusedIterator for ErasedBundleNonNullSlicePtrsIter<D, P, F>
where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized,
    P: NonNullSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
}

impl<'a, D, P, F> FieldLayouts<'a> for ErasedBundleNonNullSlicePtrsIter<D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: NonNullSliceItemPtr,
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

impl<D, P, F> CovariantFieldLayouts for ErasedBundleNonNullSlicePtrsIter<D, P, F>
where
    D: CovariantFieldLayouts + ?Sized,
    P: NonNullSliceItemPtr,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}
