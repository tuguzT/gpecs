use crate::{
    component::{
        registry::{ComponentId, ComponentRegistry},
        Component,
    },
    soa::{
        identity::Identity,
        traits::impls::{count_idents, SoaTupleImplHelper},
    },
};

use super::Bundle;

#[allow(unsafe_code)]
unsafe impl<T> Bundle for Identity<T>
where
    T: Component,
{
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
}

macro_rules! bundle_tuple_impl {
    ($($types:ident index $indices:tt),* $(,)?) => {
        #[allow(unsafe_code)]
        unsafe impl<$($types,)*> Bundle for ($($types,)*)
        where
            $($types: Component,)*
        {
            type MaybeComponentIds = [Option<ComponentId>; count_idents!($($types,)*)];

            #[inline]
            fn get_components(components: &ComponentRegistry) -> Self::MaybeComponentIds {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let component_ids = [$(components.component_id::<$types>(),)*];
                let component_ids = [$(component_ids[permutation[$indices]],)*];
                component_ids
            }

            type ComponentIds = [ComponentId; count_idents!($($types,)*)];

            #[inline]
            fn register_components(components: &mut ComponentRegistry) -> Self::ComponentIds {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

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
