use gpecs_component::registry::{
    ComponentRegistryView,
    traits::{ComponentIdFrom, FromComponentType},
};
use gpecs_soa_erased::{
    ptr::slice::{ConstSliceItemPtr, MutSliceItemPtr, NonNullSliceItemPtr},
    soa::traits::{RawSoaContext, SoaContext},
};

use crate::bundle::{
    Bundle, BundleMutPtrs, BundleNonNullPtrs, BundlePtrs, BundleRefs, BundleRefsMut,
    BundleSliceMutPtrs, BundleSlicePtrs, BundleSlices, BundleSlicesMut,
    erased::{
        ErasedBundleMutPtrs, ErasedBundleMutRefs, ErasedBundleMutSlicePtrs, ErasedBundleMutSlices,
        ErasedBundleNonNullPtrs, ErasedBundlePtrs, ErasedBundleRefs, ErasedBundleSlicePtrs,
        ErasedBundleSlices, error::DowncastError, traits::ErasedArchetypeKind,
    },
};

impl<D, P> ErasedBundleMutPtrs<D, P>
where
    D: ErasedArchetypeKind,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        mut self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleMutPtrs<B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        if let Err(error) = self.archetype().check_compatibility_of::<B, T>(components) {
            return Err(DowncastError::new(self, error.into()));
        }
        let ptrs = B::mut_ptrs_from_erased(components, self.iter_mut())
            .map_err(|error| DowncastError::new(self, error.into()))?;
        Ok(ptrs)
    }
}

impl<'a, D, P> ErasedBundleMutRefs<'a, D, P>
where
    D: ErasedArchetypeKind,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleRefsMut<'a, B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let ptrs = self
            .into_ptrs()
            .downcast::<B, T>(components)
            .map_err(|error| error.map_value(into_self))?;

        let refs = unsafe { B::CONTEXT.mut_ptrs_to_mut_refs(ptrs) };
        Ok(refs)
    }
}

impl<D, P> ErasedBundleMutSlicePtrs<D, P>
where
    D: ErasedArchetypeKind,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleSliceMutPtrs<B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let len = self.len();
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs, len) };
        let ptrs = self
            .into_ptrs()
            .downcast::<B, T>(components)
            .map_err(|error| error.map_value(into_self))?;

        let slices = B::CONTEXT.mut_slice_ptrs_from_raw_parts(ptrs, len);
        Ok(slices)
    }
}

impl<'a, D, P> ErasedBundleMutSlices<'a, D, P>
where
    D: ErasedArchetypeKind,
    P: MutSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleSlicesMut<'a, B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let slices = self
            .into_ptrs()
            .downcast::<B, T>(components)
            .map_err(|error| error.map_value(into_self))?;

        let slices = unsafe { B::CONTEXT.mut_slice_ptrs_to_mut_slices(slices) };
        Ok(slices)
    }
}

impl<D, P> ErasedBundleNonNullPtrs<D, P>
where
    D: ErasedArchetypeKind,
    P: NonNullSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleNonNullPtrs<B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let into_self = |ptrs| unsafe { Self::new_unchecked(ptrs) };
        let ptrs = ErasedBundleMutPtrs::from(self)
            .downcast::<B, T>(components)
            .map_err(|error| error.map_value(into_self))?;

        let ptrs = unsafe { B::CONTEXT.ptrs_to_nonnull(ptrs) };
        Ok(ptrs)
    }
}

impl<D, P> ErasedBundlePtrs<D, P>
where
    D: ErasedArchetypeKind,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundlePtrs<B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        if let Err(error) = self.archetype().check_compatibility_of::<B, T>(components) {
            return Err(DowncastError::new(self, error.into()));
        }

        let ptrs = B::ptrs_from_erased(components, self.iter())
            .map_err(|error| DowncastError::new(self, error.into()))?;
        Ok(ptrs)
    }
}

impl<'a, D, P> ErasedBundleRefs<'a, D, P>
where
    D: ErasedArchetypeKind,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleRefs<'a, B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let ptrs = self
            .into_ptrs()
            .downcast::<B, T>(components)
            .map_err(|error| error.map_value(into_self))?;

        let refs = unsafe { B::CONTEXT.ptrs_to_refs(ptrs) };
        Ok(refs)
    }
}

impl<D, P> ErasedBundleSlicePtrs<D, P>
where
    D: ErasedArchetypeKind,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleSlicePtrs<B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let len = self.len();
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs, len) };
        let ptrs = self
            .into_ptrs()
            .downcast::<B, T>(components)
            .map_err(|error| error.map_value(into_self))?;

        let slices = B::CONTEXT.slice_ptrs_from_raw_parts(ptrs, len);
        Ok(slices)
    }
}

impl<'a, D, P> ErasedBundleSlices<'a, D, P>
where
    D: ErasedArchetypeKind,
    P: ConstSliceItemPtr,
{
    #[inline]
    pub fn downcast<B, T>(
        self,
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<BundleSlices<'a, B>, DowncastError<Self>>
    where
        B: Bundle,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let into_self = |ptrs| unsafe { Self::from_ptrs(ptrs) };
        let slices = self
            .into_ptrs()
            .downcast::<B, T>(components)
            .map_err(|error| error.map_value(into_self))?;

        let slices = unsafe { B::CONTEXT.slice_ptrs_to_slices(slices) };
        Ok(slices)
    }
}
