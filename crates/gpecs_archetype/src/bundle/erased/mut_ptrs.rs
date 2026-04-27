use core::{
    fmt::{self, Debug},
    iter::FusedIterator,
};

use gpecs_component::{
    erased::{
        ErasedComponentMutPtr, ErasedComponentMutSlicePtr, ErasedComponentPtr,
        error::NotRegisteredError,
    },
    registry::{
        ComponentId, ComponentRegistryView,
        traits::{ComponentIdFrom, FromComponentType, WithComponentId},
    },
};
use gpecs_soa_erased::{
    CovariantFieldLayouts, ErasedSoaMutPtrs, ErasedSoaMutPtrsIter,
    ptr::slice::{CastConst, MutSliceItemPtr},
    soa::field::{FieldLayouts, FieldLayoutsIter, FieldLayoutsOutput, FieldLayoutsOwned},
    storage::AlignedStorage,
};
use itertools::equal;

use crate::{
    bundle::{
        Bundle, BundleMutPtrs,
        erased::{
            ErasedBundleKind, ErasedBundleMutRefs, ErasedBundlePtrs, ErasedBundlePtrsIter,
            ErasedBundleRefs,
            error::DowncastError,
            traits::{
                ErasedArchetypeIterator, ErasedArchetypeKind, ErasedBundleDrop,
                IntoErasedArchetypeIterator,
            },
        },
    },
    erased::ErasedArchetypeView,
};

pub struct ErasedBundleMutPtrs<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    inner: ErasedSoaMutPtrs<D, P>,
}

impl<D, P> ErasedBundleMutPtrs<D, P>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn from_inner(inner: ErasedSoaMutPtrs<D, P>) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn into_inner(self) -> ErasedSoaMutPtrs<D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn cast_const(self) -> ErasedBundlePtrs<D, CastConst<P>> {
        let Self { inner } = self;

        let inner = inner.cast_const();
        unsafe { ErasedBundlePtrs::from_inner(inner) }
    }

    #[inline]
    pub unsafe fn as_ref_unchecked<'a>(self) -> ErasedBundleRefs<'a, D, CastConst<P>> {
        unsafe { self.cast_const().as_ref_unchecked() }
    }

    #[inline]
    pub unsafe fn as_mut_unchecked<'a>(self) -> ErasedBundleMutRefs<'a, D, P> {
        unsafe { ErasedBundleMutRefs::from_ptrs(self) }
    }

    #[inline]
    #[must_use]
    pub unsafe fn add(self, count: usize) -> Self {
        let Self { inner } = self;

        let inner = unsafe { inner.add(count) };
        unsafe { Self::from_inner(inner) }
    }
}

impl<D, P> ErasedBundleMutPtrs<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn as_inner(&self) -> &ErasedSoaMutPtrs<D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub unsafe fn as_mut_inner(&mut self) -> &mut ErasedSoaMutPtrs<D, P> {
        let Self { inner } = self;
        inner
    }

    #[inline]
    pub fn as_buffer(&self) -> *const [P::Item] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(&mut self) -> *mut [P::Item] {
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
}

impl<'a, D, P> ErasedBundleMutPtrs<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    #[track_caller]
    pub unsafe fn offset_from<'n, N>(
        &'a self,
        origin: &'n ErasedBundlePtrs<N, CastConst<P>>,
    ) -> isize
    where
        N: FieldLayouts<'n, Output: IntoErasedArchetypeIterator> + ?Sized,
    {
        let Self { inner } = self;

        let origin = unsafe { origin.as_inner() };
        unsafe { inner.offset_from(origin) }
    }

    #[inline]
    pub fn iter(&'a self) -> ErasedBundlePtrsIter<FieldLayoutsIter<'a, D>, CastConst<P>> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundlePtrsIter::from_inner(inner) }
    }

    #[inline]
    pub fn iter_mut(&'a mut self) -> ErasedBundleMutPtrsIter<FieldLayoutsIter<'a, D>, P> {
        let Self { inner } = self;

        let inner = inner.iter_mut();
        unsafe { ErasedBundleMutPtrsIter::from_inner(inner) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn swap<'n, N>(&'a mut self, with: &'n mut ErasedBundleMutPtrs<N, P>)
    where
        N: FieldLayouts<'n, Output: IntoErasedArchetypeIterator> + ?Sized,
    {
        let Self { inner } = self;

        let with = unsafe { with.as_mut_inner() };
        unsafe { inner.swap(with) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_forward<'n, N>(
        &'a mut self,
        src: &'n ErasedBundlePtrs<N, CastConst<P>>,
        count: usize,
    ) where
        N: FieldLayouts<'n, Output: IntoErasedArchetypeIterator> + ?Sized,
    {
        let Self { inner } = self;

        let src = unsafe { src.as_inner() };
        unsafe { inner.copy_from_forward(src, count) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_backward<'n, N>(
        &'a mut self,
        src: &'n ErasedBundlePtrs<N, CastConst<P>>,
        count: usize,
    ) where
        N: FieldLayouts<'n, Output: IntoErasedArchetypeIterator> + ?Sized,
    {
        let Self { inner } = self;

        let src = unsafe { src.as_inner() };
        unsafe { inner.copy_from_backward(src, count) }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn copy_from_nonoverlapping<'n, N>(
        &'a mut self,
        src: &'n ErasedBundlePtrs<N, CastConst<P>>,
        count: usize,
    ) where
        N: FieldLayouts<'n, Output: IntoErasedArchetypeIterator> + ?Sized,
    {
        let Self { inner } = self;

        let src = unsafe { src.as_inner() };
        unsafe { inner.copy_from_nonoverlapping(src, count) }
    }
}

impl<D, P> ErasedBundleMutPtrs<D, P>
where
    D: FieldLayoutsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn drop_in_place<M, K>(
        &mut self,
        components: &ComponentRegistryView<M, impl ?Sized>,
    ) -> Result<(), NotRegisteredError>
    where
        K: ErasedBundleDrop<M>,
    {
        self.iter()
            .map(ErasedComponentPtr::component_id)
            .try_for_each(|id| {
                components
                    .get_component_descriptor(id)
                    .map(drop)
                    .ok_or_else(NotRegisteredError::new)
            })?;

        self.iter_mut().for_each(|to_drop| {
            let component_id = to_drop.component_id();
            let info = components
                .get_component_descriptor(component_id)
                .expect("component info should exist");
            unsafe { K::drop_in_place_with(to_drop, info) }
        });
        Ok(())
    }
}

impl<D, P> ErasedBundleMutPtrs<D, P>
where
    D: ErasedArchetypeKind,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn downcast<B>(
        mut self,
        components: &ComponentRegistryView<
            impl Sized,
            impl ComponentIdFrom<Key: FromComponentType> + ?Sized,
        >,
    ) -> Result<BundleMutPtrs<B>, DowncastError<Self>>
    where
        B: Bundle,
    {
        if let Err(error) = self.archetype().check_compatibility_of::<B>(components) {
            return Err(DowncastError::new(self, error.into()));
        }
        let ptrs = B::mut_ptrs_from_erased(components, self.iter_mut())
            .map_err(|error| DowncastError::new(self, error.into()))?;
        Ok(ptrs)
    }
}

impl<D, P> ErasedBundleMutPtrs<D, P>
where
    D: ErasedArchetypeKind + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn archetype(&self) -> ErasedArchetypeView<'_, D::Meta> {
        self.field_layouts()
    }

    #[inline]
    pub fn get(&self, component_id: ComponentId) -> Option<ErasedComponentPtr<CastConst<P>>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter().nth(index)
    }

    #[inline]
    pub fn get_mut(&mut self, component_id: ComponentId) -> Option<ErasedComponentMutPtr<P>> {
        let index = self.archetype().get_index_of(component_id)?;
        self.iter_mut().nth(index)
    }

    #[inline]
    pub unsafe fn copy_from_compatible_nonoverlapping<N>(
        &mut self,
        src: &ErasedBundlePtrs<N, CastConst<P>>,
        count: usize,
    ) where
        N: ErasedArchetypeKind + ?Sized,
    {
        let archetype = self.archetype();
        let src_archetype = src.archetype();
        archetype
            .check_compatibility(src_archetype)
            .expect("archetypes should be compatible");

        if equal(archetype.component_ids(), src_archetype.component_ids()) {
            unsafe { self.copy_from_nonoverlapping(src, count) };
            return;
        }

        for src in src.iter() {
            let component_id = src.component_id();
            let dst = self
                .get_mut(component_id)
                .expect("dst should have the same component as src has");
            unsafe { dst.copy_from_nonoverlapping(src, count) }
        }
    }

    #[inline]
    pub unsafe fn copy_from_compatible_exact_nonoverlapping<N>(
        &mut self,
        src: &ErasedBundlePtrs<N, CastConst<P>>,
        count: usize,
    ) where
        N: ErasedArchetypeKind + ?Sized,
    {
        let archetype = self.archetype();
        let src_archetype = src.archetype();
        archetype
            .check_exact_compatibility(src_archetype)
            .expect("archetypes should be exact compatible");

        unsafe { self.copy_from_compatible_nonoverlapping(src, count) }
    }

    #[inline]
    pub unsafe fn move_from_compatible_nonoverlapping<N, K>(
        &mut self,
        src: &ErasedBundlePtrs<N, CastConst<P>>,
        count: usize,
    ) where
        N: ErasedArchetypeKind + ?Sized,
        K: ErasedBundleDrop<D::Meta>,
    {
        let archetype = self.archetype();
        let src_archetype = src.archetype();
        archetype
            .check_compatibility(src_archetype)
            .expect("archetypes should be compatible");

        for src in src.iter() {
            let component_id = src.component_id();
            let dst = self
                .get_mut(component_id)
                .expect("dst should have the same component as src has");
            let meta = self
                .archetype()
                .into_get(component_id)
                .expect("archetype should contain component");

            let to_drop = unsafe { ErasedComponentMutSlicePtr::from_ptr(dst, count) };
            unsafe { K::drop_in_place_slice_with(to_drop, meta) }

            unsafe { dst.copy_from_nonoverlapping(src, count) }
        }
    }
}

impl<D, P> ErasedBundleMutPtrs<D, P>
where
    D: ErasedArchetypeKind + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub unsafe fn write<T, K, S>(&mut self, value: ErasedBundleKind<T, K, S, P::Ptrs>)
    where
        T: ErasedArchetypeKind,
        K: ErasedBundleDrop<T::Meta>,
        S: AlignedStorage<Item = P::Item>,
    {
        let src = value.as_ptrs();
        unsafe { self.copy_from_compatible_exact_nonoverlapping(&src, 1) };

        let _ = value.into_inner().into_parts();
    }
}

impl<D, P> Debug for ErasedBundleMutPtrs<D, P>
where
    D: Debug + ?Sized,
    P: MutSliceItemPtr,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { inner } = self;
        f.debug_struct("ErasedBundleMutPtrs")
            .field("inner", &inner)
            .finish()
    }
}

impl<D, P> Clone for ErasedBundleMutPtrs<D, P>
where
    D: Clone,
    P: MutSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        unsafe { Self::from_inner(inner) }
    }
}

impl<D, P> Copy for ErasedBundleMutPtrs<D, P>
where
    D: Copy,
    P: MutSliceItemPtr,
{
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleMutPtrs<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentPtr<CastConst<P>>;
    type IntoIter = ErasedBundlePtrsIter<FieldLayoutsIter<'a, D>, CastConst<P>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, D, P> IntoIterator for &'a mut ErasedBundleMutPtrs<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentMutPtr<P>;
    type IntoIter = ErasedBundleMutPtrsIter<FieldLayoutsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<D, P> IntoIterator for ErasedBundleMutPtrs<D, P>
where
    D: IntoErasedArchetypeIterator,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentMutPtr<P>;
    type IntoIter = ErasedBundleMutPtrsIter<D::IntoIter, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self { inner } = self;

        let inner = inner.into_iter();
        unsafe { ErasedBundleMutPtrsIter::from_inner(inner) }
    }
}

impl<'a, D, P> FieldLayouts<'a> for ErasedBundleMutPtrs<D, P>
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

impl<D, P> CovariantFieldLayouts for ErasedBundleMutPtrs<D, P>
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

pub struct ErasedBundleMutPtrsIter<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    inner: ErasedSoaMutPtrsIter<D, P>,
}

impl<D, P> ErasedBundleMutPtrsIter<D, P>
where
    P: MutSliceItemPtr,
{
    #[inline]
    pub(super) unsafe fn from_inner(inner: ErasedSoaMutPtrsIter<D, P>) -> Self {
        Self { inner }
    }
}

impl<D, P> ErasedBundleMutPtrsIter<D, P>
where
    D: ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn as_buffer(&self) -> *const [P::Item] {
        let Self { inner } = self;
        inner.as_buffer()
    }

    #[inline]
    pub unsafe fn as_mut_buffer(&mut self) -> *mut [P::Item] {
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
        let Self { inner, .. } = self;
        inner.layouts()
    }
}

impl<'a, D, P> ErasedBundleMutPtrsIter<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn iter(&'a self) -> ErasedBundleMutPtrsIter<FieldLayoutsIter<'a, D>, P> {
        let Self { inner } = self;

        let inner = inner.iter();
        unsafe { ErasedBundleMutPtrsIter::from_inner(inner) }
    }
}

impl<'a, D, P> IntoIterator for &'a ErasedBundleMutPtrsIter<D, P>
where
    D: FieldLayouts<'a, Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentMutPtr<P>;
    type IntoIter = ErasedBundleMutPtrsIter<FieldLayoutsIter<'a, D>, P>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<D, P> Debug for ErasedBundleMutPtrsIter<D, P>
where
    D: FieldLayoutsOwned<Output: IntoErasedArchetypeIterator> + ?Sized,
    P: MutSliceItemPtr + Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<D, P> Clone for ErasedBundleMutPtrsIter<D, P>
where
    D: Clone,
    P: MutSliceItemPtr,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { inner } = self;

        let inner = inner.clone();
        Self { inner }
    }
}

impl<D, P> Iterator for ErasedBundleMutPtrsIter<D, P>
where
    D: ErasedArchetypeIterator + ?Sized,
    P: MutSliceItemPtr,
{
    type Item = ErasedComponentMutPtr<P>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { inner } = self;

        let component_id = inner.field_layouts().into_iter().next()?.component_id();
        let fields = inner.next()?;
        let item = unsafe { ErasedComponentMutPtr::from_parts(component_id, fields) };
        Some(item)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { inner } = self;
        inner.size_hint()
    }
}

impl<D, P> ExactSizeIterator for ErasedBundleMutPtrsIter<D, P>
where
    D: ErasedArchetypeIterator + ExactSizeIterator + ?Sized,
    P: MutSliceItemPtr,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { inner } = self;
        inner.len()
    }
}

impl<D, P> FusedIterator for ErasedBundleMutPtrsIter<D, P>
where
    D: ErasedArchetypeIterator + FusedIterator + ?Sized,
    P: MutSliceItemPtr,
{
}

impl<'a, D, P> FieldLayouts<'a> for ErasedBundleMutPtrsIter<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: MutSliceItemPtr,
{
    type Output = D::Output;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        self.layouts().field_layouts()
    }
}

impl<D, P> CovariantFieldLayouts for ErasedBundleMutPtrsIter<D, P>
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
