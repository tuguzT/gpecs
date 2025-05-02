use core::alloc::Layout;

/// Descriptor for any field type used by [`Soa`](crate::traits::Soa) trait.
///
/// For now this contains only a [`Layout`] of such field.
/// Some additional data may be added in the future.
#[derive(Debug, Clone, Copy)]
pub struct FieldDescriptor {
    layout: Layout,
}

impl FieldDescriptor {
    /// Creates a new field descriptor from the given [`Layout`].
    #[inline]
    pub const fn new(layout: Layout) -> Self {
        Self { layout }
    }

    /// Creates a new field descriptor from the given type.
    #[inline]
    pub const fn of<T>() -> Self {
        let layout = Layout::new::<T>();
        Self { layout }
    }

    /// Returns the [`Layout`] of this field descriptor.
    #[inline]
    pub const fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }
}

impl AsRef<FieldDescriptor> for FieldDescriptor {
    #[inline]
    fn as_ref(&self) -> &FieldDescriptor {
        self
    }
}

impl AsMut<FieldDescriptor> for FieldDescriptor {
    #[inline]
    fn as_mut(&mut self) -> &mut FieldDescriptor {
        self
    }
}
