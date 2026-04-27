use gpecs_component::{
    erased::{
        ErasedComponentMutPtr, ErasedComponentPtr,
        error::{DowncastErrorKind, NotRegisteredError},
    },
    registry::{
        ComponentId, ComponentRegistry, ComponentRegistryView,
        traits::{ComponentIdFrom, ComponentIdFromOrInsertWith, FromComponentType, PushBackArray},
    },
};
use gpecs_soa_erased::{
    ptr::slice::{ConstSliceItemPtr, MutSliceItemPtr},
    soa::traits::{
        AllocSoa, MutPtrs, NonNullPtrs, Ptrs, Refs, RefsMut, SliceMutPtrs, SlicePtrs, Slices,
        SlicesMut, SoaOwned, SoaReadOwned, SoaWrite,
    },
};

pub type BundlePtrs<B> = Ptrs<'static, B>;
pub type BundleMutPtrs<B> = MutPtrs<'static, B>;
pub type BundleNonNullPtrs<B> = NonNullPtrs<'static, B>;

pub type BundleSlicePtrs<B> = SlicePtrs<'static, B>;
pub type BundleSliceMutPtrs<B> = SliceMutPtrs<'static, B>;

pub type BundleRefs<'a, B> = Refs<'static, 'a, B>;
pub type BundleRefsMut<'a, B> = RefsMut<'static, 'a, B>;

pub type BundleSlices<'a, B> = Slices<'static, 'a, B>;
pub type BundleSlicesMut<'a, B> = SlicesMut<'static, 'a, B>;

/// Non-empty collection of [components](gpecs_component::Component).
pub unsafe trait Bundle:
    SoaOwned + AllocSoa + SoaReadOwned<Self> + SoaWrite<Self> + Sized + 'static
{
    /// Static [SoA context](gpecs_soa_erased::soa::traits::SoaContext) instance of this bundle.
    ///
    /// This ensures that components of this bundle are known at compile time.
    const CONTEXT: &'static Self::Context;

    /// Non-empty collection of identifiers for all components of this bundle.
    ///
    /// Order of these identifiers should be the same
    /// as the order of corresponding [layouts](gpecs_soa_erased::soa::field::FieldLayouts::Output).
    type Components: IntoIterator<Item = ComponentId>;

    /// Retrieves identifiers of all already registered components of this bundle.
    fn get_components<T>(
        components: &ComponentRegistryView<impl Sized, T>,
    ) -> Result<Self::Components, NotRegisteredError>
    where
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized;

    /// Registers all components of this bundle inside of provided registry
    /// and returns their identifiers.
    fn register_components<T, M>(components: &mut ComponentRegistry<T, M>) -> Self::Components
    where
        T: PushBackArray<Item: FromComponentType>,
        M: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized;

    /// Attempts to downcast input collection of erased component pointers
    /// into the collection of pointers to components of this bundle.
    ///
    /// Note that the order of input pointers **may not** match
    /// with the order of components in this bundle.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - some of the components of this bundle were not registered,
    /// - some of the input pointers cannot be converted to the component of this bundle.
    fn ptrs_from_erased<I, T, P>(
        components: &ComponentRegistryView<impl Sized, T>,
        iter: I,
    ) -> Result<BundlePtrs<Self>, DowncastErrorKind>
    where
        I: IntoIterator<Item = ErasedComponentPtr<P>>,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
        P: ConstSliceItemPtr;

    /// Attempts to downcast input collection of erased mutable component pointers
    /// into the collection of mutable pointers to components of this bundle.
    ///
    /// Note that the order of input pointers **may not** match
    /// with the order of components in this bundle.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - some of the components of this bundle were not registered,
    /// - some of the input pointers cannot be converted to the component of this bundle.
    fn mut_ptrs_from_erased<I, T, P>(
        components: &ComponentRegistryView<impl Sized, T>,
        iter: I,
    ) -> Result<BundleMutPtrs<Self>, DowncastErrorKind>
    where
        I: IntoIterator<Item = ErasedComponentMutPtr<P>>,
        T: ComponentIdFrom<Key: FromComponentType> + ?Sized,
        P: MutSliceItemPtr;
}
