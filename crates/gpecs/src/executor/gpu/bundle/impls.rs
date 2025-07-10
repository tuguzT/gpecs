use crate::{
    component::registry::ComponentRegistry,
    executor::gpu::component::{
        GpuComponent,
        registry::{GpuComponentId, GpuComponentRegistry},
    },
    soa::{
        identity::Identity,
        traits::impls::{SoaTupleImplHelper, count_idents},
    },
};

use super::GpuBundle;

#[allow(unsafe_code)]
unsafe impl<T> GpuBundle for Identity<T>
where
    T: GpuComponent,
{
    type MaybeGpuComponentIds = [Option<GpuComponentId>; 1];

    #[inline]
    fn get_gpu_components(
        components: &ComponentRegistry,
        gpu_components: &GpuComponentRegistry,
    ) -> Self::MaybeGpuComponentIds {
        let component_id = components
            .component_id::<T>()
            .map(|id| gpu_components.map_component_id(id))
            .flatten();
        [component_id]
    }

    type GpuComponentIds = [GpuComponentId; 1];

    #[inline]
    fn register_gpu_components(
        components: &mut ComponentRegistry,
        gpu_components: &mut GpuComponentRegistry,
    ) -> Self::GpuComponentIds {
        let component_id = gpu_components.register_component::<T>(components);
        [component_id]
    }
}

macro_rules! gpu_bundle_tuple_impl {
    ($($types:ident index $indices:tt),* $(,)?) => {
        #[allow(unsafe_code)]
        unsafe impl<$($types,)*> GpuBundle for ($($types,)*)
        where
            $($types: GpuComponent,)*
        {
            type MaybeGpuComponentIds = [Option<GpuComponentId>; count_idents!($($types,)*)];

            #[inline]
            fn get_gpu_components(
                components: &ComponentRegistry,
                gpu_components: &GpuComponentRegistry,
            ) -> Self::MaybeGpuComponentIds {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let component_ids = [$(
                    components
                        .component_id::<$types>()
                        .map(|id| gpu_components.map_component_id(id))
                        .flatten(),
                )*];
                let component_ids = [$(component_ids[permutation[$indices]],)*];
                component_ids
            }

            type GpuComponentIds = [GpuComponentId; count_idents!($($types,)*)];

            #[inline]
            fn register_gpu_components(
                components: &mut ComponentRegistry,
                gpu_components: &mut GpuComponentRegistry,
            ) -> Self::GpuComponentIds {
                let permutation = SoaTupleImplHelper::<($($types,)*)>::PERMUTATION;

                let component_ids = [$(gpu_components.register_component::<$types>(components),)*];
                let component_ids = [$(component_ids[permutation[$indices]],)*];
                component_ids
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
