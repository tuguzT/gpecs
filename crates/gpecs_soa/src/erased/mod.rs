use alloc::{
    boxed::Box,
    vec::{self, Vec},
};
use core::{
    alloc::{Layout, LayoutError},
    iter,
    ptr::{self, NonNull},
    slice,
};

use crate::traits::{buffer_layout, Soa};

use self::{
    assert::assert_layouts,
    byte::ErasedByte,
    field::{
        ErasedFieldMutPtr, ErasedFieldNonNullPtr, ErasedFieldPtr, ErasedFieldRef,
        ErasedFieldRefMut, ErasedFieldSlice, ErasedFieldSliceMut, ErasedFieldSliceMutPtr,
        ErasedFieldSlicePtr,
    },
};

pub use self::{
    context::ErasedSoaContext,
    nonnull_ptrs::ErasedSoaNonNullPtrs,
    ptrs::ErasedSoaPtrs,
    ptrs_mut::ErasedSoaMutPtrs,
    refs::ErasedSoaRefs,
    refs_mut::ErasedSoaRefsMut,
    slice_ptrs::{ErasedSoaSlicePtrs, ErasedSoaSlicePtrsIter},
    slice_ptrs_mut::{ErasedSoaSliceMutPtrs, ErasedSoaSliceMutPtrsIter},
    slices::{ErasedSoaSlices, ErasedSoaSlicesIter},
    slices_mut::{ErasedSoaSlicesIterMut, ErasedSoaSlicesMut},
    value::ErasedSoa,
    vecs::{ErasedFieldVec, ErasedSoaVecs},
};

pub mod field;

mod assert;
mod byte;
mod context;
mod nonnull_ptrs;
mod ptrs;
mod ptrs_mut;
mod refs;
mod refs_mut;
mod slice_ptrs;
mod slice_ptrs_mut;
mod slices;
mod slices_mut;
mod value;
mod vecs;

unsafe impl<Fields> Soa for ErasedSoa<Fields> {
    type Context = ErasedSoaContext<Fields>;

    type Fields = Fields;

    type FieldLayouts<'a> = &'a [Layout];

    fn field_layouts(context: &Self::Context) -> Self::FieldLayouts<'_> {
        let ErasedSoaContext { field_layouts, .. } = context;
        field_layouts.as_ref()
    }

    type FieldOffsets<'a> = Box<[usize]>;

    fn buffer_layout(
        context: &Self::Context,
        capacity: usize,
    ) -> Result<(Layout, Self::FieldOffsets<'_>), LayoutError> {
        let ErasedSoaContext { field_layouts, .. } = context;
        buffer_layout(field_layouts, capacity)
    }

    type Ptrs = ErasedSoaPtrs<Fields>;
    type MutPtrs = ErasedSoaMutPtrs<Fields>;

    type ErasedPtrs = iter::Map<vec::IntoIter<ErasedFieldPtr>, fn(ErasedFieldPtr) -> *const u8>;
    type ErasedMutPtrs =
        iter::Map<vec::IntoIter<ErasedFieldMutPtr>, fn(ErasedFieldMutPtr) -> *mut u8>;

    fn ptrs_erase(context: &Self::Context, ptrs: Self::Ptrs) -> Self::ErasedPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), ptrs.fields().len());

        ptrs.into_fields()
            .into_vec()
            .into_iter()
            .map(ErasedFieldPtr::into_ptr)
    }

    fn ptrs_erase_mut(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::ErasedMutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), ptrs.fields().len());

        ptrs.into_fields()
            .into_vec()
            .into_iter()
            .map(ErasedFieldMutPtr::into_ptr)
    }

    fn ptrs_restore(
        context: &Self::Context,
        ptrs: impl IntoIterator<Item = *const u8>,
    ) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;

        let ptrs: Box<[_]> = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let buffer = ptr::slice_from_raw_parts(ptr, field_layout.size());
                ErasedFieldPtr::new(*field_layout, buffer)
            })
            .collect();
        assert_eq!(field_layouts.len(), ptrs.len());

        ErasedSoaPtrs::new(ptrs)
    }

    fn ptrs_restore_mut(
        context: &Self::Context,
        ptrs: impl IntoIterator<Item = *mut u8>,
    ) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;

        let ptrs: Box<[_]> = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let buffer = ptr::slice_from_raw_parts_mut(ptr, field_layout.size());
                ErasedFieldMutPtr::new(*field_layout, buffer)
            })
            .collect();
        assert_eq!(field_layouts.len(), ptrs.len());

        ErasedSoaMutPtrs::new(ptrs)
    }

    fn ptrs_dangling(context: &Self::Context) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;

        let ptrs = field_layouts
            .iter()
            .copied()
            .map(ErasedFieldMutPtr::dangling);
        ErasedSoaMutPtrs::new(ptrs)
    }

    fn ptrs_cast_const(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), ptrs.fields().len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(&field_layout, ptr)| assert_layouts(field_layout, ptr.layout()))
            .map(|(_, ptr)| ptr.cast_const());
        ErasedSoaPtrs::new(ptrs)
    }

    fn ptrs_cast_mut(context: &Self::Context, ptrs: Self::Ptrs) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), ptrs.fields().len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(&field_layout, ptr)| assert_layouts(field_layout, ptr.layout()))
            .map(|(_, ptr)| ptr.cast_mut());
        ErasedSoaMutPtrs::new(ptrs)
    }

    unsafe fn ptrs_add(context: &Self::Context, ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), ptrs.fields().len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(&field_layout, ptr)| assert_layouts(field_layout, ptr.layout()))
            .map(|(_, ptr)| unsafe { ptr.add(offset) });
        ErasedSoaPtrs::new(ptrs)
    }

    unsafe fn ptrs_add_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        offset: usize,
    ) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), ptrs.fields().len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(&field_layout, ptr)| assert_layouts(field_layout, ptr.layout()))
            .map(|(_, ptr)| unsafe { ptr.add(offset) });
        ErasedSoaMutPtrs::new(ptrs)
    }

    unsafe fn ptrs_offset_from(
        context: &Self::Context,
        ptrs: Self::Ptrs,
        origin: Self::Ptrs,
    ) -> isize {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), ptrs.fields().len());
        assert_eq!(ptrs.fields().len(), origin.fields().len());

        let mut offsets = field_layouts
            .iter()
            .zip(ptrs.into_fields())
            .zip(origin.into_fields())
            .inspect(|((&field_layout, ptr), origin)| {
                assert_layouts(field_layout, ptr.layout());
                assert_layouts(field_layout, origin.layout());
            })
            .map(|((_, ptr), origin)| unsafe { ptr.offset_from(&origin) });

        let offset = offsets.next().expect("self should not be a ZST");
        assert!(offsets.all(|item| item == offset));
        offset
    }

    unsafe fn ptrs_offset_from_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        origin: Self::Ptrs,
    ) -> isize {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), ptrs.fields().len());
        assert_eq!(ptrs.fields().len(), origin.fields().len());

        let mut offsets = field_layouts
            .iter()
            .zip(ptrs.into_fields())
            .zip(origin.into_fields())
            .inspect(|((&field_layout, ptr), origin)| {
                assert_layouts(field_layout, ptr.layout());
                assert_layouts(field_layout, origin.layout());
            })
            .map(|((_, ptr), origin)| unsafe { ptr.offset_from(&origin) });

        let offset = offsets.next().expect("self should not be a ZST");
        assert!(offsets.all(|item| item == offset));
        offset
    }

    unsafe fn ptrs_swap(context: &Self::Context, a: Self::MutPtrs, b: Self::MutPtrs) {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), a.fields().len());
        assert_eq!(a.fields().len(), b.fields().len());

        let mut temp = Vec::new();
        field_layouts
            .iter()
            .zip(a.into_fields())
            .zip(b.into_fields())
            .inspect(|((&field_layout, a), b)| {
                assert_layouts(field_layout, a.layout());
                assert_layouts(field_layout, b.layout());
            })
            .for_each(|((_, a), b)| {
                let len = a.layout().size();
                temp.reserve(len);

                let a = a.as_ptr();
                let b = b.as_ptr();
                unsafe {
                    ptr::copy_nonoverlapping(a, temp.as_mut_ptr(), len);
                    temp.set_len(len);

                    ptr::copy(b, a, len);
                    ptr::copy_nonoverlapping(temp.as_ptr(), b, len);
                }
                temp.clear();
            })
    }

    unsafe fn ptrs_copy(context: &Self::Context, src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), src.fields().len());
        assert_eq!(src.fields().len(), dst.fields().len());

        let mut temp = Vec::new();
        field_layouts
            .iter()
            .zip(src.into_fields())
            .zip(dst.into_fields())
            .inspect(|((&field_layout, src), dst)| {
                assert_layouts(field_layout, src.layout());
                assert_layouts(field_layout, dst.layout());
            })
            .for_each(|((_, src), dst)| {
                let len = len * src.layout().size();
                temp.reserve(len);

                let src = src.as_ptr();
                let dst = dst.as_ptr();
                unsafe {
                    ptr::copy_nonoverlapping(src, temp.as_mut_ptr(), len);
                    temp.set_len(len);

                    ptr::copy_nonoverlapping(temp.as_ptr(), dst, len);
                }
                temp.clear();
            })
    }

    unsafe fn ptrs_copy_rev(
        context: &Self::Context,
        src: Self::Ptrs,
        dst: Self::MutPtrs,
        len: usize,
    ) {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), src.fields().len());
        assert_eq!(src.fields().len(), dst.fields().len());

        let mut temp = Vec::new();
        field_layouts
            .iter()
            .zip(src.into_fields())
            .zip(dst.into_fields())
            .rev()
            .inspect(|((&field_layout, src), dst)| {
                assert_layouts(field_layout, src.layout());
                assert_layouts(field_layout, dst.layout());
            })
            .for_each(|((_, src), dst)| {
                let len = len * src.layout().size();
                temp.reserve(len);

                let src = src.as_ptr();
                let dst = dst.as_ptr();
                unsafe {
                    ptr::copy_nonoverlapping(src, temp.as_mut_ptr(), len);
                    temp.set_len(len);

                    ptr::copy_nonoverlapping(temp.as_ptr(), dst, len);
                }
                temp.clear();
            })
    }

    unsafe fn ptrs_copy_nonoverlapping(
        context: &Self::Context,
        src: Self::Ptrs,
        dst: Self::MutPtrs,
        len: usize,
    ) {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), src.fields().len());
        assert_eq!(src.fields().len(), dst.fields().len());

        field_layouts
            .iter()
            .zip(src.into_fields())
            .zip(dst.into_fields())
            .inspect(|((&field_layout, src), dst)| {
                assert_layouts(field_layout, src.layout());
                assert_layouts(field_layout, dst.layout());
            })
            .for_each(|((_, src), dst)| {
                let len = len * src.layout().size();
                let src = src.as_ptr();
                let dst = dst.as_ptr();
                unsafe {
                    ptr::copy_nonoverlapping(src, dst, len);
                }
            })
    }

    unsafe fn ptrs_read(context: &Self::Context, src: Self::Ptrs) -> Self {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), src.fields().len());

        let (buffer_layout, offsets) =
            Self::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Fields>>());

        let mut buffer = Box::new_uninit_slice(buffer_len);
        field_layouts
            .iter()
            .zip(src.into_fields())
            .zip(offsets)
            .inspect(|((&field_layout, src), _)| assert_layouts(field_layout, src.layout()))
            .for_each(|((_, src), offset)| {
                let len = src.layout().size();
                let src = src.as_ptr();
                let dst = unsafe { buffer.as_mut_ptr().cast::<u8>().add(offset) };
                unsafe { ptr::copy_nonoverlapping(src, dst, len) }
            });
        Self {
            buffer: unsafe { buffer.assume_init() },
            field_layouts: field_layouts.clone(),
        }
    }

    unsafe fn ptrs_write(context: &Self::Context, dst: Self::MutPtrs, value: Self) {
        let ErasedSoaContext { field_layouts, .. } = context;
        let Self {
            buffer,
            field_layouts: value_layouts,
        } = value;

        assert_eq!(field_layouts.len(), dst.fields().len());
        assert_eq!(field_layouts.as_ref(), value_layouts.as_ref());

        let (buffer_layout, offsets) =
            Self::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        field_layouts
            .iter()
            .zip(dst.into_fields())
            .zip(offsets)
            .inspect(|((&field_layout, dst), _)| assert_layouts(field_layout, dst.layout()))
            .for_each(|((_, dst), offset)| {
                let len = dst.layout().size();
                let src = unsafe { buffer.as_ptr().cast::<u8>().add(offset) };
                let dst = dst.as_ptr();
                unsafe { ptr::copy_nonoverlapping(src, dst, len) }
            })
    }

    unsafe fn ptrs_drop_in_place(context: &Self::Context, ptrs: Self::MutPtrs) {
        let ErasedSoaContext {
            field_layouts,
            drop_fields,
            ..
        } = context;
        let Some(drop_fields) = drop_fields else {
            return;
        };

        let layouts = ptrs.fields().iter().map(ErasedFieldMutPtr::layout);
        assert!(field_layouts.iter().copied().eq(layouts));

        drop_fields(ptrs.fields())
    }

    type NonNullPtrs = ErasedSoaNonNullPtrs<Fields>;

    unsafe fn ptrs_to_nonnull(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), ptrs.fields().len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(&field_layout, ptr)| assert_layouts(field_layout, ptr.layout()))
            .map(|(_, ptr)| {
                let layout = ptr.layout();
                let buffer = unsafe { NonNull::new_unchecked(ptr.buffer()) };
                ErasedFieldNonNullPtr::new(layout, buffer)
            });
        ErasedSoaNonNullPtrs::new(ptrs)
    }

    fn nonnull_to_ptrs(context: &Self::Context, ptrs: Self::NonNullPtrs) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), ptrs.fields().len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(&field_layout, ptr)| assert_layouts(field_layout, ptr.layout()))
            .map(|(_, ptr)| {
                let buffer = ptr.buffer().as_ptr();
                ErasedFieldMutPtr::new(ptr.layout(), buffer)
            });
        ErasedSoaMutPtrs::new(ptrs)
    }

    type Vecs = ErasedSoaVecs<Fields>;

    fn vecs_with_capacity(context: &Self::Context, capacity: usize) -> Self::Vecs {
        let ErasedSoaContext { field_layouts, .. } = context;

        let vecs = field_layouts
            .iter()
            .map(|field_layout| {
                let capacity =
                    (capacity * field_layout.size()).div_ceil(size_of::<ErasedByte<Fields>>());
                ErasedFieldVec {
                    buffer: Vec::with_capacity(capacity),
                    layout: field_layout.clone(),
                }
            })
            .collect();
        ErasedSoaVecs { len: 0, vecs }
    }

    fn vecs_as_ptrs(context: &Self::Context, vecs: &Self::Vecs) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaVecs { vecs, .. } = vecs;

        assert_eq!(field_layouts.len(), vecs.len());

        let ptrs = field_layouts
            .iter()
            .zip(vecs)
            .inspect(|(&field_layout, vec)| assert_layouts(field_layout, vec.layout))
            .map(|(_, vec)| {
                let ErasedFieldVec { buffer, layout, .. } = vec;

                let data = buffer.as_ptr().cast();
                let len = layout.size();
                let buffer = ptr::slice_from_raw_parts(data, len);
                ErasedFieldPtr::new(layout.clone(), buffer)
            });
        ErasedSoaPtrs::new(ptrs)
    }

    fn mut_vecs_as_ptrs(context: &Self::Context, vecs: &mut Self::Vecs) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaVecs { vecs, .. } = vecs;

        assert_eq!(field_layouts.len(), vecs.len());

        let ptrs = field_layouts
            .iter()
            .zip(vecs)
            .inspect(|(&field_layout, vec)| assert_layouts(field_layout, vec.layout))
            .map(|(_, vec)| {
                let ErasedFieldVec { buffer, layout, .. } = vec;

                let data = buffer.as_mut_ptr().cast();
                let len = layout.size();
                let buffer = ptr::slice_from_raw_parts_mut(data, len);
                ErasedFieldMutPtr::new(layout.clone(), buffer)
            });
        ErasedSoaMutPtrs::new(ptrs)
    }

    fn vecs_len(context: &Self::Context, vecs: &Self::Vecs) -> usize {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaVecs { vecs, len, .. } = vecs;

        assert_eq!(field_layouts.len(), vecs.len());

        // let mut lens = field_layouts.iter().zip(vecs).map(|(field_layout, vec)| {
        //     let ErasedFieldVec {
        //         buffer,
        //         layout: vec_field_layout,
        //         ..
        //     } = vec;
        //     assert_eq!(field_layout, vec_field_layout);
        //     *len
        // });
        // let len = lens.next().unwrap_or(0);
        // assert!(lens.all(|item| item == len));
        *len
    }

    unsafe fn vecs_set_len(context: &Self::Context, vecs: &mut Self::Vecs, len: usize) {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaVecs {
            vecs, len: vec_len, ..
        } = vecs;

        assert_eq!(field_layouts.len(), vecs.len());

        for (field_layout, vec) in field_layouts.iter().zip(vecs) {
            let ErasedFieldVec {
                buffer: field_buffer,
                layout: vec_field_layout,
            } = vec;
            assert_eq!(field_layout, vec_field_layout);

            *vec_len = len;
            let len = (len * vec_field_layout.size()).div_ceil(size_of::<ErasedByte<Fields>>());
            unsafe {
                field_buffer.set_len(len);
            }
        }
    }

    type Refs<'a>
        = ErasedSoaRefs<'a, Fields>
    where
        Self: 'a;

    type RefsMut<'a>
        = ErasedSoaRefsMut<'a, Fields>
    where
        Self: 'a;

    unsafe fn ptrs_to_refs<'a>(context: &Self::Context, ptrs: Self::Ptrs) -> Self::Refs<'a> {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), ptrs.fields().len());

        let refs = field_layouts
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(&field_layout, ptr)| assert_layouts(field_layout, ptr.layout()))
            .map(|(_, ptr)| {
                let layout = ptr.layout();
                let buffer = unsafe { slice::from_raw_parts(ptr.as_ptr(), layout.size()) };
                ErasedFieldRef::new(layout, buffer)
            });
        ErasedSoaRefs::new(refs)
    }

    unsafe fn ptrs_to_refs_mut<'a>(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
    ) -> Self::RefsMut<'a> {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), ptrs.fields().len());

        let refs = field_layouts
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(&field_layout, ptr)| assert_layouts(field_layout, ptr.layout()))
            .map(|(_, ptr)| {
                let layout = ptr.layout();
                let buffer = unsafe { slice::from_raw_parts_mut(ptr.as_ptr(), ptr.buffer().len()) };
                ErasedFieldRefMut::new(layout, buffer)
            });
        ErasedSoaRefsMut::new(refs)
    }

    fn refs_as_ptrs(context: &Self::Context, refs: Self::Refs<'_>) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), refs.fields().len());

        let ptrs = field_layouts
            .iter()
            .zip(refs.into_fields())
            .inspect(|(&field_layout, r#ref)| assert_layouts(field_layout, r#ref.layout()))
            .map(|(_, r#ref)| {
                let buffer = ptr::from_ref(r#ref.buffer());
                ErasedFieldPtr::new(r#ref.layout(), buffer)
            });
        ErasedSoaPtrs::new(ptrs)
    }

    fn mut_refs_as_ptrs(context: &Self::Context, refs: Self::RefsMut<'_>) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), refs.fields().len());

        let ptrs = field_layouts
            .iter()
            .zip(refs.into_fields())
            .inspect(|(&field_layout, r#ref)| assert_layouts(field_layout, r#ref.layout()))
            .map(|(_, mut r#ref)| {
                let buffer = ptr::from_mut(r#ref.buffer_mut());
                ErasedFieldMutPtr::new(r#ref.layout(), buffer)
            });
        ErasedSoaMutPtrs::new(ptrs)
    }

    fn mut_refs_as_refs<'a>(context: &Self::Context, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), refs.fields().len());

        let refs = field_layouts
            .iter()
            .zip(refs.into_fields())
            .inspect(|(&field_layout, r#ref)| assert_layouts(field_layout, r#ref.layout()))
            .map(|(_, r#ref)| ErasedFieldRef::new(r#ref.layout(), r#ref.into_buffer()));
        ErasedSoaRefs::new(refs)
    }

    type SlicePtrs = ErasedSoaSlicePtrs<Fields>;

    type SliceMutPtrs = ErasedSoaSliceMutPtrs<Fields>;

    fn slices_from_raw_parts(
        context: &Self::Context,
        ptrs: Self::Ptrs,
        len: usize,
    ) -> Self::SlicePtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), ptrs.fields().len());

        let slices = field_layouts
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(&field_layout, ptr)| assert_layouts(field_layout, ptr.layout()))
            .map(|(_, ptr)| {
                let layout = ptr.layout();
                let buffer = ptr::slice_from_raw_parts(ptr.as_ptr(), len * layout.size());
                ErasedFieldSlicePtr::new(layout, buffer)
            });
        ErasedSoaSlicePtrs::new(len, slices)
    }

    fn slices_from_raw_parts_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        len: usize,
    ) -> Self::SliceMutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), ptrs.fields().len());

        let slices = field_layouts
            .iter()
            .zip(ptrs.into_fields())
            .inspect(|(&field_layout, ptr)| assert_layouts(field_layout, ptr.layout()))
            .map(|(_, ptr)| {
                let layout = ptr.layout();
                let buffer = ptr::slice_from_raw_parts_mut(ptr.as_ptr(), len * layout.size());
                ErasedFieldSliceMutPtr::new(layout, buffer)
            });
        ErasedSoaSliceMutPtrs::new(len, slices)
    }

    fn slice_ptrs_cast_const(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicePtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), slices.fields().len());

        let len = slices.len();
        let slices = field_layouts
            .iter()
            .zip(slices.into_fields())
            .inspect(|(&field_layout, slice)| assert_layouts(field_layout, slice.layout()))
            .map(|(_, slice)| {
                let buffer = slice.buffer().cast_const();
                ErasedFieldSlicePtr::new(slice.layout(), buffer)
            });
        ErasedSoaSlicePtrs::new(len, slices)
    }

    fn slice_ptrs_cast_mut(context: &Self::Context, slices: Self::SlicePtrs) -> Self::SliceMutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), slices.fields().len());

        let len = slices.len();
        let slices = field_layouts
            .iter()
            .zip(slices.into_fields())
            .inspect(|(&field_layout, slice)| assert_layouts(field_layout, slice.layout()))
            .map(|(_, slice)| {
                let buffer = slice.buffer().cast_mut();
                ErasedFieldSliceMutPtr::new(slice.layout(), buffer)
            });
        ErasedSoaSliceMutPtrs::new(len, slices)
    }

    fn slice_ptrs_len(context: &Self::Context, slices: Self::SlicePtrs) -> usize {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), slices.fields().len());

        slices.len()
    }

    fn slice_ptrs_len_mut(context: &Self::Context, slices: Self::SliceMutPtrs) -> usize {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), slices.fields().len());

        slices.len()
    }

    fn slice_ptrs_as_ptrs(context: &Self::Context, slices: Self::SlicePtrs) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), slices.fields().len());

        let ptrs = field_layouts
            .iter()
            .zip(slices.into_fields())
            .inspect(|(&field_layout, slice)| assert_layouts(field_layout, slice.layout()))
            .map(|(_, slice)| {
                let layout = slice.layout();
                let buffer = ptr::slice_from_raw_parts(slice.as_ptr(), layout.size());
                ErasedFieldPtr::new(layout, buffer)
            });
        ErasedSoaPtrs::new(ptrs)
    }

    fn mut_slice_ptrs_as_ptrs(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), slices.fields().len());

        let ptrs = field_layouts
            .iter()
            .zip(slices.into_fields())
            .inspect(|(&field_layout, slice)| assert_layouts(field_layout, slice.layout()))
            .map(|(_, slice)| {
                let layout = slice.layout();
                let buffer = ptr::slice_from_raw_parts_mut(slice.as_ptr(), layout.size());
                ErasedFieldMutPtr::new(layout, buffer)
            });
        ErasedSoaMutPtrs::new(ptrs)
    }

    type Slices<'a>
        = ErasedSoaSlices<'a, Fields>
    where
        Self: 'a;

    type SlicesMut<'a>
        = ErasedSoaSlicesMut<'a, Fields>
    where
        Self: 'a;

    unsafe fn slice_ptrs_to_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicePtrs,
    ) -> Self::Slices<'a> {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), slices.fields().len());

        let len = slices.len();
        let slices = field_layouts
            .iter()
            .zip(slices.into_fields())
            .inspect(|(&field_layout, slice)| assert_layouts(field_layout, slice.layout()))
            .map(|(_, slice)| {
                let data = slice.as_ptr();
                let len = slice.buffer().len();
                let buffer = unsafe { slice::from_raw_parts(data, len) };
                ErasedFieldSlice::new(slice.layout(), buffer)
            });
        ErasedSoaSlices::new(len, slices)
    }

    unsafe fn slice_ptrs_to_slices_mut<'a>(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicesMut<'a> {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), slices.fields().len());

        let len = slices.len();
        let slices = field_layouts
            .iter()
            .zip(slices.into_fields())
            .inspect(|(&field_layout, slice)| assert_layouts(field_layout, slice.layout()))
            .map(|(_, slice)| {
                let data = slice.as_ptr();
                let len = slice.buffer().len();
                let buffer = unsafe { slice::from_raw_parts_mut(data, len) };
                ErasedFieldSliceMut::new(slice.layout(), buffer)
            });
        ErasedSoaSlicesMut::new(len, slices)
    }

    fn slices_len(context: &Self::Context, slices: &Self::Slices<'_>) -> usize {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), slices.fields().len());

        slices.len()
    }

    fn slices_len_mut(context: &Self::Context, slices: &Self::SlicesMut<'_>) -> usize {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), slices.fields().len());

        slices.len()
    }

    fn slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::Slices<'_>,
    ) -> Self::SlicePtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), slices.fields().len());

        let len = slices.len();
        let slices = field_layouts
            .iter()
            .zip(slices.into_fields())
            .inspect(|(&field_layout, slice)| assert_layouts(field_layout, slice.layout()))
            .map(|(_, slice)| {
                let buffer = ptr::from_ref(slice.buffer());
                ErasedFieldSlicePtr::new(slice.layout(), buffer)
            });
        ErasedSoaSlicePtrs::new(len, slices)
    }

    fn mut_slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::SliceMutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), slices.fields().len());

        let len = slices.len();
        let slices = field_layouts
            .iter()
            .zip(slices.into_fields())
            .inspect(|(&field_layout, slice)| assert_layouts(field_layout, slice.layout()))
            .map(|(_, mut slice)| {
                let buffer = ptr::from_mut(slice.buffer_mut());
                ErasedFieldSliceMutPtr::new(slice.layout(), buffer)
            });
        ErasedSoaSliceMutPtrs::new(len, slices)
    }

    fn mut_slices_as_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicesMut<'a>,
    ) -> Self::Slices<'a> {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), slices.fields().len());

        let len = slices.len();
        let slices = field_layouts
            .iter()
            .zip(slices.into_fields())
            .inspect(|(&field_layout, slice)| assert_layouts(field_layout, slice.layout()))
            .map(|(_, slice)| ErasedFieldSlice::new(slice.layout(), slice.into_buffer()));
        ErasedSoaSlices::new(len, slices)
    }

    fn slice_refs_as_ptrs(context: &Self::Context, slices: Self::Slices<'_>) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), slices.fields().len());

        let ptrs = field_layouts
            .iter()
            .zip(slices.into_fields())
            .inspect(|(&field_layout, slice)| assert_layouts(field_layout, slice.layout()))
            .map(|(_, slice)| {
                let layout = slice.layout();
                let buffer = ptr::slice_from_raw_parts(slice.as_ptr(), layout.size());
                ErasedFieldPtr::new(layout, buffer)
            });
        ErasedSoaPtrs::new(ptrs)
    }

    fn mut_slice_refs_as_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        assert_eq!(field_layouts.len(), slices.fields().len());

        let ptrs = field_layouts
            .iter()
            .zip(slices.into_fields())
            .inspect(|(&field_layout, slice)| assert_layouts(field_layout, slice.layout()))
            .map(|(_, mut slice)| {
                let layout = slice.layout();
                let buffer = ptr::slice_from_raw_parts_mut(slice.as_mut_ptr(), layout.size());
                ErasedFieldMutPtr::new(layout, buffer)
            });
        ErasedSoaMutPtrs::new(ptrs)
    }

    unsafe fn slices_drop_in_place(context: &Self::Context, slices: Self::SliceMutPtrs) {
        let ErasedSoaContext {
            field_layouts,
            drop_fields,
            ..
        } = context;
        let Some(drop_fields) = drop_fields else {
            return;
        };
        assert_eq!(field_layouts.len(), slices.fields().len());

        slices.into_iter().for_each(|ptrs| {
            let layouts = ptrs.fields().iter().map(ErasedFieldMutPtr::layout);
            assert!(field_layouts.iter().copied().eq(layouts));

            drop_fields(ptrs.fields())
        })
    }
}
