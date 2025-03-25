use core::alloc::Layout;

#[derive(Debug, Clone, Copy)]
pub struct FieldDescriptor {
    layout: Layout,
}

impl FieldDescriptor {
    #[inline]
    pub fn new(layout: Layout) -> Self {
        Self { layout }
    }

    #[inline]
    pub const fn of<T>() -> Self {
        let layout = Layout::new::<T>();
        Self { layout }
    }

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
