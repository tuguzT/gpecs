use alloc::{boxed::Box, vec::Vec};
use core::{
    alloc::Layout,
    iter,
    marker::PhantomData,
    ptr::{self, NonNull},
    slice,
};

use crate::traits::Soa;

type DynField = Box<[u8]>;

#[derive(Debug, Clone)]
pub struct DynSoa<SizeAlign> {
    fields: Box<[DynField]>,
    phantom: PhantomData<fn() -> SizeAlign>,
}

#[derive(Debug, Clone)]
pub struct DynSoaContext<SizeAlign> {
    field_layouts: Box<[Layout]>,
    phantom: PhantomData<fn() -> SizeAlign>,
}

type DynFieldPtr = *const [u8];

#[derive(Debug)]
pub struct DynSoaPtrs<SizeAlign> {
    ptrs: Box<[DynFieldPtr]>,
    phantom: PhantomData<fn() -> SizeAlign>,
}

impl<SizeAlign> Clone for DynSoaPtrs<SizeAlign> {
    fn clone(&self) -> Self {
        Self {
            ptrs: self.ptrs.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

type DynFieldPtrMut = *mut [u8];

#[derive(Debug)]
pub struct DynSoaMutPtrs<SizeAlign> {
    ptrs: Box<[DynFieldPtrMut]>,
    phantom: PhantomData<fn() -> SizeAlign>,
}

impl<SizeAlign> Clone for DynSoaMutPtrs<SizeAlign> {
    fn clone(&self) -> Self {
        Self {
            ptrs: self.ptrs.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

type DynFieldNonNullPtr = NonNull<[u8]>;

#[derive(Debug)]
pub struct DynSoaNonNullPtrs<SizeAlign> {
    ptrs: Box<[DynFieldNonNullPtr]>,
    phantom: PhantomData<fn() -> SizeAlign>,
}

impl<SizeAlign> Clone for DynSoaNonNullPtrs<SizeAlign> {
    fn clone(&self) -> Self {
        Self {
            ptrs: self.ptrs.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

// data is stored inline in a single buffer
type DynFieldVec = Vec<u8>;

#[derive(Debug)]
pub struct DynSoaVecs<SizeAlign> {
    vecs: Box<[DynFieldVec]>,
    phantom: PhantomData<fn() -> SizeAlign>,
}

impl<SizeAlign> Clone for DynSoaVecs<SizeAlign> {
    fn clone(&self) -> Self {
        Self {
            vecs: self.vecs.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

type DynFieldRef<'a> = &'a [u8];

#[derive(Debug)]
pub struct DynSoaRefs<'a, SizeAlign>
where
    SizeAlign: 'a,
{
    refs: Box<[DynFieldRef<'a>]>,
    phantom: PhantomData<fn() -> SizeAlign>,
}

impl<'a, SizeAlign> Clone for DynSoaRefs<'a, SizeAlign> {
    fn clone(&self) -> Self {
        Self {
            refs: self.refs.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

type DynFieldRefMut<'a> = &'a mut [u8];

#[derive(Debug)]
pub struct DynSoaRefsMut<'a, SizeAlign>
where
    SizeAlign: 'a,
{
    refs: Box<[DynFieldRefMut<'a>]>,
    phantom: PhantomData<fn() -> SizeAlign>,
}

// data is stored inline in a single buffer
type DynFieldSlicePtr = *const [u8];

#[derive(Debug)]
pub struct DynSoaSlicePtrs<SizeAlign> {
    slices: Box<[DynFieldSlicePtr]>,
    phantom: PhantomData<fn() -> SizeAlign>,
}

impl<SizeAlign> Clone for DynSoaSlicePtrs<SizeAlign> {
    fn clone(&self) -> Self {
        Self {
            slices: self.slices.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

// data is stored inline in a single buffer
type DynFieldSliceMutPtr = *mut [u8];

#[derive(Debug)]
pub struct DynSoaSliceMutPtrs<SizeAlign> {
    slices: Box<[DynFieldSliceMutPtr]>,
    phantom: PhantomData<fn() -> SizeAlign>,
}

impl<SizeAlign> Clone for DynSoaSliceMutPtrs<SizeAlign> {
    fn clone(&self) -> Self {
        Self {
            slices: self.slices.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

// data is stored inline in a single buffer
type DynFieldSliceRef<'a> = &'a [u8];

#[derive(Debug)]
pub struct DynSoaSlices<'a, SizeAlign>
where
    SizeAlign: 'a,
{
    slices: Box<[DynFieldSliceRef<'a>]>,
    phantom: PhantomData<fn() -> SizeAlign>,
}

impl<'a, SizeAlign> Clone for DynSoaSlices<'a, SizeAlign> {
    fn clone(&self) -> Self {
        Self {
            slices: self.slices.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

// data is stored inline in a single buffer
type DynFieldSliceRefMut<'a> = &'a mut [u8];

#[derive(Debug)]
pub struct DynSoaSlicesMut<'a, SizeAlign>
where
    SizeAlign: 'a,
{
    slices: Box<[DynFieldSliceRefMut<'a>]>,
    phantom: PhantomData<fn() -> SizeAlign>,
}

unsafe impl<SizeAlign> Soa for DynSoa<SizeAlign> {
    type SizeAlign = SizeAlign;

    type Context = DynSoaContext<SizeAlign>;

    type FieldLayouts<'a> = &'a [Layout];

    fn field_layouts(context: &Self::Context) -> Self::FieldLayouts<'_> {
        let DynSoaContext { field_layouts, .. } = context;
        field_layouts.as_ref()
    }

    type Ptrs = DynSoaPtrs<SizeAlign>;

    type MutPtrs = DynSoaMutPtrs<SizeAlign>;

    unsafe fn ptrs(
        context: &Self::Context,
        ptr: *mut u8,
        offsets: impl IntoIterator<Item = usize>,
    ) -> Self::MutPtrs {
        let DynSoaContext { field_layouts, .. } = context;

        let ptrs: Box<[_]> = field_layouts
            .iter()
            .zip(offsets)
            .map(|(field_layout, offset)| unsafe {
                let data = ptr.add(offset);
                let len = field_layout.size();
                ptr::slice_from_raw_parts_mut(data, len)
            })
            .collect();
        assert_eq!(field_layouts.len(), ptrs.len());

        DynSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn ptrs_dangling(context: &Self::Context) -> Self::MutPtrs {
        let DynSoaContext { field_layouts, .. } = context;

        let ptrs = field_layouts
            .iter()
            .map(|field_layout| {
                let data = ptr::without_provenance_mut(field_layout.align());
                let len = field_layout.size();
                ptr::slice_from_raw_parts_mut(data, len)
            })
            .collect();
        DynSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn ptrs_cast_const(_: &Self::Context, ptrs: Self::MutPtrs) -> Self::Ptrs {
        let DynSoaMutPtrs { ptrs, .. } = ptrs;

        let ptrs = ptrs
            .into_vec()
            .into_iter()
            .map(|ptr| ptr.cast_const())
            .collect();
        DynSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn ptrs_cast_mut(_: &Self::Context, ptrs: Self::Ptrs) -> Self::MutPtrs {
        let DynSoaPtrs { ptrs, .. } = ptrs;

        let ptrs = ptrs
            .into_vec()
            .into_iter()
            .map(|ptr| ptr.cast_mut())
            .collect();
        DynSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    unsafe fn ptrs_add(context: &Self::Context, ptrs: Self::Ptrs, offset: usize) -> Self::Ptrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(field_layout.size(), ptr.len());

                let count = offset * field_layout.pad_to_align().size();
                let data = unsafe { ptr.cast::<u8>().add(count) };
                let len = field_layout.size();
                ptr::slice_from_raw_parts(data, len)
            })
            .collect();
        DynSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    unsafe fn ptrs_add_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        offset: usize,
    ) -> Self::MutPtrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(field_layout.size(), ptr.len());

                let count = offset * field_layout.pad_to_align().size();
                let data = unsafe { ptr.cast::<u8>().add(count) };
                let len = field_layout.size();
                ptr::slice_from_raw_parts_mut(data, len)
            })
            .collect();
        DynSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    unsafe fn ptrs_offset_from(
        context: &Self::Context,
        ptrs: Self::Ptrs,
        origin: Self::Ptrs,
    ) -> isize {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaPtrs { ptrs, .. } = ptrs;
        let DynSoaPtrs { ptrs: origin, .. } = origin;

        assert_eq!(field_layouts.len(), ptrs.len());
        assert_eq!(ptrs.len(), origin.len());

        let mut offsets =
            field_layouts
                .iter()
                .zip(ptrs)
                .zip(origin)
                .map(|((field_layout, ptr), origin)| {
                    assert_eq!(field_layout.size(), ptr.len());
                    assert_eq!(ptr.len(), origin.len());

                    let offset = unsafe { ptr.cast::<u8>().offset_from(origin.cast()) };
                    let field_size = field_layout
                        .size()
                        .try_into()
                        .expect("size of layout should be less than `isize::MAX`");
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
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaMutPtrs { ptrs, .. } = ptrs;
        let DynSoaPtrs { ptrs: origin, .. } = origin;

        assert_eq!(field_layouts.len(), ptrs.len());
        assert_eq!(ptrs.len(), origin.len());

        let mut offsets =
            field_layouts
                .iter()
                .zip(ptrs)
                .zip(origin)
                .map(|((field_layout, ptr), origin)| {
                    assert_eq!(field_layout.size(), ptr.len());
                    assert_eq!(ptr.len(), origin.len());

                    let offset = unsafe { ptr.cast::<u8>().offset_from(origin.cast()) };
                    let field_size = field_layout
                        .size()
                        .try_into()
                        .expect("size of layout should be less than `isize::MAX`");
                    offset
                        .checked_div(field_size)
                        .expect("self should not be a ZST")
                });

        let offset = offsets.next().expect("self should not be a ZST");
        assert!(offsets.all(|item| item == offset));
        offset
    }

    unsafe fn ptrs_swap(context: &Self::Context, a: Self::MutPtrs, b: Self::MutPtrs) {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaMutPtrs { ptrs: a, .. } = a;
        let DynSoaMutPtrs { ptrs: b, .. } = b;

        assert_eq!(field_layouts.len(), a.len());
        assert_eq!(a.len(), b.len());

        let mut temp = Vec::new();
        for ((field_layout, a), b) in field_layouts.iter().zip(a).zip(b) {
            assert_eq!(field_layout.size(), a.len());
            assert_eq!(a.len(), b.len());

            let a = a.cast::<u8>();
            let b = b.cast();

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
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaPtrs { ptrs: src, .. } = src;
        let DynSoaMutPtrs { ptrs: dst, .. } = dst;

        assert_eq!(field_layouts.len(), src.len());
        assert_eq!(src.len(), dst.len());

        let mut temp = Vec::new();
        for ((field_layout, src), dst) in field_layouts.iter().zip(src).zip(dst) {
            assert_eq!(field_layout.size(), src.len());
            assert_eq!(src.len(), dst.len());

            let src = src.cast::<u8>();
            let dst = dst.cast();

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
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaPtrs { ptrs: src, .. } = src;
        let DynSoaMutPtrs { ptrs: dst, .. } = dst;

        assert_eq!(field_layouts.len(), src.len());
        assert_eq!(src.len(), dst.len());

        let mut temp = Vec::new();
        for ((field_layout, src), dst) in field_layouts.iter().zip(src).zip(dst).rev() {
            assert_eq!(field_layout.size(), src.len());
            assert_eq!(src.len(), dst.len());

            let src = src.cast::<u8>();
            let dst = dst.cast();

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
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaPtrs { ptrs: src, .. } = src;
        let DynSoaMutPtrs { ptrs: dst, .. } = dst;

        assert_eq!(field_layouts.len(), src.len());
        assert_eq!(src.len(), dst.len());

        for ((field_layout, src), dst) in field_layouts.iter().zip(src).zip(dst) {
            assert_eq!(field_layout.size(), src.len());
            assert_eq!(src.len(), dst.len());

            let src = src.cast::<u8>();
            let dst = dst.cast();

            let len = len * field_layout.size();
            unsafe {
                ptr::copy_nonoverlapping(src, dst, len);
            }
        }
    }

    unsafe fn ptrs_read(context: &Self::Context, src: Self::Ptrs) -> Self {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaPtrs { ptrs: src, .. } = src;

        assert_eq!(field_layouts.len(), src.len());

        let fields = field_layouts
            .iter()
            .zip(src)
            .map(|(field_layout, ptr)| {
                assert_eq!(field_layout.size(), ptr.len());

                let len = ptr.len();
                let mut field = Box::new_uninit_slice(len);
                unsafe {
                    ptr::copy_nonoverlapping(ptr.cast(), field.as_mut_ptr(), len);
                    field.assume_init()
                }
            })
            .collect();
        Self {
            fields,
            phantom: PhantomData,
        }
    }

    unsafe fn ptrs_write(context: &Self::Context, dst: Self::MutPtrs, value: Self) {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaMutPtrs { ptrs: dst, .. } = dst;
        let Self { fields, .. } = value;

        assert_eq!(field_layouts.len(), dst.len());
        assert_eq!(dst.len(), fields.len());

        for ((field_layout, dst), field) in field_layouts.iter().zip(dst).zip(fields) {
            assert_eq!(field_layout.size(), dst.len());
            assert_eq!(dst.len(), field.len());

            let len = field_layout.size();
            unsafe {
                ptr::copy_nonoverlapping(field.as_ptr(), dst.cast(), len);
            }
        }
    }

    unsafe fn ptrs_drop_in_place(context: &Self::Context, ptrs: Self::MutPtrs) {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());
        // TODO: call drop function pointers (when they are added in context)
    }

    type NonNullPtrs = DynSoaNonNullPtrs<SizeAlign>;

    unsafe fn ptrs_to_nonnull(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(field_layout.size(), ptr.len());
                unsafe { NonNull::new_unchecked(ptr) }
            })
            .collect();
        DynSoaNonNullPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn nonnull_to_ptrs(context: &Self::Context, ptrs: Self::NonNullPtrs) -> Self::MutPtrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaNonNullPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(field_layout.size(), ptr.len());
                ptr.as_ptr()
            })
            .collect();
        DynSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    type Vecs = DynSoaVecs<SizeAlign>;

    fn vecs_with_capacity(context: &Self::Context, capacity: usize) -> Self::Vecs {
        let DynSoaContext { field_layouts, .. } = context;

        let vecs = iter::repeat_n(Vec::with_capacity(capacity), field_layouts.len()).collect();
        DynSoaVecs {
            vecs,
            phantom: PhantomData,
        }
    }

    fn vecs_as_ptrs(context: &Self::Context, vecs: &Self::Vecs) -> Self::Ptrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaVecs { vecs, .. } = vecs;

        assert_eq!(field_layouts.len(), vecs.len());

        let ptrs = field_layouts
            .iter()
            .zip(vecs)
            .map(|(field_layout, vec)| {
                assert_eq!(vec.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
                ptr::from_ref(vec.as_slice())
            })
            .collect();
        DynSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn mut_vecs_as_ptrs(context: &Self::Context, vecs: &mut Self::Vecs) -> Self::MutPtrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaVecs { vecs, .. } = vecs;

        assert_eq!(field_layouts.len(), vecs.len());

        let ptrs = field_layouts
            .iter()
            .zip(vecs)
            .map(|(field_layout, vec)| {
                assert_eq!(vec.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
                ptr::from_mut(vec.as_mut_slice())
            })
            .collect();
        DynSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn vecs_len(context: &Self::Context, vecs: &Self::Vecs) -> usize {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaVecs { vecs, .. } = vecs;

        assert_eq!(field_layouts.len(), vecs.len());

        let mut lens = vecs.iter().map(Vec::len);
        let len = lens.next().unwrap_or(0);
        assert!(lens.all(|item| item == len));
        len
    }

    unsafe fn vecs_set_len(context: &Self::Context, vecs: &mut Self::Vecs, len: usize) {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaVecs { vecs, .. } = vecs;

        assert_eq!(field_layouts.len(), vecs.len());

        for vec in vecs {
            unsafe {
                vec.set_len(len);
            }
        }
    }

    type Refs<'a>
        = DynSoaRefs<'a, SizeAlign>
    where
        Self: 'a;

    type RefsMut<'a>
        = DynSoaRefsMut<'a, SizeAlign>
    where
        Self: 'a;

    unsafe fn ptrs_to_refs<'a>(context: &Self::Context, ptrs: Self::Ptrs) -> Self::Refs<'a> {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let refs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(field_layout.size(), ptr.len());
                unsafe { slice::from_raw_parts(ptr.cast(), ptr.len()) }
            })
            .collect();
        DynSoaRefs {
            refs,
            phantom: PhantomData,
        }
    }

    unsafe fn ptrs_to_refs_mut<'a>(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
    ) -> Self::RefsMut<'a> {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let refs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(field_layout.size(), ptr.len());
                unsafe { slice::from_raw_parts_mut(ptr.cast(), ptr.len()) }
            })
            .collect();
        DynSoaRefsMut {
            refs,
            phantom: PhantomData,
        }
    }

    fn refs_as_ptrs(context: &Self::Context, refs: Self::Refs<'_>) -> Self::Ptrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaRefs { refs, .. } = refs;

        assert_eq!(field_layouts.len(), refs.len());

        let ptrs = field_layouts
            .iter()
            .zip(refs)
            .map(|(field_layout, r#ref)| {
                assert_eq!(field_layout.size(), r#ref.len());
                ptr::from_ref(r#ref)
            })
            .collect();
        DynSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn mut_refs_as_ptrs(context: &Self::Context, refs: Self::RefsMut<'_>) -> Self::MutPtrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaRefsMut { refs, .. } = refs;

        assert_eq!(field_layouts.len(), refs.len());

        let ptrs = field_layouts
            .iter()
            .zip(refs)
            .map(|(field_layout, r#ref)| {
                assert_eq!(field_layout.size(), r#ref.len());
                ptr::from_mut(r#ref)
            })
            .collect();
        DynSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn mut_refs_as_refs<'a>(context: &Self::Context, refs: Self::RefsMut<'a>) -> Self::Refs<'a> {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaRefsMut { refs, .. } = refs;

        assert_eq!(field_layouts.len(), refs.len());

        let refs = field_layouts
            .iter()
            .zip(refs)
            .map(|(field_layout, r#ref)| {
                assert_eq!(field_layout.size(), r#ref.len());
                &*r#ref
            })
            .collect();
        DynSoaRefs {
            refs,
            phantom: PhantomData,
        }
    }

    type SlicePtrs = DynSoaSlicePtrs<SizeAlign>;

    type SliceMutPtrs = DynSoaSliceMutPtrs<SizeAlign>;

    fn slices_from_raw_parts(
        context: &Self::Context,
        ptrs: Self::Ptrs,
        len: usize,
    ) -> Self::SlicePtrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let slices = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(field_layout.size(), ptr.len());

                let data = ptr.cast();
                let len = len * field_layout.size();
                ptr::slice_from_raw_parts(data, len)
            })
            .collect();
        DynSoaSlicePtrs {
            slices,
            phantom: PhantomData,
        }
    }

    fn slices_from_raw_parts_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
        len: usize,
    ) -> Self::SliceMutPtrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let slices = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(field_layout.size(), ptr.len());

                let data = ptr.cast();
                let len = len * field_layout.size();
                ptr::slice_from_raw_parts_mut(data, len)
            })
            .collect();
        DynSoaSliceMutPtrs {
            slices,
            phantom: PhantomData,
        }
    }

    fn slice_ptrs_cast_const(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicePtrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSliceMutPtrs { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
                slice.cast_const()
            })
            .collect();
        DynSoaSlicePtrs {
            slices,
            phantom: PhantomData,
        }
    }

    fn slice_ptrs_cast_mut(context: &Self::Context, slices: Self::SlicePtrs) -> Self::SliceMutPtrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSlicePtrs { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
                slice.cast_mut()
            })
            .collect();
        DynSoaSliceMutPtrs {
            slices,
            phantom: PhantomData,
        }
    }

    fn slice_ptrs_len(context: &Self::Context, slices: Self::SlicePtrs) -> usize {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSlicePtrs { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let mut lens = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
                slice.len().checked_div(field_layout.size()).unwrap_or(0)
            });
        let len = lens.next().unwrap_or(0);
        assert!(lens.all(|item| item == len));
        len
    }

    fn slice_ptrs_len_mut(context: &Self::Context, slices: Self::SliceMutPtrs) -> usize {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSliceMutPtrs { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let mut lens = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
                slice.len().checked_div(field_layout.size()).unwrap_or(0)
            });
        let len = lens.next().unwrap_or(0);
        assert!(lens.all(|item| item == len));
        len
    }

    fn slice_ptrs_as_ptrs(context: &Self::Context, slices: Self::SlicePtrs) -> Self::Ptrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSlicePtrs { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.cast();
                let len = slice.len().checked_div(field_layout.size()).unwrap_or(0);
                ptr::slice_from_raw_parts(data, len)
            })
            .collect();
        DynSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn mut_slice_ptrs_as_ptrs(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::MutPtrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSliceMutPtrs { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.cast();
                let len = slice.len().checked_div(field_layout.size()).unwrap_or(0);
                ptr::slice_from_raw_parts_mut(data, len)
            })
            .collect();
        DynSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    type Slices<'a>
        = DynSoaSlices<'a, SizeAlign>
    where
        Self: 'a;

    type SlicesMut<'a>
        = DynSoaSlicesMut<'a, SizeAlign>
    where
        Self: 'a;

    unsafe fn slice_ptrs_to_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicePtrs,
    ) -> Self::Slices<'a> {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSlicePtrs { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
                unsafe { slice::from_raw_parts(slice.cast(), slice.len()) }
            })
            .collect();
        DynSoaSlices {
            slices,
            phantom: PhantomData,
        }
    }

    unsafe fn slice_ptrs_to_slices_mut<'a>(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicesMut<'a> {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSliceMutPtrs { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
                unsafe { slice::from_raw_parts_mut(slice.cast(), slice.len()) }
            })
            .collect();
        DynSoaSlicesMut {
            slices,
            phantom: PhantomData,
        }
    }

    fn slices_len(context: &Self::Context, slices: &Self::Slices<'_>) -> usize {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSlices { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let mut lens = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
                slice.len().checked_div(field_layout.size()).unwrap_or(0)
            });
        let len = lens.next().unwrap_or(0);
        assert!(lens.all(|item| item == len));
        len
    }

    fn slices_len_mut(context: &Self::Context, slices: &Self::SlicesMut<'_>) -> usize {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSlicesMut { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let mut lens = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
                slice.len().checked_div(field_layout.size()).unwrap_or(0)
            });
        let len = lens.next().unwrap_or(0);
        assert!(lens.all(|item| item == len));
        len
    }

    fn slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::Slices<'_>,
    ) -> Self::SlicePtrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSlices { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
                ptr::from_ref(slice)
            })
            .collect();
        DynSoaSlicePtrs {
            slices,
            phantom: PhantomData,
        }
    }

    fn mut_slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::SliceMutPtrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSlicesMut { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
                ptr::from_mut(slice)
            })
            .collect();
        DynSoaSliceMutPtrs {
            slices,
            phantom: PhantomData,
        }
    }

    fn mut_slices_as_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicesMut<'a>,
    ) -> Self::Slices<'a> {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSlicesMut { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
                &*slice
            })
            .collect();
        DynSoaSlices {
            slices,
            phantom: PhantomData,
        }
    }

    fn slice_refs_as_ptrs(context: &Self::Context, slices: Self::Slices<'_>) -> Self::Ptrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSlices { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
                ptr::slice_from_raw_parts(slice.as_ptr(), field_layout.size())
            })
            .collect();
        DynSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn mut_slice_refs_as_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::MutPtrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSlicesMut { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
                ptr::slice_from_raw_parts_mut(slice.as_mut_ptr(), field_layout.size())
            })
            .collect();
        DynSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    unsafe fn slices_drop_in_place(context: &Self::Context, slices: Self::SliceMutPtrs) {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSliceMutPtrs { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());
        // TODO: call drop function pointers on all the fields (when they are added in context)
    }
}
