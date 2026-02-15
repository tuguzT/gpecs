use std::mem::MaybeUninit;

use gpecs_soa_erased::{
    data::{ErasedMutPtr, ErasedPtr},
    ptr::slice::{ConstSliceItemPtr, MutSliceItemPtr},
};

use crate::{
    bundle::{Bundle, error::PtrsFromIterError},
    component::{
        Component,
        error::NotRegisteredError,
        registry::{ComponentId, ComponentRegistry},
    },
    soa::{
        identity::{Identity, IdentityContext},
        traits::{MutPtrs, Ptrs, TupleContext, count_idents},
    },
};

unsafe impl<T> Bundle for Identity<T>
where
    T: Component,
{
    const CONTEXT: &'static Self::Context = &IdentityContext::<T>::new();

    type MaybeComponentIds = [Option<ComponentId>; 1];

    #[inline]
    fn get_components(components: &ComponentRegistry) -> Self::MaybeComponentIds {
        let component_id = components.component_id::<T>();
        [component_id]
    }

    type ComponentIds = [ComponentId; 1];

    #[inline]
    fn register_components(components: &mut ComponentRegistry) -> Self::ComponentIds {
        let component_id = components.register_component::<T>();
        [component_id]
    }

    #[inline]
    fn ptrs_from_erased<I, P>(
        components: &ComponentRegistry,
        iter: I,
    ) -> Result<Ptrs<'static, Self>, PtrsFromIterError<ErasedPtr<P>>>
    where
        I: IntoIterator<Item = (ComponentId, ErasedPtr<P>)>,
        P: ConstSliceItemPtr<Item = MaybeUninit<u8>>,
    {
        let component_id = components.component_id::<T>().ok_or(NotRegisteredError)?;
        let (_, ptr) = iter
            .into_iter()
            .find(|(id, _)| *id == component_id)
            .ok_or(NotRegisteredError)?;

        let ptr = ptr.try_into()?;
        Ok(ptr)
    }

    #[inline]
    fn mut_ptrs_from_erased<I, P>(
        components: &ComponentRegistry,
        iter: I,
    ) -> Result<MutPtrs<'static, Self>, PtrsFromIterError<ErasedMutPtr<P>>>
    where
        I: IntoIterator<Item = (ComponentId, ErasedMutPtr<P>)>,
        P: MutSliceItemPtr<Item = MaybeUninit<u8>>,
    {
        let component_id = components.component_id::<T>().ok_or(NotRegisteredError)?;
        let (_, ptr) = iter
            .into_iter()
            .find(|(id, _)| *id == component_id)
            .ok_or(NotRegisteredError)?;

        let ptr = ptr.try_into()?;
        Ok(ptr)
    }
}

macro_rules! bundle_tuple_impl {
    ($($types:ident index $indices:tt),* $(,)?) => {
        unsafe impl<$($types,)*> Bundle for ($($types,)*)
        where
            $($types: Component,)*
        {
            const CONTEXT: &'static Self::Context = &TupleContext::<($($types,)*)>::new();

            type MaybeComponentIds = [Option<ComponentId>; count_idents!($($types,)*)];

            #[inline]
            fn get_components(components: &ComponentRegistry) -> Self::MaybeComponentIds {
                let permutation = Self::Context::PERMUTATION;

                let component_ids = [$(components.component_id::<$types>(),)*];
                let component_ids = [$(component_ids[permutation[$indices]],)*];
                component_ids
            }

            type ComponentIds = [ComponentId; count_idents!($($types,)*)];

            #[inline]
            fn register_components(components: &mut ComponentRegistry) -> Self::ComponentIds {
                let permutation = Self::Context::PERMUTATION;

                let component_ids = [$(components.register_component::<$types>(),)*];
                let component_ids = [$(component_ids[permutation[$indices]],)*];
                component_ids
            }

            #[inline]
            fn ptrs_from_erased<Iter, P>(
                components: &ComponentRegistry,
                iter: Iter,
            ) -> Result<Ptrs<'static, Self>, PtrsFromIterError<ErasedPtr<P>>>
            where
                Iter: IntoIterator<Item = (ComponentId, ErasedPtr<P>)>,
                P: ConstSliceItemPtr<Item = MaybeUninit<u8>>,
            {
                let component_ids = [$(components.component_id::<$types>().ok_or(NotRegisteredError)?,)*];

                let mut ptrs = ($(None::<*const $types>,)*);
                #[expect(clippy::needless_continue)]
                for (id, ptr) in iter {
                    $(
                        if ptrs.$indices.is_none() && id == component_ids[$indices] {
                            ptrs.$indices = Some(ptr.try_into()?);
                            continue;
                        }
                    )*
                }

                let ptrs = ($(ptrs.$indices.ok_or(NotRegisteredError)?,)*);
                Ok(ptrs)
            }

            #[inline]
            fn mut_ptrs_from_erased<Iter, P>(
                components: &ComponentRegistry,
                iter: Iter,
            ) -> Result<MutPtrs<'static, Self>, PtrsFromIterError<ErasedMutPtr<P>>>
            where
                Iter: IntoIterator<Item = (ComponentId, ErasedMutPtr<P>)>,
                P: MutSliceItemPtr<Item = MaybeUninit<u8>>,
            {
                let component_ids = [$(components.component_id::<$types>().ok_or(NotRegisteredError)?,)*];

                let mut ptrs = ($(None::<*mut $types>,)*);
                #[expect(clippy::needless_continue)]
                for (id, ptr) in iter {
                    $(
                        if ptrs.$indices.is_none() && id == component_ids[$indices] {
                            ptrs.$indices = Some(ptr.try_into()?);
                            continue;
                        }
                    )*
                }

                let ptrs = ($(ptrs.$indices.ok_or(NotRegisteredError)?,)*);
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
