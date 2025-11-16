use gpecs_soa_erased::field::ErasedFieldMutPtr;

use crate::{
    component::{
        Component,
        registry::{ComponentId, ComponentRegistry},
    },
    soa::{
        identity::{Identity, IdentityContext},
        traits::{MutPtrs, TupleContext, impls::count_idents},
    },
};

use super::Bundle;

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
    unsafe fn ptrs_from_iter<I>(components: &ComponentRegistry, iter: I) -> MutPtrs<'static, Self>
    where
        I: IntoIterator<Item = (ComponentId, ErasedFieldMutPtr)>,
    {
        let component_id = unsafe { components.component_id::<T>().unwrap_unchecked() };
        let (_, ptr) = unsafe {
            iter.into_iter()
                .find(|(id, _)| *id == component_id)
                .unwrap_unchecked()
        };
        unsafe { ptr.into().unwrap_unchecked() }
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
            unsafe fn ptrs_from_iter<Iter>(components: &ComponentRegistry, iter: Iter) -> MutPtrs<'static, Self>
            where
                Iter: IntoIterator<Item = (ComponentId, ErasedFieldMutPtr)>,
            {
                let component_ids = [$(unsafe { components.component_id::<$types>().unwrap_unchecked() },)*];

                let mut ptrs = ($(None::<*mut $types>,)*);
                for (id, ptr) in iter {
                    $(
                        if id == component_ids[$indices] {
                            ptrs.$indices = Some(unsafe { ptr.into().unwrap_unchecked() });
                        }
                    )*
                }
                ($(unsafe { ptrs.$indices.unwrap_unchecked() },)*)
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
