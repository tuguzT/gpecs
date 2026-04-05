use core::slice;

use crate::soa::{
    field::{FieldDescriptor, FieldDescriptorsOutput, FieldDescriptorsOwned},
    identity::Identity,
};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

pub trait CovariantFieldDescriptors<T = Self>: FieldDescriptorsOwned<T>
where
    T: ?Sized,
{
    /// Restricts [field descriptors](crate::soa::field::FieldDescriptors::Output)
    /// to be covariant over generic lifetime.
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self, T>,
    ) -> FieldDescriptorsOutput<'short, Self, T>;
}

impl<'a, T, U> CovariantFieldDescriptors<&'a U> for &'a T
where
    T: CovariantFieldDescriptors<U> + ?Sized,
    U: ?Sized,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self, &'a U>,
    ) -> FieldDescriptorsOutput<'short, Self, &'a U> {
        T::upcast_field_descriptors(from)
    }
}

#[cfg(feature = "alloc")]
impl<T, U> CovariantFieldDescriptors<Box<U>> for Box<T>
where
    T: CovariantFieldDescriptors<U> + ?Sized,
    U: ?Sized,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self, Box<U>>,
    ) -> FieldDescriptorsOutput<'short, Self, Box<U>> {
        T::upcast_field_descriptors(from)
    }
}

impl<T, U> CovariantFieldDescriptors<Identity<U>> for Identity<T>
where
    T: CovariantFieldDescriptors<U> + ?Sized,
    U: ?Sized,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self, Identity<U>>,
    ) -> FieldDescriptorsOutput<'short, Self, Identity<U>> {
        T::upcast_field_descriptors(from)
    }
}

impl<T> CovariantFieldDescriptors for [T]
where
    T: AsRef<FieldDescriptor> + 'static,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        from
    }
}

impl<T, const N: usize> CovariantFieldDescriptors for [T; N]
where
    T: AsRef<FieldDescriptor> + 'static,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        from
    }
}

impl<T> CovariantFieldDescriptors for slice::Iter<'_, T>
where
    T: AsRef<FieldDescriptor> + 'static,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        from
    }
}
