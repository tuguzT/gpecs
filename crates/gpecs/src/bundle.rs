use gpecs_sparse::soa::Soa;

use crate::component::{Component, ComponentId, ComponentRegistry};

#[allow(unsafe_code)]
pub unsafe trait Bundle: Soa + 'static {
    // order of component ids should be the same as the order of layouts returned by `field_layouts` method
    fn component_ids(components: &mut ComponentRegistry) -> impl IntoIterator<Item = ComponentId>;
}

#[allow(unsafe_code)]
unsafe impl Bundle for () {
    fn component_ids(components: &mut ComponentRegistry) -> impl IntoIterator<Item = ComponentId> {
        [components.register_component::<Self>()]
    }
}

macro_rules! bundle_tuple_impl {
    ($($types:ident index $indices:tt),* $(,)?) => {
        #[allow(unsafe_code)]
        unsafe impl<$($types,)*> Bundle for ($($types,)*)
        where
            $($types: Component,)*
        {
            fn component_ids(components: &mut ComponentRegistry) -> impl IntoIterator<Item = ComponentId> {
                let permutation = $crate::soa::traits::SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let component_ids = [$(components.register_component::<$types>(),)*];
                [$(component_ids[permutation[$indices]],)*]
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
