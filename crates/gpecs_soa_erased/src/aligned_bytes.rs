use alloc::alloc::{alloc, dealloc, handle_alloc_error};
use core::{
    alloc::Layout,
    ptr::{self, NonNull},
};

pub struct AlignedBytes {
    ptr: NonNull<u8>,
    layout: Layout,
}

impl AlignedBytes {
    #[inline]
    pub fn new(layout: Layout) -> Self {
        let ptr = match layout.size() {
            0 => ptr::without_provenance_mut(layout.align()),
            _ => unsafe { alloc(layout) },
        };
        let Some(ptr) = NonNull::new(ptr.cast()) else {
            handle_alloc_error(layout);
        };
        Self { ptr, layout }
    }

    #[inline]
    pub fn layout(&self) -> Layout {
        let Self { layout, .. } = *self;
        layout
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        let Self { ptr, .. } = self;
        ptr.as_ptr()
    }
}

impl AsRef<AlignedBytes> for AlignedBytes {
    #[inline]
    fn as_ref(&self) -> &AlignedBytes {
        self
    }
}

impl AsMut<AlignedBytes> for AlignedBytes {
    #[inline]
    fn as_mut(&mut self) -> &mut AlignedBytes {
        self
    }
}

impl Drop for AlignedBytes {
    fn drop(&mut self) {
        let Self { ptr, layout } = *self;
        if layout.size() == 0 {
            return;
        }
        unsafe { dealloc(ptr.as_ptr().cast(), layout) }
    }
}
