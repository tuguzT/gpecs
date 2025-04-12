use crate::{
    component::registry::{ComponentId, ComponentRegistry},
    soa::traits::Soa,
};

use self::error::{ComponentNotRegisteredError, DuplicateComponentError, GetComponentsError};

pub mod error;

/// Non-empty collection of [components](crate::component::Component).
#[allow(unsafe_code)]
pub unsafe trait Bundle: Soa + 'static {
    /// Order of component identifiers should be the same as
    /// the order of corresponding [descriptors](Soa::FieldDescriptors).
    type ComponentIds: IntoIterator<Item = ComponentId>;

    fn get_components(
        context: &Self::Context,
        components: &ComponentRegistry,
    ) -> Result<<Self as Bundle>::ComponentIds, GetComponentsError>;

    fn register_components(
        context: &Self::Context,
        components: &mut ComponentRegistry,
    ) -> Result<<Self as Bundle>::ComponentIds, DuplicateComponentError>;
}

#[inline]
fn find_first_duplicate<const N: usize>(
    mut component_ids: [ComponentId; N],
) -> Option<ComponentId> {
    component_ids.sort_unstable();

    for pair in component_ids.windows(2) {
        let [first, second] = pair else {
            unreachable!("should contain exactly two elements")
        };
        if first == second {
            return Some(*first);
        }
    }
    None
}

macro_rules! bundle_tuple_impl {
    ($($types:ident index $indices:tt),* $(,)?) => {
        #[allow(unsafe_code)]
        unsafe impl<$($types,)*> Bundle for ($($types,)*)
        where
            $($types: $crate::component::Component,)*
        {
            type ComponentIds = [ComponentId; $crate::soa::traits::impls::count_idents!($($types,)*)];

            #[inline]
            fn get_components(
                _: &Self::Context,
                components: &ComponentRegistry,
            ) -> Result<<Self as Bundle>::ComponentIds, GetComponentsError> {
                let component_ids = [$(
                    components
                        .component_id::<$types>()
                        .ok_or_else(|| ComponentNotRegisteredError::of::<$types>())?,
                )*];
                if let Some(component_id) = find_first_duplicate(component_ids) {
                    let error = DuplicateComponentError::new(component_id);
                    return Err(error.into());
                }

                let permutation = $crate::soa::traits::impls::SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;
                let component_ids = [$(component_ids[permutation[$indices]],)*];
                Ok(component_ids)
            }

            #[inline]
            fn register_components(
                _: &Self::Context,
                components: &mut ComponentRegistry,
            ) -> Result<<Self as Bundle>::ComponentIds, DuplicateComponentError> {
                let component_ids = [$(components.register_component::<$types>(),)*];
                if let Some(component_id) = find_first_duplicate(component_ids) {
                    let error = DuplicateComponentError::new(component_id);
                    return Err(error);
                }

                let permutation = $crate::soa::traits::impls::SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;
                let component_ids = [$(component_ids[permutation[$indices]],)*];
                Ok(component_ids)
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
