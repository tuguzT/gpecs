use alloc::{boxed::Box, vec::Vec};
use core::{alloc::Layout, borrow::Borrow, iter, ptr, slice};

use crate::traits::{buffer_layout, Soa};

use super::{
    assert::validate_layout,
    byte::ErasedByte,
    field::{ErasedFieldRef, ErasedFieldRefMut},
    ErasedSoaRefs, ErasedSoaRefsMut,
};

// data is stored inline in a single buffer
type ErasedFields<Fields> = Box<[ErasedByte<Fields>]>;

pub struct ErasedSoa<Fields> {
    pub(super) buffer: ErasedFields<Fields>,
    pub(super) field_layouts: Box<[Layout]>,
}

impl<Fields> ErasedSoa<Fields> {
    #[inline]
    pub fn new<I, F>(fields: I) -> Self
    where
        I: IntoIterator<Item = (Layout, F)>,
        F: Borrow<[u8]>,
    {
        let (field_layouts, fields): (Vec<_>, Vec<_>) = fields
            .into_iter()
            .inspect(|(field_layout, src)| assert_eq!(field_layout.size(), src.borrow().len()))
            .unzip();
        let field_layouts = field_layouts.into_boxed_slice();

        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(&field_layouts, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Fields>>());

        let mut buffer = Box::new_uninit_slice(buffer_len);
        let buffer = unsafe {
            for ((field_layout, src), offset) in field_layouts.iter().zip(fields).zip(offsets) {
                let src = src.borrow().as_ptr();
                let dst = buffer.as_mut_ptr().cast::<u8>().add(offset);

                let len = field_layout.size();
                ptr::copy_nonoverlapping(src, dst, len);
            }
            buffer.assume_init()
        };
        Self {
            buffer,
            field_layouts,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, value: T) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<T::Fields, _>)
            .collect();

        let (buffer_layout, offsets) =
            T::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Fields>>());

        let mut buffer = Box::new_uninit_slice(buffer_len);
        let buffer = unsafe {
            let dst = {
                let buffer = buffer.as_mut_ptr().cast::<u8>();
                let ptrs = offsets.into_iter().map(|offset| buffer.add(offset));
                T::ptrs_restore_mut(context, ptrs)
            };
            T::ptrs_write(context, dst, value);
            buffer.assume_init()
        };

        Self {
            buffer,
            field_layouts,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T
    where
        T: Soa<Fields = Fields>,
    {
        let Self {
            buffer,
            field_layouts,
        } = self;

        let target_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<T::Fields, _>);
        assert!(target_layouts.eq(field_layouts));

        let (buffer_layout, offsets) =
            T::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        unsafe {
            let src = {
                let buffer = buffer.as_ptr().cast::<u8>();
                let ptrs = offsets.into_iter().map(|offset| buffer.add(offset));
                T::ptrs_restore(context, ptrs)
            };
            T::ptrs_read(context, src)
        }
    }

    #[inline]
    pub fn into_fields(self) -> Box<[(Layout, Box<[u8]>)]> {
        let Self {
            buffer,
            field_layouts,
        } = self;

        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(&field_layouts, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        iter::zip(field_layouts, offsets)
            .map(|(field_layout, offset)| {
                let data = unsafe { buffer.as_ptr().cast::<u8>().add(offset) };
                let len = field_layout.size();
                let r#ref = unsafe { slice::from_raw_parts(data, len) };
                (field_layout.clone(), r#ref.into())
            })
            .collect()
    }

    #[inline]
    pub fn layouts(&self) -> &[Layout] {
        let Self { field_layouts, .. } = self;
        field_layouts.as_ref()
    }

    #[inline]
    pub fn as_refs(&self) -> ErasedSoaRefs<'_, Fields> {
        let Self {
            buffer,
            field_layouts,
        } = self;

        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(field_layouts, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        let refs = field_layouts
            .iter()
            .zip(offsets)
            .map(|(field_layout, offset)| {
                let data = unsafe { buffer.as_ptr().cast::<u8>().add(offset) };
                let len = field_layout.size();
                let r#ref = unsafe { slice::from_raw_parts(data, len) };
                ErasedFieldRef::new(field_layout.clone(), r#ref)
            });
        ErasedSoaRefs::new(refs)
    }

    #[inline]
    pub fn as_refs_mut(&mut self) -> ErasedSoaRefsMut<'_, Fields> {
        let Self {
            buffer,
            field_layouts,
        } = self;

        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(&*field_layouts, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        let refs = field_layouts
            .iter()
            .zip(offsets)
            .map(|(field_layout, offset)| {
                let data = unsafe { buffer.as_mut_ptr().cast::<u8>().add(offset) };
                let len = field_layout.size();
                let r#ref = unsafe { slice::from_raw_parts_mut(data, len) };
                ErasedFieldRefMut::new(field_layout.clone(), r#ref)
            });
        ErasedSoaRefsMut::new(refs)
    }
}
