use gpecs_component::{
    Component,
    erased::{
        ErasedComponentMutPtr, ErasedComponentPtr,
        error::{DowncastError as ComponentDowncastError, NotRegisteredError},
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

use crate::{
    bundle::{Bundle, BundleMutPtrs, BundlePtrs, error::DowncastError},
    erased::error::{ArchetypeError, DuplicateComponentError},
};

unsafe impl<T> Bundle for Identity<T>
where
    T: Component,
{
    const CONTEXT: &'static Self::Context = &();

    type Components = [ComponentId; 1];

    #[inline]
    fn get_components<U>(
        components: &ComponentRegistryView<impl Sized, U>,
    ) -> Result<Self::Components, ArchetypeError>
    where
        U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
    {
        let component_id = components
            .component_id::<T>()
            .ok_or_else(NotRegisteredError::of::<T>)?;
        let component_ids = [component_id];
        Ok(component_ids)
    }

    #[inline]
    fn register_components<U, M>(
        components: &mut ComponentRegistry<U, M>,
    ) -> Result<Self::Components, DuplicateComponentError>
    where
        U: PushBackArray<Item: FromComponentType>,
        M: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
    {
        let component_id = components.register_component::<T>();
        let component_ids = [component_id];
        Ok(component_ids)
    }

    #[inline]
    fn ptrs_from_erased<I, U, P>(
        components: &ComponentRegistryView<impl Sized, U>,
        iter: I,
    ) -> Result<BundlePtrs<Self>, DowncastError>
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

        let ptr = ptr
            .downcast::<T, U>(components)
            .map_err(ComponentDowncastError::into_source)?
            .cast();
        Ok(ptr)
    }

    #[inline]
    fn mut_ptrs_from_erased<I, U, P>(
        components: &ComponentRegistryView<impl Sized, U>,
        iter: I,
    ) -> Result<BundleMutPtrs<Self>, DowncastError>
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

        let ptr = ptr
            .downcast::<T, U>(components)
            .map_err(ComponentDowncastError::into_source)?
            .cast();
        Ok(ptr)
    }
}

#[inline]
fn find_duplicate_components(component_ids: &[ComponentId]) -> Result<(), DuplicateComponentError> {
    for i in 1..component_ids.len() {
        let component_id = component_ids[i];
        if component_ids[..i].contains(&component_id) {
            let error = DuplicateComponentError::new(component_id);
            return Err(error);
        }
    }
    Ok(())
}

macro_rules! bundle_tuple_impl {
    ($($types:ident index $indices:tt),* $(,)?) => {
        unsafe impl<$($types,)*> Bundle for ($($types,)*)
        where
            $($types: Component,)*
        {
            const CONTEXT: &'static Self::Context = &();

            type Components = [ComponentId; count_idents!($($types,)*)];

            #[inline]
            fn get_components<U>(
                components: &ComponentRegistryView<impl Sized, U>,
            ) -> Result<Self::Components, ArchetypeError>
            where
                U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
            {
                let component_ids = [$(components.component_id::<$types>().ok_or_else(NotRegisteredError::of::<$types>)?,)*];
                find_duplicate_components(&component_ids)?;

                let permutation = TupleHelper::<($($types,)*)>::PERMUTATION;
                let component_ids = [$(component_ids[permutation[$indices]],)*];
                Ok(component_ids)
            }

            #[inline]
            fn register_components<U, M>(
                components: &mut ComponentRegistry<U, M>,
            ) -> Result<Self::Components, DuplicateComponentError>
            where
                U: PushBackArray<Item: FromComponentType>,
                M: ComponentIdFromOrInsertWith<Key: FromComponentType> + ?Sized,
            {
                let component_ids = [$(components.register_component::<$types>(),)*];
                find_duplicate_components(&component_ids)?;

                let permutation = TupleHelper::<($($types,)*)>::PERMUTATION;
                let component_ids = [$(component_ids[permutation[$indices]],)*];
                Ok(component_ids)
            }

            #[inline]
            fn ptrs_from_erased<Iter, U, P>(
                components: &ComponentRegistryView<impl Sized, U>,
                iter: Iter,
            ) -> Result<BundlePtrs<Self>, DowncastError>
            where
                Iter: IntoIterator<Item = ErasedComponentPtr<P>>,
                U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
                P: ConstSliceItemPtr,
            {
                let component_ids = [$(components.component_id::<$types>().ok_or_else(NotRegisteredError::of::<$types>)?,)*];
                find_duplicate_components(&component_ids)?;

                let mut ptrs = ($(None::<*const $types>,)*);
                #[expect(clippy::needless_continue)]
                for ptr in iter {
                    $(
                        if ptrs.$indices.is_none() && ptr.component_id() == component_ids[$indices] {
                            ptrs.$indices = Some(ptr.downcast(components).map_err(ComponentDowncastError::into_source)?);
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
            ) -> Result<BundleMutPtrs<Self>, DowncastError>
            where
                Iter: IntoIterator<Item = ErasedComponentMutPtr<P>>,
                U: ComponentIdFrom<Key: FromComponentType> + ?Sized,
                P: MutSliceItemPtr,
            {
                let component_ids = [$(components.component_id::<$types>().ok_or_else(NotRegisteredError::of::<$types>)?,)*];
                find_duplicate_components(&component_ids)?;

                let mut ptrs = ($(None::<*mut $types>,)*);
                #[expect(clippy::needless_continue)]
                for ptr in iter {
                    $(
                        if ptrs.$indices.is_none() && ptr.component_id() == component_ids[$indices] {
                            ptrs.$indices = Some(ptr.downcast(components).map_err(ComponentDowncastError::into_source)?);
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
