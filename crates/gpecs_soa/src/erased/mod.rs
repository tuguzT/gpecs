use alloc::{
    boxed::Box,
    vec::{self, Vec},
};
use core::{
    alloc::{Layout, LayoutError},
    iter,
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
};

use crate::traits::{buffer_layout, Soa};

use self::{
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
        let ErasedSoaPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        ptrs.into_vec().into_iter().map(ErasedFieldPtr::into_ptr)
    }

    fn ptrs_erase_mut(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::ErasedMutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        ptrs.into_vec().into_iter().map(ErasedFieldMutPtr::into_ptr)
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

        ErasedSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
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

        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn ptrs_dangling(context: &Self::Context) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;

        let ptrs = field_layouts
            .iter()
            .copied()
            .map(ErasedFieldMutPtr::dangling)
            .collect();
        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn ptrs_cast_const(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(*field_layout, ptr.layout());

                let buffer = ptr.buffer().cast_const();
                ErasedFieldPtr::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn ptrs_cast_mut(context: &Self::Context, ptrs: Self::Ptrs) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, field_ptr)| {
                assert_eq!(*field_layout, field_ptr.layout());

                let buffer = field_ptr.buffer().cast_mut();
                ErasedFieldMutPtr::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    unsafe fn ptrs_add(context: &Self::Context, ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(*field_layout, ptr.layout());

                let data = unsafe { ptr.as_ptr().add(offset * field_layout.size()) };
                let len = field_layout.size();
                let buffer = ptr::slice_from_raw_parts(data, len);
                ErasedFieldPtr::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    unsafe fn ptrs_add_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        offset: usize,
    ) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(*field_layout, ptr.layout());

                let data = unsafe { ptr.as_ptr().add(offset * field_layout.size()) };
                let len = field_layout.size();
                let buffer = ptr::slice_from_raw_parts_mut(data, len);
                ErasedFieldMutPtr::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    unsafe fn ptrs_offset_from(
        context: &Self::Context,
        ptrs: Self::Ptrs,
        origin: Self::Ptrs,
    ) -> isize {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs, .. } = ptrs;
        let ErasedSoaPtrs { ptrs: origin, .. } = origin;

        assert_eq!(field_layouts.len(), ptrs.len());
        assert_eq!(ptrs.len(), origin.len());

        let mut offsets =
            field_layouts
                .iter()
                .zip(ptrs)
                .zip(origin)
                .map(|((field_layout, ptr), origin)| {
                    assert_eq!(*field_layout, ptr.layout());
                    assert_eq!(*field_layout, origin.layout());

                    let offset = unsafe { ptr.as_ptr().offset_from(origin.as_ptr()) };
                    let field_size = field_layout
                        .size()
                        .try_into()
                        .expect("layout size should not exceed `isize::MAX`");
                    offset
                        .checked_div(field_size)
                        .expect("self should not be a ZST")
                });

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
        let ErasedSoaMutPtrs { ptrs, .. } = ptrs;
        let ErasedSoaPtrs { ptrs: origin, .. } = origin;

        assert_eq!(field_layouts.len(), ptrs.len());
        assert_eq!(ptrs.len(), origin.len());

        let mut offsets =
            field_layouts
                .iter()
                .zip(ptrs)
                .zip(origin)
                .map(|((field_layout, ptr), origin)| {
                    assert_eq!(*field_layout, ptr.layout());
                    assert_eq!(*field_layout, origin.layout());

                    let offset = unsafe { ptr.as_ptr().offset_from(origin.as_ptr()) };
                    let field_size = field_layout
                        .size()
                        .try_into()
                        .expect("layout size should not exceed `isize::MAX`");
                    offset
                        .checked_div(field_size)
                        .expect("self should not be a ZST")
                });

        let offset = offsets.next().expect("self should not be a ZST");
        assert!(offsets.all(|item| item == offset));
        offset
    }

    unsafe fn ptrs_swap(context: &Self::Context, a: Self::MutPtrs, b: Self::MutPtrs) {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs: a, .. } = a;
        let ErasedSoaMutPtrs { ptrs: b, .. } = b;

        assert_eq!(field_layouts.len(), a.len());
        assert_eq!(a.len(), b.len());

        let mut temp = Vec::new();
        for ((field_layout, a), b) in field_layouts.iter().zip(a).zip(b) {
            assert_eq!(*field_layout, a.layout());
            assert_eq!(*field_layout, b.layout());

            let a = a.as_ptr();
            let b = b.as_ptr();

            let len = field_layout.size();
            temp.reserve(len);
            unsafe {
                ptr::copy_nonoverlapping(a, temp.as_mut_ptr(), len);
                temp.set_len(len);

                ptr::copy(b, a, len);
                ptr::copy_nonoverlapping(temp.as_ptr(), b, len);
            }
            temp.clear();
        }
    }

    unsafe fn ptrs_copy(context: &Self::Context, src: Self::Ptrs, dst: Self::MutPtrs, len: usize) {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs: src, .. } = src;
        let ErasedSoaMutPtrs { ptrs: dst, .. } = dst;

        assert_eq!(field_layouts.len(), src.len());
        assert_eq!(src.len(), dst.len());

        let mut temp = Vec::new();
        for ((field_layout, src), dst) in field_layouts.iter().zip(src).zip(dst) {
            assert_eq!(*field_layout, src.layout());
            assert_eq!(*field_layout, dst.layout());

            let src = src.as_ptr();
            let dst = dst.as_ptr();

            let len = len * field_layout.size();
            temp.reserve(len);
            unsafe {
                ptr::copy_nonoverlapping(src, temp.as_mut_ptr(), len);
                temp.set_len(len);

                ptr::copy_nonoverlapping(temp.as_ptr(), dst, len);
            }
            temp.clear();
        }
    }

    unsafe fn ptrs_copy_rev(
        context: &Self::Context,
        src: Self::Ptrs,
        dst: Self::MutPtrs,
        len: usize,
    ) {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs: src, .. } = src;
        let ErasedSoaMutPtrs { ptrs: dst, .. } = dst;

        assert_eq!(field_layouts.len(), src.len());
        assert_eq!(src.len(), dst.len());

        let mut temp = Vec::new();
        for ((field_layout, src), dst) in field_layouts.iter().zip(src).zip(dst).rev() {
            assert_eq!(*field_layout, src.layout());
            assert_eq!(*field_layout, dst.layout());

            let src = src.as_ptr();
            let dst = dst.as_ptr();

            let len = len * field_layout.size();
            temp.reserve(len);
            unsafe {
                ptr::copy_nonoverlapping(src, temp.as_mut_ptr(), len);
                temp.set_len(len);

                ptr::copy_nonoverlapping(temp.as_ptr(), dst, len);
            }
            temp.clear();
        }
    }

    unsafe fn ptrs_copy_nonoverlapping(
        context: &Self::Context,
        src: Self::Ptrs,
        dst: Self::MutPtrs,
        len: usize,
    ) {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs: src, .. } = src;
        let ErasedSoaMutPtrs { ptrs: dst, .. } = dst;

        assert_eq!(field_layouts.len(), src.len());
        assert_eq!(src.len(), dst.len());

        for ((field_layout, src), dst) in field_layouts.iter().zip(src).zip(dst) {
            assert_eq!(*field_layout, src.layout());
            assert_eq!(*field_layout, dst.layout());

            let src = src.as_ptr();
            let dst = dst.as_ptr();

            let len = len * field_layout.size();
            unsafe {
                ptr::copy_nonoverlapping(src, dst, len);
            }
        }
    }

    unsafe fn ptrs_read(context: &Self::Context, src: Self::Ptrs) -> Self {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs: src, .. } = src;
        assert_eq!(field_layouts.len(), src.len());

        let (buffer_layout, offsets) =
            Self::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Fields>>());

        let mut buffer = Box::new_uninit_slice(buffer_len);
        for ((field_layout, src), offset) in field_layouts.iter().zip(src).zip(offsets) {
            assert_eq!(*field_layout, src.layout());

            let src = src.as_ptr();
            let dst = unsafe { buffer.as_mut_ptr().cast::<u8>().add(offset) };

            let len = field_layout.size();
            unsafe {
                ptr::copy_nonoverlapping(src, dst, len);
            }
        }
        let buffer = unsafe { buffer.assume_init() };
        Self {
            buffer,
            field_layouts: field_layouts.clone(),
        }
    }

    unsafe fn ptrs_write(context: &Self::Context, dst: Self::MutPtrs, value: Self) {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs: dst, .. } = dst;
        let Self {
            buffer,
            field_layouts: value_layouts,
        } = value;

        assert_eq!(field_layouts.len(), dst.len());
        assert_eq!(field_layouts.as_ref(), value_layouts.as_ref());

        let (buffer_layout, offsets) =
            Self::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout
            .size()
            .div_ceil(size_of::<ErasedByte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        for ((field_layout, dst), offset) in field_layouts.iter().zip(dst).zip(offsets) {
            assert_eq!(*field_layout, dst.layout());

            let src = unsafe { buffer.as_ptr().cast::<u8>().add(offset) };
            let dst = dst.as_ptr();

            let len = field_layout.size();
            unsafe {
                ptr::copy_nonoverlapping(src, dst, len);
            }
        }
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

        let ErasedSoaMutPtrs { ptrs, .. } = ptrs;
        assert!(field_layouts
            .iter()
            .copied()
            .eq(ptrs.iter().map(ErasedFieldMutPtr::layout)));

        drop_fields(ptrs.as_ref());
    }

    type NonNullPtrs = ErasedSoaNonNullPtrs<Fields>;

    unsafe fn ptrs_to_nonnull(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(*field_layout, ptr.layout());

                let buffer = unsafe { NonNull::new_unchecked(ptr.buffer()) };
                ErasedFieldNonNullPtr::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaNonNullPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn nonnull_to_ptrs(context: &Self::Context, ptrs: Self::NonNullPtrs) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaNonNullPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(*field_layout, ptr.layout());

                let buffer = ptr.buffer().as_ptr();
                ErasedFieldMutPtr::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
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
            .map(|(field_layout, vec)| {
                let ErasedFieldVec {
                    buffer,
                    layout: vec_field_layout,
                    ..
                } = vec;
                assert_eq!(field_layout, vec_field_layout);

                let data = buffer.as_ptr().cast();
                let len = field_layout.size();
                let buffer = ptr::slice_from_raw_parts(data, len);
                ErasedFieldPtr::new(field_layout.clone(), buffer)
            })
            .collect();
        ErasedSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn mut_vecs_as_ptrs(context: &Self::Context, vecs: &mut Self::Vecs) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaVecs { vecs, .. } = vecs;

        assert_eq!(field_layouts.len(), vecs.len());

        let ptrs = field_layouts
            .iter()
            .zip(vecs)
            .map(|(field_layout, vec)| {
                let ErasedFieldVec {
                    buffer,
                    layout: vec_field_layout,
                    ..
                } = vec;
                assert_eq!(field_layout, vec_field_layout);

                let data = buffer.as_mut_ptr().cast();
                let len = field_layout.size();
                let buffer = ptr::slice_from_raw_parts_mut(data, len);
                ErasedFieldMutPtr::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
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
        let ErasedSoaPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let refs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(*field_layout, ptr.layout());

                let buffer = unsafe { slice::from_raw_parts(ptr.as_ptr(), field_layout.size()) };
                ErasedFieldRef::new(field_layout.clone(), buffer)
            })
            .collect();
        ErasedSoaRefs {
            refs,
            phantom: PhantomData,
        }
    }

    unsafe fn ptrs_to_refs_mut<'a>(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
    ) -> Self::RefsMut<'a> {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let refs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(*field_layout, ptr.layout());

                let buffer = unsafe { slice::from_raw_parts_mut(ptr.as_ptr(), ptr.buffer().len()) };
                ErasedFieldRefMut::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaRefsMut {
            refs,
            phantom: PhantomData,
        }
    }

    fn refs_as_ptrs(context: &Self::Context, refs: Self::Refs<'_>) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaRefs { refs, .. } = refs;

        assert_eq!(field_layouts.len(), refs.len());

        let ptrs = field_layouts
            .iter()
            .zip(refs)
            .map(|(field_layout, r#ref)| {
                assert_eq!(*field_layout, r#ref.layout());

                let buffer = ptr::from_ref(r#ref.buffer());
                ErasedFieldPtr::new(r#ref.layout(), buffer)
            })
            .collect();
        ErasedSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn mut_refs_as_ptrs(context: &Self::Context, refs: Self::RefsMut<'_>) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaRefsMut { refs, .. } = refs;

        assert_eq!(field_layouts.len(), refs.len());

        let ptrs = field_layouts
            .iter()
            .zip(refs)
            .map(|(field_layout, mut r#ref)| {
                assert_eq!(*field_layout, r#ref.layout());

                let buffer = ptr::from_mut(r#ref.buffer_mut());
                ErasedFieldMutPtr::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn mut_refs_as_refs<'a>(context: &Self::Context, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaRefsMut { refs, .. } = refs;

        assert_eq!(field_layouts.len(), refs.len());

        let refs = field_layouts
            .iter()
            .zip(refs)
            .map(|(field_layout, r#ref)| {
                assert_eq!(*field_layout, r#ref.layout());

                ErasedFieldRef::new(*field_layout, r#ref.into_buffer())
            })
            .collect();
        ErasedSoaRefs {
            refs,
            phantom: PhantomData,
        }
    }

    type SlicePtrs = ErasedSoaSlicePtrs<Fields>;

    type SliceMutPtrs = ErasedSoaSliceMutPtrs<Fields>;

    fn slices_from_raw_parts(
        context: &Self::Context,
        ptrs: Self::Ptrs,
        len: usize,
    ) -> Self::SlicePtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let slices = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(*field_layout, ptr.layout());

                let data = ptr.as_ptr();
                let len = len * field_layout.size();
                let buffer = ptr::slice_from_raw_parts(data, len);
                ErasedFieldSlicePtr::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaSlicePtrs {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn slices_from_raw_parts_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        len: usize,
    ) -> Self::SliceMutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let slices = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(*field_layout, ptr.layout());

                let data = ptr.as_ptr();
                let len = len * field_layout.size();
                let buffer = ptr::slice_from_raw_parts_mut(data, len);
                ErasedFieldSliceMutPtr::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaSliceMutPtrs {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn slice_ptrs_cast_const(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicePtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSliceMutPtrs { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(*field_layout, slice.layout());

                let buffer = slice.buffer().cast_const();
                ErasedFieldSlicePtr::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaSlicePtrs {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn slice_ptrs_cast_mut(context: &Self::Context, slices: Self::SlicePtrs) -> Self::SliceMutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlicePtrs { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(*field_layout, slice.layout());

                let buffer = slice.buffer().cast_mut();
                ErasedFieldSliceMutPtr::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaSliceMutPtrs {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn slice_ptrs_len(context: &Self::Context, slices: Self::SlicePtrs) -> usize {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlicePtrs { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        // let mut lens = field_layouts
        //     .iter()
        //     .zip(slices)
        //     .map(|(field_layout, slice)| {
        //         assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
        //         slice.len().checked_div(field_layout.size()).unwrap_or(0)
        //     });
        // let len = lens.next().unwrap_or(0);
        // assert!(lens.all(|item| item == len));
        len
    }

    fn slice_ptrs_len_mut(context: &Self::Context, slices: Self::SliceMutPtrs) -> usize {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSliceMutPtrs { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        // let mut lens = field_layouts
        //     .iter()
        //     .zip(slices)
        //     .map(|(field_layout, (layout, slice))| {
        //         assert_eq!(field_layout, &layout);
        //         assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

        //         slice.len().checked_div(field_layout.size()).unwrap_or(0)
        //     });
        // let len = lens.next().unwrap_or(0);
        // assert!(lens.all(|item| item == len));
        len
    }

    fn slice_ptrs_as_ptrs(context: &Self::Context, slices: Self::SlicePtrs) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlicePtrs { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(*field_layout, slice.layout());

                let data = slice.as_ptr();
                let len = field_layout.size();
                let buffer = ptr::slice_from_raw_parts(data, len);
                ErasedFieldPtr::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn mut_slice_ptrs_as_ptrs(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSliceMutPtrs { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(*field_layout, slice.layout());

                let data = slice.as_ptr();
                let len = field_layout.size();
                let buffer = ptr::slice_from_raw_parts_mut(data, len);
                ErasedFieldMutPtr::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
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
        let ErasedSoaSlicePtrs { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(*field_layout, slice.layout());

                let data = slice.as_ptr();
                let len = slice.buffer().len();
                let buffer = unsafe { slice::from_raw_parts(data, len) };
                ErasedFieldSlice::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaSlices {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    unsafe fn slice_ptrs_to_slices_mut<'a>(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicesMut<'a> {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSliceMutPtrs { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(*field_layout, slice.layout());

                let data = slice.as_ptr();
                let len = slice.buffer().len();
                let buffer = unsafe { slice::from_raw_parts_mut(data, len) };
                ErasedFieldSliceMut::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaSlicesMut {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn slices_len(context: &Self::Context, slices: &Self::Slices<'_>) -> usize {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlices { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let mut lens = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(*field_layout, slice.layout());

                slice.len()
            });
        let len = lens.next().unwrap_or(0);
        assert!(lens.all(|item| item == len));
        len
    }

    fn slices_len_mut(context: &Self::Context, slices: &Self::SlicesMut<'_>) -> usize {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlicesMut { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let mut lens = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(*field_layout, slice.layout());

                slice.len()
            });
        let len = lens.next().unwrap_or(0);
        assert!(lens.all(|item| item == len));
        len
    }

    fn slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::Slices<'_>,
    ) -> Self::SlicePtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlices { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(*field_layout, slice.layout());

                let buffer = ptr::from_ref(slice.buffer());
                ErasedFieldSlicePtr::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaSlicePtrs {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn mut_slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::SliceMutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlicesMut { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, mut slice)| {
                assert_eq!(*field_layout, slice.layout());

                let buffer = ptr::from_mut(slice.buffer_mut());
                ErasedFieldSliceMutPtr::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaSliceMutPtrs {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn mut_slices_as_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicesMut<'a>,
    ) -> Self::Slices<'a> {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlicesMut { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(*field_layout, slice.layout());

                ErasedFieldSlice::new(*field_layout, slice.into_buffer())
            })
            .collect();
        ErasedSoaSlices {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn slice_refs_as_ptrs(context: &Self::Context, slices: Self::Slices<'_>) -> Self::Ptrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlices { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(*field_layout, slice.layout());

                let data = slice.as_ptr();
                let len = field_layout.size();
                let buffer = ptr::slice_from_raw_parts(data, len);
                ErasedFieldPtr::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn mut_slice_refs_as_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlicesMut { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, mut slice)| {
                assert_eq!(*field_layout, slice.layout());

                let data = slice.as_mut_ptr();
                let len = field_layout.size();
                let buffer = ptr::slice_from_raw_parts_mut(data, len);
                ErasedFieldMutPtr::new(*field_layout, buffer)
            })
            .collect();
        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
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

        for ptrs in slices {
            let ErasedSoaMutPtrs { ptrs, .. } = ptrs;
            assert!(field_layouts
                .iter()
                .copied()
                .eq(ptrs.iter().map(ErasedFieldMutPtr::layout)));

            drop_fields(ptrs.as_ref());
        }
    }
}
