use core::{alloc::Layout, mem, ptr};

use super::debug_assert_ptr_is_aligned;

pub type DropFn = unsafe fn(to_drop: *mut u8);

#[derive(Debug, Clone, Copy)]
pub struct FieldDescriptor {
    layout: Layout,
    drop_fn: Option<DropFn>,
}

impl FieldDescriptor {
    #[inline]
    pub fn new<D>(layout: Layout, drop_fn: D) -> Self
    where
        D: Into<Option<DropFn>>,
    {
        Self {
            layout,
            drop_fn: drop_fn.into(),
        }
    }

    #[inline]
    pub const fn of<T>() -> Self {
        let layout = Layout::new::<T>();
        let drop_fn = if mem::needs_drop::<T>() {
            let drop_fn: DropFn = |to_drop| unsafe {
                let to_drop = to_drop.cast();
                debug_assert_ptr_is_aligned(to_drop);
                ptr::drop_in_place::<T>(to_drop);
            };
            Some(drop_fn)
        } else {
            None
        };

        Self { layout, drop_fn }
    }

    #[inline]
    pub const fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    #[inline]
    pub const fn drop_fn(&self) -> Option<DropFn> {
        let Self { drop_fn, .. } = *self;
        drop_fn
    }

    #[inline]
    pub const fn into_inner(self) -> (Layout, Option<DropFn>) {
        let Self { layout, drop_fn } = self;
        (layout, drop_fn)
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
