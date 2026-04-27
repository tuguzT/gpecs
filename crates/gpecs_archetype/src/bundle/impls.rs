use gpecs_component::{
    Component,
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
    soa::{
        identity::Identity,
        traits::{TupleHelper, count_idents},
    },
};

use crate::bundle::{Bundle, BundleMutPtrs, BundlePtrs};

unsafe impl<T> Bundle for Identity<T>
where
    T: Component,
{
    const CONTEXT: &'static Self::Context = &();

    type GetComponents = [Option<ComponentId>; 1];

    #[inline]
    fn get_components<U>(components: &ComponentRegistryView<impl Sized, U>) -> Self::GetComponents
    where
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = components.component_id::<T>();
        [component_id]
    }

    type RegisterComponents = [ComponentId; 1];

    #[inline]
    fn register_components<U, M>(
        components: &mut ComponentRegistry<U, M>,
    ) -> Self::RegisterComponents
    where
        U: PushBackArray<Item: FromComponentType>,
        M: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
    {
        let component_id = components.register_component::<T>();
        [component_id]
    }

    #[inline]
    fn ptrs_from_erased<I, U, P>(
        components: &ComponentRegistryView<impl Sized, U>,
        iter: I,
    ) -> Result<BundlePtrs<Self>, DowncastErrorKind>
    where
        I: IntoIterator<Item = ErasedComponentPtr<P>>,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
        P: ConstSliceItemPtr,
    {
        let component_id = components
            .component_id::<T>()
            .ok_or_else(NotRegisteredError::of::<T>)?;
        let ptr = iter
            .into_iter()
            .find(|ptr| ptr.component_id() == component_id)
            .ok_or_else(NotRegisteredError::of::<T>)?;

        let ptr = ptr.downcast::<T, U>(components)?.cast();
        Ok(ptr)
    }

    #[inline]
    fn mut_ptrs_from_erased<I, U, P>(
        components: &ComponentRegistryView<impl Sized, U>,
        iter: I,
    ) -> Result<BundleMutPtrs<Self>, DowncastErrorKind>
    where
        I: IntoIterator<Item = ErasedComponentMutPtr<P>>,
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
        P: MutSliceItemPtr,
    {
        let component_id = components
            .component_id::<T>()
            .ok_or_else(NotRegisteredError::of::<T>)?;
        let ptr = iter
            .into_iter()
            .find(|ptr| ptr.component_id() == component_id)
            .ok_or_else(NotRegisteredError::of::<T>)?;

        let ptr = ptr.downcast::<T, U>(components)?.cast();
        Ok(ptr)
    }
}

macro_rules! bundle_tuple_impl {
    ($($types:ident index $indices:tt),* $(,)?) => {
        unsafe impl<$($types,)*> Bundle for ($($types,)*)
        where
            $($types: Component,)*
        {
            const CONTEXT: &'static Self::Context = &();

            type GetComponents = [Option<ComponentId>; count_idents!($($types,)*)];

            #[inline]
            fn get_components<U>(components: &ComponentRegistryView<impl Sized, U>) -> Self::GetComponents
            where
                U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
            {
                let permutation = TupleHelper::<($($types,)*)>::PERMUTATION;

                let component_ids = [$(components.component_id::<$types>(),)*];
                let component_ids = [$(component_ids[permutation[$indices]],)*];
                component_ids
            }

            type RegisterComponents = [ComponentId; count_idents!($($types,)*)];

            #[inline]
            fn register_components<U, M>(
                components: &mut ComponentRegistry<U, M>,
            ) -> Self::RegisterComponents
            where
                U: PushBackArray<Item: FromComponentType>,
                M: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
            {
                let permutation = TupleHelper::<($($types,)*)>::PERMUTATION;

                let component_ids = [$(components.register_component::<$types>(),)*];
                let component_ids = [$(component_ids[permutation[$indices]],)*];
                component_ids
            }

            #[inline]
            fn ptrs_from_erased<Iter, U, P>(
                components: &ComponentRegistryView<impl Sized, U>,
                iter: Iter,
            ) -> Result<BundlePtrs<Self>, DowncastErrorKind>
            where
                Iter: IntoIterator<Item = ErasedComponentPtr<P>>,
                U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
                P: ConstSliceItemPtr,
            {
                let component_ids = [$(components.component_id::<$types>().ok_or_else(NotRegisteredError::of::<$types>)?,)*];

                let mut ptrs = ($(None::<*const $types>,)*);
                #[expect(clippy::needless_continue)]
                for ptr in iter {
                    $(
                        if ptrs.$indices.is_none() && ptr.component_id() == component_ids[$indices] {
                            ptrs.$indices = Some(ptr.downcast(components)?);
                            continue;
                        }
                    )*
                }

                let ptrs = ($(ptrs.$indices.ok_or_else(NotRegisteredError::of::<$types>)?,)*);
                Ok(ptrs)
            }

            #[inline]
            fn mut_ptrs_from_erased<Iter, U, P>(
                components: &ComponentRegistryView<impl Sized, U>,
                iter: Iter,
            ) -> Result<BundleMutPtrs<Self>, DowncastErrorKind>
            where
                Iter: IntoIterator<Item = ErasedComponentMutPtr<P>>,
                U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
                P: MutSliceItemPtr,
            {
                let component_ids = [$(components.component_id::<$types>().ok_or_else(NotRegisteredError::of::<$types>)?,)*];

                let mut ptrs = ($(None::<*mut $types>,)*);
                #[expect(clippy::needless_continue)]
                for ptr in iter {
                    $(
                        if ptrs.$indices.is_none() && ptr.component_id() == component_ids[$indices] {
                            ptrs.$indices = Some(ptr.downcast(components)?);
                            continue;
                        }
                    )*
                }

                let ptrs = ($(ptrs.$indices.ok_or_else(NotRegisteredError::of::<$types>)?,)*);
                Ok(ptrs)
            }
        }
    };
}

bundle_tuple_impl!(
    A index 0,
);

bundle_tuple_impl!(
    A index 0,
    B index 1,
);

bundle_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
);

bundle_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
);

bundle_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
);

bundle_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
);

bundle_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
);

bundle_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
);

bundle_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
    I index 8,
);

bundle_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
    I index 8,
    J index 9,
);

bundle_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
    I index 8,
    J index 9,
    K index 10,
);

bundle_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
    I index 8,
    J index 9,
    K index 10,
    L index 11,
);
