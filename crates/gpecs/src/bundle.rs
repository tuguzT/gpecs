use std::{
    error::Error,
    fmt::{self, Display},
    result,
};

use crate::{
    component::{Component, ComponentId, ComponentRegistry},
    soa::traits::Soa,
};

pub type Result<T> = result::Result<T, DuplicateComponentError>;

#[allow(unsafe_code)]
pub unsafe trait Bundle: Soa + 'static {
    type ComponentIds: IntoIterator<Item = ComponentId>;

    /// Order of component identifiers should be the same as
    /// the order of layouts returned by [`Soa::field_layouts()`] method.
    fn component_ids(
        context: &Self::Context,
        components: &mut ComponentRegistry,
    ) -> Result<Self::ComponentIds>;
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct DuplicateComponentError {
    component_id: ComponentId,
}

impl DuplicateComponentError {
    #[inline]
    pub fn new(component_id: ComponentId) -> Self {
        Self { component_id }
    }

    #[inline]
    pub fn component_id(&self) -> ComponentId {
        let Self { component_id } = *self;
        component_id
    }
}

impl Display for DuplicateComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { component_id } = *self;
        write!(f, "duplicate component {component_id:?} were found")
    }
}

impl Error for DuplicateComponentError {}

#[allow(unsafe_code)]
unsafe impl Bundle for () {
    type ComponentIds = [ComponentId; 1];

    #[inline]
    fn component_ids(
        _: &Self::Context,
        components: &mut ComponentRegistry,
    ) -> Result<Self::ComponentIds> {
        let component_ids = [components.register_component::<Self>()];
        Ok(component_ids)
    }
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
            $($types: Component,)*
        {
            type ComponentIds = [ComponentId; $crate::soa::traits::count_idents!($($types,)*)];

            #[inline]
            fn component_ids(
                _: &Self::Context,
                components: &mut ComponentRegistry,
            ) -> Result<Self::ComponentIds> {
                let component_ids = [$(components.register_component::<$types>(),)*];
                if let Some(component_id) = find_first_duplicate(component_ids) {
                    return Err(DuplicateComponentError::new(component_id));
                }

                let permutation = $crate::soa::traits::SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;
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
