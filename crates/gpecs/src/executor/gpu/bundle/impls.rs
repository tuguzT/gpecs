use crate::{
    archetype::erased::error::{ArchetypeError, DuplicateComponentError},
    bundle::Bundle,
    component::erased::error::NotRegisteredError,
    context::Components,
    executor::gpu::{
        bundle::GpuBundle,
        component::{
            GpuComponent,
            registry::{GpuComponentId, GpuComponentRegistry},
        },
    },
    soa::{count_idents, identity::Identity, traits::TupleHelper},
};

unsafe impl<T> GpuBundle for Identity<T>
where
    T: GpuComponent,
{
    type GpuComponents = [GpuComponentId; 1];

    #[inline]
    fn get_gpu_components(
        components: &Components,
        gpu_components: &GpuComponentRegistry,
    ) -> Result<Self::GpuComponents, ArchetypeError> {
        let [component_id] = Self::get_components(&components.as_view())?
            .map(|id| gpu_components.map_component_id(id));
        let component_ids = [component_id.ok_or_else(NotRegisteredError::of::<T>)?];
        Ok(component_ids)
    }

    #[inline]
    fn register_gpu_components(
        components: &mut Components,
        gpu_components: &mut GpuComponentRegistry,
    ) -> Result<Self::GpuComponents, DuplicateComponentError> {
        Self::register_components(components)?;

        let component_id = gpu_components.register_component::<T>(components);
        let component_ids = [component_id];
        Ok(component_ids)
    }
}

macro_rules! gpu_bundle_tuple_impl {
    ($($types:ident index $indices:tt),* $(,)?) => {
        unsafe impl<$($types,)*> GpuBundle for ($($types,)*)
        where
            $($types: GpuComponent,)*
        {
            type GpuComponents = [GpuComponentId; count_idents!($($types,)*)];

            #[inline]
            fn get_gpu_components(
                components: &Components,
                gpu_components: &GpuComponentRegistry,
            ) -> Result<Self::GpuComponents, ArchetypeError> {
                let component_ids = Self::get_components(&components.as_view())?
                    .map(|id| gpu_components.map_component_id(id));

                let component_ids = [$(component_ids[$indices].ok_or_else(NotRegisteredError::of::<$types>)?,)*];
                Ok(component_ids)
            }

            #[inline]
            fn register_gpu_components(
                components: &mut Components,
                gpu_components: &mut GpuComponentRegistry,
            ) -> Result<Self::GpuComponents, DuplicateComponentError> {
                Self::register_components(components)?;

                let component_ids = [$(gpu_components.register_component::<$types>(components),)*];
                let permutation = TupleHelper::<($($types,)*)>::PERMUTATION;
                let component_ids = [$(component_ids[permutation[$indices]],)*];
                Ok(component_ids)
            }
        }
    };
}

gpu_bundle_tuple_impl!(
    A index 0,
);

gpu_bundle_tuple_impl!(
    A index 0,
    B index 1,
);

gpu_bundle_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
);

gpu_bundle_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
);

gpu_bundle_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
);

gpu_bundle_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
);

gpu_bundle_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
);

gpu_bundle_tuple_impl!(
    A index 0,
    B index 1,
    C index 2,
    D index 3,
    E index 4,
    F index 5,
    G index 6,
    H index 7,
);

gpu_bundle_tuple_impl!(
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

gpu_bundle_tuple_impl!(
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

gpu_bundle_tuple_impl!(
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

gpu_bundle_tuple_impl!(
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
