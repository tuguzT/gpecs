use alloc::{boxed::Box, vec::Vec};
use core::{iter, ptr, slice};

use crate::traits::{buffer_layout, FieldDescriptor, Soa};

use super::{
    assert::{check_same_len, validate_layout},
    byte::{Aligned, ErasedByte},
    error::LenMismatchError,
    field::{ErasedField, ErasedFieldRef, ErasedFieldRefMut},
    ErasedSoaRefs, ErasedSoaRefsMut,
};

pub struct ErasedSoa<Fields> {
    buffer: Box<[ErasedByte<Aligned<Fields>>]>,
    descriptors: Box<[FieldDescriptor]>,
}

impl<Fields> ErasedSoa<Fields> {
    #[inline]
    pub fn new<I, F>(fields: I) -> Result<Self, LenMismatchError>
    where
        I: IntoIterator<Item = (FieldDescriptor, F)>,
        F: AsRef<[u8]>,
    {
        let (descriptors, fields): (Vec<_>, Vec<_>) = fields
            .into_iter()
            .map(|(desc, src)| {
                validate_layout::<Fields>(desc.layout());
                check_same_len(src.as_ref().len(), desc.layout().size())?;
                Ok((desc, src))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .unzip();
        let descriptors = descriptors.into_boxed_slice();

        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(descriptors.iter().map(FieldDescriptor::layout), 1)
                .expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Aligned<Fields>>>());

        let mut buffer = Box::new_uninit_slice(buffer_len);
        for ((desc, src), offset) in descriptors.iter().zip(fields).zip(offsets) {
            let src = src.as_ref().as_ptr();
            let dst = unsafe { buffer.as_mut_ptr().cast::<u8>().add(offset) };

            let len = desc.layout().size();
            unsafe {
                ptr::copy_nonoverlapping(src, dst, len);
            }
        }
        Ok(Self {
            buffer: unsafe { buffer.assume_init() },
            descriptors,
        })
    }

    #[inline]
    pub fn from<T>(context: &T::Context, value: T) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let descriptors = T::field_descriptors(context)
            .into_iter()
            .inspect(|desc| validate_layout::<T::Fields>(desc.as_ref().layout()))
            .map(|desc| desc.as_ref().clone())
            .collect();

        let (buffer_layout, offsets) =
            T::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Aligned<Fields>>>());

        let mut buffer = Box::new_uninit_slice(buffer_len);
        unsafe {
            let dst = {
                let buffer = buffer.as_mut_ptr().cast::<u8>();
                let ptrs = offsets.into_iter().map(|offset| buffer.add(offset));
                T::ptrs_restore_mut(context, ptrs)
            };
            T::ptrs_write(context, dst, value);
        }

        Self {
            buffer: unsafe { buffer.assume_init() },
            descriptors,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T
    where
        T: Soa<Fields = Fields>,
    {
        let Self {
            buffer,
            descriptors,
        } = self;

        let target_layouts = T::field_descriptors(context)
            .into_iter()
            .inspect(|desc| validate_layout::<T::Fields>(desc.as_ref().layout()))
            .map(|desc| desc.as_ref().layout());
        let field_layouts = descriptors.iter().map(FieldDescriptor::layout);
        assert!(target_layouts.eq(field_layouts));

        let (buffer_layout, offsets) =
            T::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Aligned<Fields>>>());
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
    pub fn into_fields(self) -> Box<[ErasedField<Aligned<Fields>>]> {
        let Self {
            buffer,
            descriptors,
        } = self;

        let field_layouts = descriptors.iter().map(FieldDescriptor::layout);
        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(field_layouts, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Aligned<Fields>>>());
        assert_eq!(buffer_len, buffer.len());

        iter::zip(descriptors, offsets)
            .map(|(desc, offset)| {
                let data = unsafe { buffer.as_ptr().cast::<u8>().add(offset) };
                let buffer = unsafe { slice::from_raw_parts(data, desc.layout().size()) };
                unsafe { ErasedField::new_unchecked(desc, buffer) }
            })
            .collect()
    }

    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }

    #[inline]
    pub fn as_refs(&self) -> ErasedSoaRefs<'_, Fields> {
        let Self {
            buffer,
            descriptors,
        } = self;

        let field_layouts = descriptors.iter().map(FieldDescriptor::layout);
        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(field_layouts, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Aligned<Fields>>>());
        assert_eq!(buffer_len, buffer.len());

        let refs = descriptors.iter().zip(offsets).map(|(desc, offset)| {
            let data = unsafe { buffer.as_ptr().cast::<u8>().add(offset) };
            let len = desc.layout().size();
            let r#ref = unsafe { slice::from_raw_parts(data, len) };
            unsafe { ErasedFieldRef::new_unchecked(desc.clone(), r#ref) }
        });
        ErasedSoaRefs::new(refs)
    }

    #[inline]
    pub fn as_refs_mut(&mut self) -> ErasedSoaRefsMut<'_, Fields> {
        let Self {
            buffer,
            descriptors,
        } = self;

        let field_layouts = descriptors.iter().map(FieldDescriptor::layout);
        let (buffer_layout, offsets): (_, Box<[_]>) =
            buffer_layout(field_layouts, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Aligned<Fields>>>());
        assert_eq!(buffer_len, buffer.len());

        let refs = descriptors.iter().zip(offsets).map(|(desc, offset)| {
            let data = unsafe { buffer.as_mut_ptr().cast::<u8>().add(offset) };
            let len = desc.layout().size();
            let r#ref = unsafe { slice::from_raw_parts_mut(data, len) };
            unsafe { ErasedFieldRefMut::new_unchecked(desc.clone(), r#ref) }
        });
        ErasedSoaRefsMut::new(refs)
    }
}
