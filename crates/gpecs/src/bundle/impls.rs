use crate::{
    bundle::{Bundle, BundleMutPtrs, BundlePtrs},
    component::{
        Component,
        erased::{
            ErasedComponent, ErasedComponentMutPtr, ErasedComponentPtr, error::DowncastErrorKind,
        },
        error::NotRegisteredError,
        registry::{ComponentId, ComponentRegistry},
    },
    soa::{
        identity::{Identity, IdentityContext},
        traits::{TupleContext, count_idents},
    },
};

unsafe impl<T> Bundle for Identity<T>
where
    T: Component,
{
    const CONTEXT: &'static Self::Context = &IdentityContext::<T>::new();

    type GetComponents = [Option<ComponentId>; 1];

    #[inline]
    fn get_components(components: &ComponentRegistry) -> Self::GetComponents {
        let component_id = components.component_id::<T>();
        [component_id]
    }

    type RegisterComponents = [ComponentId; 1];

    #[inline]
    fn register_components(components: &mut ComponentRegistry) -> Self::RegisterComponents {
        let component_id = components.register_component::<T>();
        [component_id]
    }

    #[inline]
    fn ptrs_from_erased<I>(
        components: &ComponentRegistry,
        iter: I,
    ) -> Result<BundlePtrs<Self>, DowncastErrorKind>
    where
        I: IntoIterator<Item = ErasedComponentPtr>,
    {
        let component_id = components.component_id::<T>().ok_or(NotRegisteredError)?;
        let ptr = iter
            .into_iter()
            .find(|ptr| ptr.component_id() == component_id)
            .ok_or(NotRegisteredError)?;

        let ptr = ptr.downcast::<T>(components)?.cast();
        Ok(ptr)
    }

    #[inline]
    fn mut_ptrs_from_erased<I>(
        components: &ComponentRegistry,
        iter: I,
    ) -> Result<BundleMutPtrs<Self>, DowncastErrorKind>
    where
        I: IntoIterator<Item = ErasedComponentMutPtr>,
    {
        let component_id = components.component_id::<T>().ok_or(NotRegisteredError)?;
        let ptr = iter
            .into_iter()
            .find(|ptr| ptr.component_id() == component_id)
            .ok_or(NotRegisteredError)?;

        let ptr = ptr.downcast::<T>(components)?.cast();
        Ok(ptr)
    }

    #[inline]
    fn from_erased<I>(components: &ComponentRegistry, iter: I) -> Result<Self, DowncastErrorKind>
    where
        I: IntoIterator<Item = ErasedComponent>,
    {
        let component_id = components.component_id::<T>().ok_or(NotRegisteredError)?;
        let component = iter
            .into_iter()
            .find(|component| component.component_id() == component_id)
            .ok_or(NotRegisteredError)?;

        let component = component.downcast::<T>(components)?;
        Ok(component.into())
    }
}

macro_rules! bundle_tuple_impl {
    ($($types:ident index $indices:tt),* $(,)?) => {
        unsafe impl<$($types,)*> Bundle for ($($types,)*)
        where
            $($types: Component,)*
        {
            const CONTEXT: &'static Self::Context = &TupleContext::<($($types,)*)>::new();

            type GetComponents = [Option<ComponentId>; count_idents!($($types,)*)];

            #[inline]
            fn get_components(components: &ComponentRegistry) -> Self::GetComponents {
                let permutation = Self::Context::PERMUTATION;

                let component_ids = [$(components.component_id::<$types>(),)*];
                let component_ids = [$(component_ids[permutation[$indices]],)*];
                component_ids
            }

            type RegisterComponents = [ComponentId; count_idents!($($types,)*)];

            #[inline]
            fn register_components(components: &mut ComponentRegistry) -> Self::RegisterComponents {
                let permutation = Self::Context::PERMUTATION;

                let component_ids = [$(components.register_component::<$types>(),)*];
                let component_ids = [$(component_ids[permutation[$indices]],)*];
                component_ids
            }

            #[inline]
            fn ptrs_from_erased<Iter>(
                components: &ComponentRegistry,
                iter: Iter,
            ) -> Result<BundlePtrs<Self>, DowncastErrorKind>
            where
                Iter: IntoIterator<Item = ErasedComponentPtr>,
            {
                let component_ids = [$(components.component_id::<$types>().ok_or(NotRegisteredError)?,)*];

                let mut ptrs = ($(None::<*const $types>,)*);
                #[expect(clippy::needless_continue)]
                for ptr in iter {
                    $(
                        if ptrs.$indices.is_none() && ptr.component_id() == component_ids[$indices] {
                            ptrs.$indices = Some(ptr.downcast(components)?);
                            continue;
                        }
                    )*
                }

                let ptrs = ($(ptrs.$indices.ok_or(NotRegisteredError)?,)*);
                Ok(ptrs)
            }

            #[inline]
            fn mut_ptrs_from_erased<Iter>(
                components: &ComponentRegistry,
                iter: Iter,
            ) -> Result<BundleMutPtrs<Self>, DowncastErrorKind>
            where
                Iter: IntoIterator<Item = ErasedComponentMutPtr>,
            {
                let component_ids = [$(components.component_id::<$types>().ok_or(NotRegisteredError)?,)*];

                let mut ptrs = ($(None::<*mut $types>,)*);
                #[expect(clippy::needless_continue)]
                for ptr in iter {
                    $(
                        if ptrs.$indices.is_none() && ptr.component_id() == component_ids[$indices] {
                            ptrs.$indices = Some(ptr.downcast(components)?);
                            continue;
                        }
                    )*
                }

                let ptrs = ($(ptrs.$indices.ok_or(NotRegisteredError)?,)*);
                Ok(ptrs)
            }

            #[inline]
            fn from_erased<Iter>(components: &ComponentRegistry, iter: Iter) -> Result<Self, DowncastErrorKind>
            where
                Iter: IntoIterator<Item = ErasedComponent>,
            {
                let component_ids = [$(components.component_id::<$types>().ok_or(NotRegisteredError)?,)*];

                let mut fields = ($(None::<$types>,)*);
                #[expect(clippy::needless_continue)]
                for field in iter {
                    $(
                        if fields.$indices.is_none() && field.component_id() == component_ids[$indices] {
                            fields.$indices = Some(field.downcast(components)?);
                            continue;
                        }
                    )*
                }

                let fields = ($(fields.$indices.ok_or(NotRegisteredError)?,)*);
                Ok(fields)
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
