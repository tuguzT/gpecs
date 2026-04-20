use core::{alloc::Layout, slice};

#[cfg(feature = "alloc")]
use core_alloc::boxed::Box;

use crate::layout::{FfiLayout, WithLayout};

/// Descriptor for any field type used by [SoA](crate::traits::RawSoa) types.
///
/// For now this contains only a [`Layout`] of such field.
/// Some additional data may be added in the future.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FieldDescriptor {
    layout: FfiLayout,
}

impl FieldDescriptor {
    /// Creates a new field descriptor from the given [`Layout`].
    #[inline]
    pub const fn new(layout: Layout) -> Self {
        let layout = FfiLayout::new(layout);
        Self { layout }
    }

    /// Creates a new field descriptor from the given type.
    #[inline]
    pub const fn of<T>() -> Self {
        let layout = Layout::new::<T>();
        Self::new(layout)
    }

    /// Returns the [`Layout`] of this field descriptor.
    #[inline]
    pub const fn layout(self) -> Layout {
        let Self { layout } = self;
        layout.layout()
    }
}

impl AsRef<Self> for FieldDescriptor {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<Self> for FieldDescriptor {
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl From<FieldDescriptor> for Layout {
    #[inline]
    fn from(descriptor: FieldDescriptor) -> Self {
        descriptor.layout()
    }
}

impl WithLayout for FieldDescriptor {
    #[inline]
    fn layout(&self) -> Layout {
        (*self).into()
    }
}

/// Alias for the [field descriptors](FieldDescriptors::Output) of some type `T`.
pub type FieldDescriptorsOutput<'a, T, U = T> = <T as FieldDescriptors<'a, U>>::Output;

/// Alias for an iterator of the [field descriptors](FieldDescriptors::Output) of some type `T`.
pub type FieldDescriptorsIter<'a, T, U = T> =
    <FieldDescriptorsOutput<'a, T, U> as IntoIterator>::IntoIter;

/// Alias for an iterator item of the [field descriptors](FieldDescriptors::Output) of some type `T`.
pub type FieldDescriptorsItem<'a, T, U = T> =
    <FieldDescriptorsOutput<'a, T, U> as IntoIterator>::Item;

/// Used to retrieve a non-owning collection of field descriptors.
pub trait FieldDescriptors<'a, T = Self>
where
    T: ?Sized,
{
    /// Collection of items which could be converted into a [field descriptor](FieldDescriptor).
    type Output: IntoIterator<Item: AsRef<FieldDescriptor>> + 'a;

    /// Returns [field descriptors](FieldDescriptors::Output) from self.
    fn field_descriptors(&'a self) -> Self::Output;
}

impl<'a, T, U> FieldDescriptors<'a, &U> for &T
where
    T: FieldDescriptors<'a, U> + ?Sized,
    U: ?Sized,
{
    type Output = T::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        (**self).field_descriptors()
    }
}

#[cfg(feature = "alloc")]
impl<'a, T, U> FieldDescriptors<'a, Box<U>> for Box<T>
where
    T: FieldDescriptors<'a, U> + ?Sized,
    U: ?Sized,
{
    type Output = T::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        (**self).field_descriptors()
    }
}

impl<'a, T> FieldDescriptors<'a> for [T]
where
    T: AsRef<FieldDescriptor> + 'a,
{
    type Output = &'a [T];

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self
    }
}

impl<'a, T, const N: usize> FieldDescriptors<'a> for [T; N]
where
    T: AsRef<FieldDescriptor> + 'a,
{
    type Output = &'a [T; N];

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self
    }
}

impl<'a, T> FieldDescriptors<'a> for slice::Iter<'_, T>
where
    T: AsRef<FieldDescriptor> + 'a,
{
    type Output = slice::Iter<'a, T>;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.clone()
    }
}

/// Used to retrieve a non-owning collection of field descriptors for any lifetime.
pub trait FieldDescriptorsOwned<T = Self>: for<'a> FieldDescriptors<'a, T>
where
    T: ?Sized,
{
}

impl<T, U> FieldDescriptorsOwned<U> for T
where
    T: for<'a> FieldDescriptors<'a, U> + ?Sized,
    U: ?Sized,
{
}
