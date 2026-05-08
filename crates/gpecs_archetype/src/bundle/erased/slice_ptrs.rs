use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_component::{
    erased::ErasedComponentSlicePtr,
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType, WithComponentId},
    },
};
use gpecs_soa_erased::{
    BufferOffsetsFrom, BufferOffsetsFromLayout, CovariantFieldLayouts, ErasedSoaSlicePtrs,
    ErasedSoaSlicePtrsIter,
    ptr::slice::{CastMut, ConstSliceItemPtr},
    soa::{
        field::{
            FieldLayouts, FieldLayoutsItem, FieldLayoutsIter, FieldLayoutsOutput, FieldLayoutsOwned,
        },
        traits::RawSoaContext,
    },
};

use crate::{
    bundle::{
        Bundle, BundleSlicePtrs,
        erased::{
            ErasedBundleMutSlicePtrs, ErasedBundlePtrs, ErasedBundleSlices,
            error::DowncastError,
            traits::{ErasedArchetypeIterator, ErasedArchetypeKind, IntoErasedArchetypeIterator},
        },
    },
    erased::ErasedArchetypeView,
};

pub struct ErasedBundleSlicePtrs<D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    inner: ErasedSoaSlicePtrs<D, P>,
}

impl<D, P> ErasedBundleSlicePtrs<D, P>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_inner(inner: ErasedSoaSlicePtrs<D, P>) -> Self {
        Self { inner }
    }

    #[inline]
    pub unsafe fn from_ptrs(ptrs: ErasedBundlePtrs<D, P>, len: usize) -> Self {
        let inner = ptrs.into_inner();
        let inner = unsafe { ErasedSoaSlicePtrs::from_ptrs(inner, len) };
        unsafe { Self::from_inner(inner) }
    }

    #[inline]
    pub fn into_inner(self) -> ErasedSoaSlicePtrs<D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn into_ptrs(self) -> ErasedBundlePtrs<D, P> {
        let Self { inner } = self;

        let inner = inner.into_ptrs();
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }

    #[inline]
    pub fn cast_mut(self) -> ErasedBundleMutSlicePtrs<D, CastMut<P>> {
        let Self { inner } = self;

        let inner = inner.cast_mut();
        unsafe { ErasedBundleMutSlicePtrs::from_inner(inner) }
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> ErasedBundleSlices<'a, D, P> {
        unsafe { ErasedBundleSlices::from_ptrs(self) }
    }
}

impl<D, P> ErasedBundleSlicePtrs<D, P>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [P::Item] {
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

impl<'a, D, P> ErasedBundleSlicePtrs<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn iter(
        &'a self,
    ) -> ErasedBundleSlicePtrsIter<FieldLayoutsIter<'a, D>, P, BufferOffsetsFromLayout> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicePtrsIter::from_inner(inner) }
    }
}

impl<D, P> ErasedBundleSlicePtrs<D, P>
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
    ) -> Result<BundleSlicePtrs<B>, DowncastError<Self>>
    where
        B: Bundle,
    {
        let len = self.len();
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs, len) };
        let ptrs = self
            .into_ptrs()
            .downcast::<B>(components)
            .map_err(|error| error.map_value(into_self))?;

        let slices = B::CONTEXT.slice_ptrs_from_raw_parts(ptrs, len);
        Ok(slices)
    }
}

impl<D, P> ErasedBundleSlicePtrs<D, P>
where
    D: ErasedArchetypeKind + ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_layouts()
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentSlicePtr<P>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }
}

impl<D, P> Debug for ErasedBundleSlicePtrs<D, P>
where
    D: Debug + ?Sized,
    P: ConstSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        f.debug_struct("ErasedBundleSlicePtrs")
            .field("inner", &inner)
            .finish()
    }
}

impl<D, P> Clone for ErasedBundleSlicePtrs<D, P>
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

impl<D, P> Copy for ErasedBundleSlicePtrs<D, P>
where
    D: Copy,
    P: ConstSliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleSlicePtrs<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentSlicePtr<P>;
    type IntoIter = ErasedBundleSlicePtrsIter<FieldLayoutsIter<'a, D>, P, BufferOffsetsFromLayout>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> IntoIterator for ErasedBundleSlicePtrs<D, P>
where
    D: IntoErasedArchetypeIterator,
    P: ConstSliceItemPtr,
{
    type Item = ErasedComponentSlicePtr<P>;
    type IntoIter = ErasedBundleSlicePtrsIter<D::IntoIter, P, BufferOffsetsFromLayout>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleSlicePtrsIter::from_inner(inner) }
    }
}

impl<'a, D, P> FieldLayouts<'a> for ErasedBundleSlicePtrs<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedBundleSlicePtrs<D, P>
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

pub struct ErasedBundleSlicePtrsIter<D, P, F>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    inner: ErasedSoaSlicePtrsIter<D, P, F>,
}

impl<D, P, F> ErasedBundleSlicePtrsIter<D, P, F>
where
    P: ConstSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn from_inner(inner: ErasedSoaSlicePtrsIter<D, P, F>) -> Self {
        Self { inner }
    }
}

impl<D, P, F> ErasedBundleSlicePtrsIter<D, P, F>
where
    D: ?Sized,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [P::Item] {
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

impl<'a, D, P, F> ErasedBundleSlicePtrsIter<D, P, F>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
    F: BufferOffsetsFrom<FieldLayoutsItem<'a, D>> + Clone,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleSlicePtrsIter<FieldLayoutsIter<'a, D>, P, F> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleSlicePtrsIter::from_inner(inner) }
    }
}

impl<'a, D, P, F> IntoIterator for &'a ErasedBundleSlicePtrsIter<D, P, F>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr,
    F: BufferOffsetsFrom<FieldLayoutsItem<'a, D>> + Clone,
{
    type Item = ErasedComponentSlicePtr<P>;
    type IntoIter = ErasedBundleSlicePtrsIter<FieldLayoutsIter<'a, D>, P, F>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P, F> Debug for ErasedBundleSlicePtrsIter<D, P, F>
where
    D: FieldLayoutsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
    P: ConstSliceItemPtr + Debug,
    F: for<'a> BufferOffsetsFrom<FieldLayoutsItem<'a, D>> + Clone,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<D, P, F> Clone for ErasedBundleSlicePtrsIter<D, P, F>
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

impl<D, P, F> Iterator for ErasedBundleSlicePtrsIter<D, P, F>
where
    D: ErasedArchetypeIterator + ?Sized,
    P: ConstSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
    type Item = ErasedComponentSlicePtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_layouts().into_iter().next()?.component_id();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentSlicePtr::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<D, P, F> ExactSizeIterator for ErasedBundleSlicePtrsIter<D, P, F>
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

impl<D, P, F> FusedIterator for ErasedBundleSlicePtrsIter<D, P, F>
where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized,
    P: ConstSliceItemPtr,
    F: BufferOffsetsFrom<D::Item>,
{
}

impl<'a, D, P, F> FieldLayouts<'a> for ErasedBundleSlicePtrsIter<D, P, F>
where
    D: FieldLayouts<'a> + ?Sized,
    P: ConstSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        let Self { inner } = self;
        inner.field_layouts()
    }
}

impl<D, P, F> CovariantFieldLayouts for ErasedBundleSlicePtrsIter<D, P, F>
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
