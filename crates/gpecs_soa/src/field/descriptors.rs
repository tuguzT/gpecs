use core::{alloc::Layout, num::NonZeroUsize, slice};

/// Descriptor for any field type used by [SoA](crate::traits::RawSoa) types.
///
/// For now this contains only a [`Layout`] of such field.
/// Some additional data may be added in the future.
#[derive(Debug, Clone, Copy)]
#[repr(C)] // should be just `Layout`, but it is not `repr(C)`
pub struct FieldDescriptor {
    size: usize,
    align: NonZeroUsize,
}

impl FieldDescriptor {
    /// Creates a new field descriptor from the given [`Layout`].
    #[inline]
    pub const fn new(layout: Layout) -> Self {
        let size = layout.size();
        // SAFETY: Layout::align() is guaranteed to be a power of two, which is non-zero
        let align = unsafe { NonZeroUsize::new_unchecked(layout.align()) };
        Self { size, align }
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
        let Self { size, align } = self;
        // SAFETY: self could only be created from a valid `Layout`
        unsafe { Layout::from_size_align_unchecked(size, align.get()) }
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

/// Alias for the [field descriptors](FieldDescriptors::Output) of some type `T`.
pub type FieldDescriptorsOutput<'a, T> = <T as FieldDescriptors<'a>>::Output;

/// Alias for an iterator of the [field descriptors](FieldDescriptors::Output) of some type `T`.
pub type FieldDescriptorsIter<'a, T> = <FieldDescriptorsOutput<'a, T> as IntoIterator>::IntoIter;

/// Used to retrieve a non-owning collection of field descriptors.
pub trait FieldDescriptors<'a> {
    /// Collection of items which could be converted into a [field descriptor](FieldDescriptor).
    type Output: IntoIterator<Item: AsRef<FieldDescriptor>>;

    /// Returns [field descriptors](FieldDescriptors::Output) from self.
    fn field_descriptors(&'a self) -> Self::Output;
}

impl<'a, T> FieldDescriptors<'a> for &T
where
    T: FieldDescriptors<'a> + ?Sized,
{
    type Output = T::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        (**self).field_descriptors()
    }
}

#[cfg(feature = "alloc")]
impl<'a, T> FieldDescriptors<'a> for core_alloc::boxed::Box<T>
where
    T: FieldDescriptors<'a> + ?Sized,
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
    T: AsRef<FieldDescriptor>,
{
    type Output = Self;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        self.clone()
    }
}

/// Used to retrieve a non-owning collection of field descriptors for any lifetime.
pub trait FieldDescriptorsOwned: for<'a> FieldDescriptors<'a> {}

impl<T> FieldDescriptorsOwned for T where T: for<'a> FieldDescriptors<'a> + ?Sized {}
