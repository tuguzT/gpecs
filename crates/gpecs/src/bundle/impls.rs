use crate::component::registry::{ComponentId, ComponentRegistry};

use super::Bundle;

macro_rules! bundle_tuple_impl {
    ($($types:ident index $indices:tt),* $(,)?) => {
        #[allow(unsafe_code)]
        unsafe impl<$($types,)*> Bundle for ($($types,)*)
        where
            $($types: $crate::component::Component,)*
        {
            type MaybeComponentIds = [Option<ComponentId>; $crate::soa::traits::impls::count_idents!($($types,)*)];
            type ComponentIds = [ComponentId; $crate::soa::traits::impls::count_idents!($($types,)*)];

            #[inline]
            fn get_components(
                _: &Self::Context,
                components: &ComponentRegistry,
            ) -> Self::MaybeComponentIds {
                let permutation = $crate::soa::traits::impls::SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let component_ids = [$(components.component_id::<$types>(),)*];
                let component_ids = [$(component_ids[permutation[$indices]],)*];
                component_ids
            }

            #[inline]
            fn register_components(
                _: &Self::Context,
                components: &mut ComponentRegistry,
            ) -> Self::ComponentIds {
                let permutation = $crate::soa::traits::impls::SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let component_ids = [$(components.register_component::<$types>(),)*];
                let component_ids = [$(component_ids[permutation[$indices]],)*];
                component_ids
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
