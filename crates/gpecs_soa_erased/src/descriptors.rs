use crate::soa::{
    field::{FieldDescriptor, FieldDescriptors, FieldDescriptorsOwned},
    identity::Identity,
};

pub trait CovariantFieldDescriptors: FieldDescriptorsOwned {
    /// Restricts [field descriptors](FieldDescriptors::Output)
    /// to be covariant over generic lifetime.
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output;
}

impl<T> CovariantFieldDescriptors for &T
where
    T: CovariantFieldDescriptors + ?Sized,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        T::upcast_field_descriptors(from)
    }
}

#[cfg(feature = "alloc")]
impl<T> CovariantFieldDescriptors for alloc::boxed::Box<T>
where
    T: CovariantFieldDescriptors + ?Sized,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        T::upcast_field_descriptors(from)
    }
}

impl<T> CovariantFieldDescriptors for Identity<T>
where
    T: CovariantFieldDescriptors + ?Sized,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        T::upcast_field_descriptors(from)
    }
}

impl<T> CovariantFieldDescriptors for [T]
where
    T: AsRef<FieldDescriptor> + 'static,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        from
    }
}

impl<T, const N: usize> CovariantFieldDescriptors for [T; N]
where
    T: AsRef<FieldDescriptor> + 'static,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        from
    }
}

impl<T> CovariantFieldDescriptors for core::slice::Iter<'_, T>
where
    T: AsRef<FieldDescriptor>,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        from
    }
}
