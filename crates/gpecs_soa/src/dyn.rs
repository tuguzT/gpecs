use alloc::{boxed::Box, vec::Vec};
use core::{
    alloc::Layout,
    borrow::Borrow,
    fmt::{self, Debug},
    hash::{self, Hash},
    iter,
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    ptr::{self, NonNull},
    slice,
};

use crate::traits::Soa;

union Byte<Fields> {
    _byte: u8,
    _size_align: ManuallyDrop<MaybeUninit<Fields>>,
}

impl<Fields> Clone for Byte<Fields>
where
    Fields: Copy,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<Fields> Copy for Byte<Fields> where Fields: Copy {}

unsafe impl<Fields> Send for Byte<Fields> where Fields: Send {}
unsafe impl<Fields> Sync for Byte<Fields> where Fields: Sync {}

type DynFields<Fields> = Box<[Byte<Fields>]>;

pub struct DynSoa<Fields> {
    buffer: DynFields<Fields>,
    field_layouts: Box<[Layout]>,
}

impl<Fields> DynSoa<Fields> {
    #[inline]
    pub fn new<'a, I>(context: &DynSoaContext<Fields>, fields: I) -> Self
    where
        I: IntoIterator<Item = &'a [u8]>,
    {
        let DynSoaContext { field_layouts, .. } = context;
        let fields: Box<[_]> = fields.into_iter().collect();
        assert_eq!(field_layouts.len(), fields.len());

        let (buffer_layout, offsets) =
            Self::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout.size().div_ceil(size_of::<Byte<Fields>>());

        let mut buffer = Box::new_uninit_slice(buffer_len);
        let buffer = unsafe {
            for ((field_layout, src), offset) in field_layouts.iter().zip(fields).zip(offsets) {
                assert_eq!(field_layout.size(), src.len());

                let src = src.as_ptr();
                let dst = buffer.as_mut_ptr().cast::<u8>().add(offset);

                let len = field_layout.size();
                ptr::copy_nonoverlapping(src, dst, len);
            }
            buffer.assume_init()
        };
        Self {
            buffer,
            field_layouts: field_layouts.clone(),
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, value: T) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let DynSoaContext { field_layouts, .. } = DynSoaContext::of::<T>(context);

        let (buffer_layout, offsets) =
            T::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout.size().div_ceil(size_of::<Byte<Fields>>());

        let mut buffer = Box::new_uninit_slice(buffer_len);
        let buffer = unsafe {
            let dst = T::ptrs(context, buffer.as_mut_ptr().cast(), offsets);
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
            mut buffer,
            field_layouts,
        } = self;
        let DynSoaContext {
            field_layouts: target_field_layouts,
            ..
        } = DynSoaContext::of::<T>(context);

        assert_eq!(field_layouts.as_ref(), target_field_layouts.as_ref());

        let (buffer_layout, offsets) =
            T::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout.size().div_ceil(size_of::<Byte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        unsafe {
            let src = T::ptrs(context, buffer.as_mut_ptr().cast(), offsets);
            let src = T::ptrs_cast_const(context, src);
            T::ptrs_read(context, src)
        }
    }

    #[inline]
    pub fn layouts(&self) -> &[Layout] {
        let Self { field_layouts, .. } = self;
        field_layouts.as_ref()
    }

    #[inline]
    pub fn as_refs(&self, context: &DynSoaContext<Fields>) -> DynSoaRefs<'_, Fields> {
        let Self {
            buffer,
            field_layouts: value_layouts,
        } = self;
        let DynSoaContext { field_layouts, .. } = context;

        assert_eq!(field_layouts.as_ref(), value_layouts.as_ref());

        let (buffer_layout, offsets) =
            Self::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout.size().div_ceil(size_of::<Byte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        let refs = field_layouts
            .iter()
            .zip(offsets)
            .map(|(field_layout, offset)| unsafe {
                let data = buffer.as_ptr().cast::<u8>().add(offset);
                let len = field_layout.size();
                let r#ref = slice::from_raw_parts(data, len);
                (field_layout.clone(), r#ref)
            })
            .collect();
        DynSoaRefs {
            refs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn as_refs_mut(&mut self, context: &DynSoaContext<Fields>) -> DynSoaRefsMut<'_, Fields> {
        let Self {
            buffer,
            field_layouts: value_layouts,
        } = self;
        let DynSoaContext { field_layouts, .. } = context;

        assert_eq!(field_layouts.as_ref(), value_layouts.as_ref());

        let (buffer_layout, offsets) =
            Self::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout.size().div_ceil(size_of::<Byte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        let refs = field_layouts
            .iter()
            .zip(offsets)
            .map(|(field_layout, offset)| unsafe {
                let data = buffer.as_mut_ptr().cast::<u8>().add(offset);
                let len = field_layout.size();
                let r#ref = slice::from_raw_parts_mut(data, len);
                (field_layout.clone(), r#ref)
            })
            .collect();
        DynSoaRefsMut {
            refs,
            phantom: PhantomData,
        }
    }
}

impl<Fields> Clone for DynSoa<Fields>
where
    Fields: Copy,
{
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            field_layouts: self.field_layouts.clone(),
        }
    }
}

pub struct DynSoaContext<Fields> {
    field_layouts: Box<[Layout]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> DynSoaContext<Fields> {
    #[inline]
    pub fn new<I>(field_layouts: I) -> Self
    where
        I: IntoIterator,
        I::Item: Borrow<Layout>,
    {
        Self {
            field_layouts: collect_layouts::<Fields, I>(field_layouts),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn of<T>(context: &T::Context) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let mut permutation = permutation_of::<T>(context);
        let mut field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));
        apply_permutation(&mut permutation, &mut field_layouts);

        Self {
            field_layouts,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn layouts(&self) -> &[Layout] {
        let Self { field_layouts, .. } = self;
        field_layouts.as_ref()
    }
}

impl<Fields> Debug for DynSoaContext<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DynSoaContext")
            .field(&self.field_layouts)
            .finish()
    }
}

impl<Fields> Clone for DynSoaContext<Fields> {
    fn clone(&self) -> Self {
        Self {
            field_layouts: self.field_layouts.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

#[inline]
fn collect_layouts<Fields, I>(field_layouts: I) -> Box<[Layout]>
where
    I: IntoIterator,
    I::Item: Borrow<Layout>,
{
    field_layouts
        .into_iter()
        .map(|item| {
            let layout: &Layout = item.borrow();

            let input_align = layout.align();
            let max_align = align_of::<Fields>();
            assert!(
                input_align <= max_align,
                "input alignment must be less than or equal to {max_align}, but got {input_align}",
            );
            layout.clone()
        })
        .collect()
}

#[inline]
fn permutation_of<T>(context: &T::Context) -> Box<[usize]>
where
    T: Soa,
{
    let (_, offsets) =
        T::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
    let offsets: Box<[_]> = offsets.into_iter().collect();

    let mut permutation: Box<[_]> = (0..offsets.len()).collect();
    permutation.sort_by_key(|&index| offsets[index]);

    permutation
}

#[inline]
// code was taken from `permutation` crate:
// https://github.com/jeremysalwen/rust-permutations/blob/5528e4fec7c5eb4551cfb39029c8d7982be2e6a4/src/permutation.rs#L400
// dependency was not used because he lack of `#[no_std]` attribute
fn apply_permutation<T>(permutation: &mut [usize], data: &mut [T]) {
    assert_eq!(permutation.len(), data.len());

    const MARKER: usize = isize::MIN as usize;

    for i in 0..permutation.len() {
        let i_idx = permutation[i];
        if (i_idx & MARKER) != 0 {
            continue;
        }

        let mut j = i;
        let mut j_idx = i_idx;
        while j_idx != i {
            permutation[j] = j_idx ^ MARKER;
            data.swap(j, j_idx);
            j = j_idx;
            j_idx = permutation[j];
        }
        permutation[j] = j_idx ^ MARKER;
    }

    for idx in permutation.iter_mut() {
        *idx = *idx ^ MARKER;
    }
}

type DynFieldPtr = *const [u8];

pub struct DynSoaPtrs<Fields> {
    ptrs: Box<[(Layout, DynFieldPtr)]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> DynSoaPtrs<Fields> {
    #[inline]
    pub fn new<I>(context: &DynSoaContext<Fields>, ptrs: I) -> Self
    where
        I: IntoIterator<Item = DynFieldPtr>,
    {
        let DynSoaContext { field_layouts, .. } = context;
        let ptrs: Box<[_]> = ptrs.into_iter().collect();
        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(field_layout.size(), ptr.len());
                (field_layout.clone(), ptr)
            })
            .collect();
        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, ptrs: T::Ptrs) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let ptrs = T::ptrs_erase(context, ptrs);

        let mut permutation = permutation_of::<T>(context);
        let field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));

        let mut ptrs: Box<[_]> = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts(ptr.cast(), len);
                (field_layout.clone(), ptr)
            })
            .collect();
        apply_permutation(&mut permutation, &mut ptrs);

        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::Ptrs
    where
        T: Soa<Fields = Fields>,
    {
        let Self { ptrs, .. } = self;

        let mut permutation = permutation_of::<T>(context);
        assert_eq!(ptrs.len(), permutation.len());

        let mut field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));
        apply_permutation(&mut permutation, &mut field_layouts);

        let mut ptrs: Box<[_]> = ptrs
            .iter()
            .zip(field_layouts)
            .map(|((layout, ptr), field_layout)| {
                assert_eq!(layout, &field_layout);
                ptr.cast()
            })
            .collect();
        apply_permutation(&mut permutation, &mut ptrs);

        T::ptrs_restore(context, ptrs)
    }
}

impl<Fields> AsRef<[(Layout, DynFieldPtr)]> for DynSoaPtrs<Fields> {
    fn as_ref(&self) -> &[(Layout, DynFieldPtr)] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }
}

impl<Fields> AsMut<[(Layout, DynFieldPtr)]> for DynSoaPtrs<Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, DynFieldPtr)] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut()
    }
}

impl<Fields> Debug for DynSoaPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DynSoaPtrs").field(&self.ptrs).finish()
    }
}

impl<Fields> PartialEq for DynSoaPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.ptrs == other.ptrs && self.phantom == other.phantom
    }
}

impl<Fields> Eq for DynSoaPtrs<Fields> {}

impl<Fields> Hash for DynSoaPtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.ptrs.hash(state);
        self.phantom.hash(state);
    }
}

impl<Fields> Clone for DynSoaPtrs<Fields> {
    fn clone(&self) -> Self {
        Self {
            ptrs: self.ptrs.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

type DynFieldMutPtr = *mut [u8];

pub struct DynSoaMutPtrs<Fields> {
    ptrs: Box<[(Layout, DynFieldMutPtr)]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> DynSoaMutPtrs<Fields> {
    pub fn new<I>(context: &DynSoaContext<Fields>, ptrs: I) -> Self
    where
        I: IntoIterator<Item = DynFieldMutPtr>,
    {
        let DynSoaContext { field_layouts, .. } = context;
        let ptrs: Box<[_]> = ptrs.into_iter().collect();
        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(field_layout.size(), ptr.len());
                (field_layout.clone(), ptr)
            })
            .collect();
        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, ptrs: T::MutPtrs) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let ptrs = T::ptrs_erase_mut(context, ptrs);

        let mut permutation = permutation_of::<T>(context);
        let field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));

        let mut ptrs: Box<[_]> = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let data = ptr.cast();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (field_layout.clone(), ptr)
            })
            .collect();
        apply_permutation(&mut permutation, &mut ptrs);

        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::MutPtrs
    where
        T: Soa<Fields = Fields>,
    {
        let Self { ptrs, .. } = self;

        let mut permutation = permutation_of::<T>(context);
        assert_eq!(ptrs.len(), permutation.len());

        let mut field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));
        apply_permutation(&mut permutation, &mut field_layouts);

        let mut ptrs: Box<[_]> = ptrs
            .iter()
            .zip(field_layouts)
            .map(|((layout, ptr), field_layout)| {
                assert_eq!(layout, &field_layout);
                ptr.cast()
            })
            .collect();
        apply_permutation(&mut permutation, &mut ptrs);

        T::ptrs_restore_mut(context, ptrs)
    }
}

impl<Fields> AsRef<[(Layout, DynFieldMutPtr)]> for DynSoaMutPtrs<Fields> {
    fn as_ref(&self) -> &[(Layout, DynFieldMutPtr)] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }
}

impl<Fields> AsMut<[(Layout, DynFieldMutPtr)]> for DynSoaMutPtrs<Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, DynFieldMutPtr)] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut()
    }
}

impl<Fields> Debug for DynSoaMutPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DynSoaMutPtrs").field(&self.ptrs).finish()
    }
}

impl<Fields> PartialEq for DynSoaMutPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.ptrs == other.ptrs && self.phantom == other.phantom
    }
}

impl<Fields> Eq for DynSoaMutPtrs<Fields> {}

impl<Fields> Hash for DynSoaMutPtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.ptrs.hash(state);
        self.phantom.hash(state);
    }
}

impl<Fields> Clone for DynSoaMutPtrs<Fields> {
    fn clone(&self) -> Self {
        Self {
            ptrs: self.ptrs.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

type DynFieldNonNullPtr = NonNull<[u8]>;

pub struct DynSoaNonNullPtrs<Fields> {
    ptrs: Box<[(Layout, DynFieldNonNullPtr)]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> DynSoaNonNullPtrs<Fields> {
    pub fn new<I>(context: &DynSoaContext<Fields>, ptrs: I) -> Self
    where
        I: IntoIterator<Item = DynFieldNonNullPtr>,
    {
        let DynSoaContext { field_layouts, .. } = context;
        let ptrs: Box<[_]> = ptrs.into_iter().collect();
        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                assert_eq!(field_layout.size(), ptr.len());
                (field_layout.clone(), ptr)
            })
            .collect();
        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, ptrs: T::NonNullPtrs) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let ptrs = T::nonnull_to_ptrs(context, ptrs);
        let ptrs = T::ptrs_erase_mut(context, ptrs);

        let mut permutation = permutation_of::<T>(context);
        let field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));

        let mut ptrs: Box<[_]> = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(ptr.cast(), len);
                (field_layout.clone(), unsafe { NonNull::new_unchecked(ptr) })
            })
            .collect();
        apply_permutation(&mut permutation, &mut ptrs);

        Self {
            ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::NonNullPtrs
    where
        T: Soa<Fields = Fields>,
    {
        let Self { ptrs, .. } = self;

        let mut permutation = permutation_of::<T>(context);
        assert_eq!(ptrs.len(), permutation.len());

        let mut field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));
        apply_permutation(&mut permutation, &mut field_layouts);

        let mut ptrs: Box<[_]> = ptrs
            .iter()
            .zip(field_layouts)
            .map(|((layout, ptr), field_layout)| {
                assert_eq!(layout, &field_layout);
                ptr.as_ptr().cast()
            })
            .collect();
        apply_permutation(&mut permutation, &mut ptrs);

        let ptrs = T::ptrs_restore_mut(context, ptrs);
        unsafe { T::ptrs_to_nonnull(context, ptrs) }
    }
}

impl<Fields> AsRef<[(Layout, DynFieldNonNullPtr)]> for DynSoaNonNullPtrs<Fields> {
    fn as_ref(&self) -> &[(Layout, DynFieldNonNullPtr)] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }
}

impl<Fields> AsMut<[(Layout, DynFieldNonNullPtr)]> for DynSoaNonNullPtrs<Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, DynFieldNonNullPtr)] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut()
    }
}

impl<Fields> Debug for DynSoaNonNullPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DynSoaNonNullPtrs")
            .field(&self.ptrs)
            .finish()
    }
}

impl<Fields> PartialEq for DynSoaNonNullPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.ptrs == other.ptrs && self.phantom == other.phantom
    }
}

impl<Fields> Eq for DynSoaNonNullPtrs<Fields> {}

impl<Fields> Hash for DynSoaNonNullPtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.ptrs.hash(state);
        self.phantom.hash(state);
    }
}

impl<Fields> Clone for DynSoaNonNullPtrs<Fields> {
    fn clone(&self) -> Self {
        Self {
            ptrs: self.ptrs.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

// data is stored inline in a single buffer
struct DynFieldVec<Fields> {
    buffer: Vec<Byte<Fields>>,
    layout: Layout,
}

pub struct DynSoaVecs<Fields> {
    len: usize,
    vecs: Box<[DynFieldVec<Fields>]>,
}

type DynFieldRef<'a> = &'a [u8];

pub struct DynSoaRefs<'a, Fields>
where
    Fields: 'a,
{
    refs: Box<[(Layout, DynFieldRef<'a>)]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> DynSoaRefs<'a, Fields> {
    pub fn new<I>(context: &DynSoaContext<Fields>, refs: I) -> Self
    where
        I: IntoIterator<Item = DynFieldRef<'a>>,
    {
        let DynSoaContext { field_layouts, .. } = context;
        let refs: Box<[_]> = refs.into_iter().collect();
        assert_eq!(field_layouts.len(), refs.len());

        let refs = field_layouts
            .iter()
            .zip(refs)
            .map(|(field_layout, r#ref)| {
                assert_eq!(field_layout.size(), r#ref.len());
                (field_layout.clone(), r#ref)
            })
            .collect();
        Self {
            refs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, refs: T::Refs<'a>) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let ptrs = T::refs_as_ptrs(context, refs);
        let ptrs = T::ptrs_erase(context, ptrs);

        let mut permutation = permutation_of::<T>(context);
        let field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));

        let mut refs: Box<[_]> = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| unsafe {
                let len = field_layout.size();
                (field_layout.clone(), slice::from_raw_parts(ptr.cast(), len))
            })
            .collect();
        apply_permutation(&mut permutation, &mut refs);

        Self {
            refs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::Refs<'a>
    where
        T: Soa<Fields = Fields>,
    {
        let Self { refs, .. } = self;

        let mut permutation = permutation_of::<T>(context);
        assert_eq!(refs.len(), permutation.len());

        let mut field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));
        apply_permutation(&mut permutation, &mut field_layouts);

        let mut ptrs: Box<[_]> = refs
            .iter()
            .zip(field_layouts)
            .map(|((layout, r#ref), field_layout)| {
                assert_eq!(layout, &field_layout);
                r#ref.as_ptr()
            })
            .collect();
        apply_permutation(&mut permutation, &mut ptrs);

        let ptrs = T::ptrs_restore(context, ptrs);
        unsafe { T::ptrs_to_refs(context, ptrs) }
    }
}

impl<'a, Fields> AsRef<[(Layout, DynFieldRef<'a>)]> for DynSoaRefs<'a, Fields> {
    fn as_ref(&self) -> &[(Layout, DynFieldRef<'a>)] {
        let Self { refs, .. } = self;
        refs.as_ref()
    }
}

impl<'a, Fields> AsMut<[(Layout, DynFieldRef<'a>)]> for DynSoaRefs<'a, Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, DynFieldRef<'a>)] {
        let Self { refs, .. } = self;
        refs.as_mut()
    }
}

impl<'a, Fields> Debug for DynSoaRefs<'a, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DynSoaRefs").field(&self.refs).finish()
    }
}

impl<'a, Fields> PartialEq for DynSoaRefs<'a, Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.refs == other.refs && self.phantom == other.phantom
    }
}

impl<'a, Fields> Eq for DynSoaRefs<'a, Fields> {}

impl<'a, Fields> Hash for DynSoaRefs<'a, Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.refs.hash(state);
        self.phantom.hash(state);
    }
}

impl<'a, Fields> Clone for DynSoaRefs<'a, Fields> {
    fn clone(&self) -> Self {
        Self {
            refs: self.refs.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

unsafe impl<'a, Fields> Send for DynSoaRefs<'a, Fields> where Fields: Sync {}
unsafe impl<'a, Fields> Sync for DynSoaRefs<'a, Fields> where Fields: Sync {}

type DynFieldRefMut<'a> = &'a mut [u8];

pub struct DynSoaRefsMut<'a, Fields>
where
    Fields: 'a,
{
    refs: Box<[(Layout, DynFieldRefMut<'a>)]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> DynSoaRefsMut<'a, Fields> {
    pub fn new<I>(context: &DynSoaContext<Fields>, refs: I) -> Self
    where
        I: IntoIterator<Item = DynFieldRefMut<'a>>,
    {
        let DynSoaContext { field_layouts, .. } = context;
        let refs: Box<[_]> = refs.into_iter().collect();
        assert_eq!(field_layouts.len(), refs.len());

        let refs = field_layouts
            .iter()
            .zip(refs)
            .map(|(field_layout, r#ref)| {
                assert_eq!(field_layout.size(), r#ref.len());
                (field_layout.clone(), r#ref)
            })
            .collect();
        Self {
            refs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, refs: T::RefsMut<'a>) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let ptrs = T::mut_refs_as_ptrs(context, refs);
        let ptrs = T::ptrs_erase_mut(context, ptrs);

        let mut permutation = permutation_of::<T>(context);
        let field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));

        let mut refs: Box<[_]> = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let data = ptr.cast();
                let len = field_layout.size();
                let r#ref = unsafe { slice::from_raw_parts_mut(data, len) };
                (field_layout.clone(), r#ref)
            })
            .collect();
        apply_permutation(&mut permutation, &mut refs);

        Self {
            refs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::Refs<'a>
    where
        T: Soa<Fields = Fields>,
    {
        let Self { refs, .. } = self;

        let mut permutation = permutation_of::<T>(context);
        assert_eq!(refs.len(), permutation.len());

        let mut field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));
        apply_permutation(&mut permutation, &mut field_layouts);

        let mut ptrs: Box<[_]> = refs
            .iter()
            .zip(field_layouts)
            .map(|((layout, r#ref), field_layout)| {
                assert_eq!(layout, &field_layout);
                r#ref.as_ptr()
            })
            .collect();
        apply_permutation(&mut permutation, &mut ptrs);

        let ptrs = T::ptrs_restore(context, ptrs);
        unsafe { T::ptrs_to_refs(context, ptrs) }
    }
}

impl<'a, Fields> AsRef<[(Layout, DynFieldRefMut<'a>)]> for DynSoaRefsMut<'a, Fields> {
    fn as_ref(&self) -> &[(Layout, DynFieldRefMut<'a>)] {
        let Self { refs, .. } = self;
        refs.as_ref()
    }
}

impl<'a, Fields> AsMut<[(Layout, DynFieldRefMut<'a>)]> for DynSoaRefsMut<'a, Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, DynFieldRefMut<'a>)] {
        let Self { refs, .. } = self;
        refs.as_mut()
    }
}

impl<'a, Fields> Debug for DynSoaRefsMut<'a, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DynSoaRefsMut").field(&self.refs).finish()
    }
}

impl<'a, Fields> PartialEq for DynSoaRefsMut<'a, Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.refs == other.refs && self.phantom == other.phantom
    }
}

impl<'a, Fields> Eq for DynSoaRefsMut<'a, Fields> {}

impl<'a, Fields> Hash for DynSoaRefsMut<'a, Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.refs.hash(state);
        self.phantom.hash(state);
    }
}

unsafe impl<'a, Fields> Send for DynSoaRefsMut<'a, Fields> where Fields: Send {}
unsafe impl<'a, Fields> Sync for DynSoaRefsMut<'a, Fields> where Fields: Sync {}

// data is stored inline in a single buffer
type DynFieldSlicePtr = *const [u8];

pub struct DynSoaSlicePtrs<Fields> {
    len: usize,
    slices: Box<[(Layout, DynFieldSlicePtr)]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> DynSoaSlicePtrs<Fields> {
    pub fn new<I>(context: &DynSoaContext<Fields>, len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = DynFieldSlicePtr>,
    {
        let DynSoaContext { field_layouts, .. } = context;
        let slices: Box<[_]> = slices.into_iter().collect();
        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(
                    slice.len().checked_div(field_layout.size()).unwrap_or(len),
                    len,
                );
                (field_layout.clone(), slice)
            })
            .collect();
        Self {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, slices: T::SlicePtrs) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let len = T::slice_ptrs_len(context, slices.clone());
        let ptrs = T::slice_ptrs_as_ptrs(context, slices);
        let ptrs = T::ptrs_erase(context, ptrs);

        let mut permutation = permutation_of::<T>(context);
        let field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));

        let mut slices: Box<[_]> = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size() * len;
                let slice = ptr::slice_from_raw_parts(ptr.cast(), len);
                (field_layout.clone(), slice)
            })
            .collect();
        apply_permutation(&mut permutation, &mut slices);

        Self {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::SlicePtrs
    where
        T: Soa<Fields = Fields>,
    {
        let Self { slices, len, .. } = self;

        let mut permutation = permutation_of::<T>(context);
        assert_eq!(slices.len(), permutation.len());

        let mut field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));
        apply_permutation(&mut permutation, &mut field_layouts);

        let mut ptrs: Box<[_]> = iter::zip(slices, field_layouts)
            .map(|((layout, slice), field_layout)| {
                assert_eq!(layout, field_layout);
                slice.cast()
            })
            .collect();
        apply_permutation(&mut permutation, &mut ptrs);

        let ptrs = T::ptrs_restore(context, ptrs);
        T::slices_from_raw_parts(context, ptrs, len)
    }
}

impl<Fields> AsRef<[(Layout, DynFieldSlicePtr)]> for DynSoaSlicePtrs<Fields> {
    fn as_ref(&self) -> &[(Layout, DynFieldSlicePtr)] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }
}

impl<Fields> AsMut<[(Layout, DynFieldSlicePtr)]> for DynSoaSlicePtrs<Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, DynFieldSlicePtr)] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }
}

impl<Fields> Debug for DynSoaSlicePtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynSoaSlicePtrs")
            .field("len", &self.len)
            .field("slices", &self.slices)
            .finish()
    }
}

impl<Fields> PartialEq for DynSoaSlicePtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.slices == other.slices && self.phantom == other.phantom
    }
}

impl<Fields> Eq for DynSoaSlicePtrs<Fields> {}

impl<Fields> Hash for DynSoaSlicePtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.len.hash(state);
        self.slices.hash(state);
        self.phantom.hash(state);
    }
}

impl<Fields> Clone for DynSoaSlicePtrs<Fields> {
    fn clone(&self) -> Self {
        Self {
            len: self.len.clone(),
            slices: self.slices.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

// data is stored inline in a single buffer
type DynFieldSliceMutPtr = *mut [u8];

pub struct DynSoaSliceMutPtrs<Fields> {
    len: usize,
    slices: Box<[(Layout, DynFieldSliceMutPtr)]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> DynSoaSliceMutPtrs<Fields> {
    pub fn new<I>(context: &DynSoaContext<Fields>, len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = DynFieldSliceMutPtr>,
    {
        let DynSoaContext { field_layouts, .. } = context;
        let slices: Box<[_]> = slices.into_iter().collect();
        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(
                    slice.len().checked_div(field_layout.size()).unwrap_or(len),
                    len,
                );
                (field_layout.clone(), slice)
            })
            .collect();
        Self {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, slices: T::SliceMutPtrs) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let len = T::slice_ptrs_len_mut(context, slices.clone());
        let ptrs = T::mut_slice_ptrs_as_ptrs(context, slices);
        let ptrs = T::ptrs_erase_mut(context, ptrs);

        let mut permutation = permutation_of::<T>(context);
        let field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));

        let mut slices: Box<[_]> = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size() * len;
                let slice = ptr::slice_from_raw_parts_mut(ptr.cast(), len);
                (field_layout.clone(), slice)
            })
            .collect();
        apply_permutation(&mut permutation, &mut slices);

        Self {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::SliceMutPtrs
    where
        T: Soa<Fields = Fields>,
    {
        let Self { slices, len, .. } = self;

        let mut permutation = permutation_of::<T>(context);
        assert_eq!(slices.len(), permutation.len());

        let mut field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));
        apply_permutation(&mut permutation, &mut field_layouts);

        let mut ptrs: Box<[_]> = iter::zip(slices, field_layouts)
            .map(|((layout, slice), field_layout)| {
                assert_eq!(layout, field_layout);
                slice.cast()
            })
            .collect();
        apply_permutation(&mut permutation, &mut ptrs);

        let ptrs = T::ptrs_restore_mut(context, ptrs);
        T::slices_from_raw_parts_mut(context, ptrs, len)
    }
}

impl<Fields> AsRef<[(Layout, DynFieldSliceMutPtr)]> for DynSoaSliceMutPtrs<Fields> {
    fn as_ref(&self) -> &[(Layout, DynFieldSliceMutPtr)] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }
}

impl<Fields> AsMut<[(Layout, DynFieldSliceMutPtr)]> for DynSoaSliceMutPtrs<Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, DynFieldSliceMutPtr)] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }
}

impl<Fields> Debug for DynSoaSliceMutPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynSoaSliceMutPtrs")
            .field("len", &self.len)
            .field("slices", &self.slices)
            .finish()
    }
}

impl<Fields> PartialEq for DynSoaSliceMutPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.slices == other.slices && self.phantom == other.phantom
    }
}

impl<Fields> Eq for DynSoaSliceMutPtrs<Fields> {}

impl<Fields> Hash for DynSoaSliceMutPtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.len.hash(state);
        self.slices.hash(state);
        self.phantom.hash(state);
    }
}

impl<Fields> Clone for DynSoaSliceMutPtrs<Fields> {
    fn clone(&self) -> Self {
        Self {
            len: self.len.clone(),
            slices: self.slices.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

// data is stored inline in a single buffer
type DynFieldSlice<'a> = &'a [u8];

pub struct DynSoaSlices<'a, Fields>
where
    Fields: 'a,
{
    len: usize,
    slices: Box<[(Layout, DynFieldSlice<'a>)]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> DynSoaSlices<'a, Fields> {
    pub fn new<I>(context: &DynSoaContext<Fields>, len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = DynFieldSlice<'a>>,
    {
        let DynSoaContext { field_layouts, .. } = context;
        let slices: Box<[_]> = slices.into_iter().collect();
        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(
                    slice.len().checked_div(field_layout.size()).unwrap_or(len),
                    len,
                );
                (field_layout.clone(), slice)
            })
            .collect();
        Self {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, slices: T::Slices<'a>) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let len = T::slices_len(context, &slices);
        let ptrs = T::slice_refs_as_ptrs(context, slices);
        let ptrs = T::ptrs_erase(context, ptrs);

        let mut permutation = permutation_of::<T>(context);
        let field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));

        let mut slices: Box<[_]> = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size() * len;
                let slice = unsafe { slice::from_raw_parts(ptr.cast(), len) };
                (field_layout.clone(), slice)
            })
            .collect();
        apply_permutation(&mut permutation, &mut slices);

        Self {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::Slices<'a>
    where
        T: Soa<Fields = Fields>,
    {
        let Self { slices, len, .. } = self;

        let mut permutation = permutation_of::<T>(context);
        assert_eq!(slices.len(), permutation.len());

        let mut field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));
        apply_permutation(&mut permutation, &mut field_layouts);

        let mut ptrs: Box<[_]> = iter::zip(slices, field_layouts)
            .map(|((layout, slice), field_layout)| {
                assert_eq!(layout, field_layout);
                slice.as_ptr()
            })
            .collect();
        apply_permutation(&mut permutation, &mut ptrs);

        let ptrs = T::ptrs_restore(context, ptrs);
        let slices = T::slices_from_raw_parts(context, ptrs, len);
        unsafe { T::slice_ptrs_to_slices(context, slices) }
    }
}

impl<'a, Fields> AsRef<[(Layout, DynFieldSlice<'a>)]> for DynSoaSlices<'a, Fields> {
    fn as_ref(&self) -> &[(Layout, DynFieldSlice<'a>)] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }
}

impl<'a, Fields> AsMut<[(Layout, DynFieldSlice<'a>)]> for DynSoaSlices<'a, Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, DynFieldSlice<'a>)] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }
}

impl<'a, Fields> Debug for DynSoaSlices<'a, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynSoaSlices")
            .field("len", &self.len)
            .field("slices", &self.slices)
            .finish()
    }
}

impl<'a, Fields> PartialEq for DynSoaSlices<'a, Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.slices == other.slices && self.phantom == other.phantom
    }
}

impl<'a, Fields> Eq for DynSoaSlices<'a, Fields> {}

impl<'a, Fields> Hash for DynSoaSlices<'a, Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.len.hash(state);
        self.slices.hash(state);
        self.phantom.hash(state);
    }
}

impl<'a, Fields> Clone for DynSoaSlices<'a, Fields> {
    fn clone(&self) -> Self {
        Self {
            len: self.len.clone(),
            slices: self.slices.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

unsafe impl<'a, Fields> Send for DynSoaSlices<'a, Fields> where Fields: Sync {}
unsafe impl<'a, Fields> Sync for DynSoaSlices<'a, Fields> where Fields: Sync {}

// data is stored inline in a single buffer
type DynFieldSliceMut<'a> = &'a mut [u8];

pub struct DynSoaSlicesMut<'a, Fields>
where
    Fields: 'a,
{
    len: usize,
    slices: Box<[(Layout, DynFieldSliceMut<'a>)]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> DynSoaSlicesMut<'a, Fields> {
    pub fn new<I>(context: &DynSoaContext<Fields>, len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = DynFieldSliceMut<'a>>,
    {
        let DynSoaContext { field_layouts, .. } = context;
        let slices: Box<[_]> = slices.into_iter().collect();
        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, slice)| {
                assert_eq!(
                    slice.len().checked_div(field_layout.size()).unwrap_or(len),
                    len,
                );
                (field_layout.clone(), slice)
            })
            .collect();
        Self {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn from<T>(context: &T::Context, slices: T::SlicesMut<'a>) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let len = T::slices_len_mut(context, &slices);
        let ptrs = T::mut_slice_refs_as_ptrs(context, slices);
        let ptrs = T::ptrs_erase_mut(context, ptrs);

        let mut permutation = permutation_of::<T>(context);
        let field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));

        let mut slices: Box<[_]> = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size() * len;
                let slice = unsafe { slice::from_raw_parts_mut(ptr.cast(), len) };
                (field_layout.clone(), slice)
            })
            .collect();
        apply_permutation(&mut permutation, &mut slices);

        Self {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::SlicesMut<'a>
    where
        T: Soa<Fields = Fields>,
    {
        let Self { slices, len, .. } = self;

        let mut permutation = permutation_of::<T>(context);
        assert_eq!(slices.len(), permutation.len());

        let mut field_layouts = collect_layouts::<Fields, _>(T::field_layouts(context));
        apply_permutation(&mut permutation, &mut field_layouts);

        let mut ptrs: Box<[_]> = iter::zip(slices, field_layouts)
            .map(|((layout, slice), field_layout)| {
                assert_eq!(layout, field_layout);
                slice.as_mut_ptr()
            })
            .collect();
        apply_permutation(&mut permutation, &mut ptrs);

        let ptrs = T::ptrs_restore_mut(context, ptrs);
        let slices = T::slices_from_raw_parts_mut(context, ptrs, len);
        unsafe { T::slice_ptrs_to_slices_mut(context, slices) }
    }
}

impl<'a, Fields> AsRef<[(Layout, DynFieldSliceMut<'a>)]> for DynSoaSlicesMut<'a, Fields> {
    fn as_ref(&self) -> &[(Layout, DynFieldSliceMut<'a>)] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }
}

impl<'a, Fields> AsMut<[(Layout, DynFieldSliceMut<'a>)]> for DynSoaSlicesMut<'a, Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, DynFieldSliceMut<'a>)] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }
}

impl<'a, Fields> Debug for DynSoaSlicesMut<'a, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynSoaSlicesMut")
            .field("len", &self.len)
            .field("slices", &self.slices)
            .finish()
    }
}

impl<'a, Fields> PartialEq for DynSoaSlicesMut<'a, Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.slices == other.slices && self.phantom == other.phantom
    }
}

impl<'a, Fields> Eq for DynSoaSlicesMut<'a, Fields> {}

impl<'a, Fields> Hash for DynSoaSlicesMut<'a, Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.len.hash(state);
        self.slices.hash(state);
        self.phantom.hash(state);
    }
}

unsafe impl<'a, Fields> Send for DynSoaSlicesMut<'a, Fields> where Fields: Send {}
unsafe impl<'a, Fields> Sync for DynSoaSlicesMut<'a, Fields> where Fields: Sync {}

unsafe impl<Fields> Soa for DynSoa<Fields> {
    type Context = DynSoaContext<Fields>;

    type Fields = Fields;

    type FieldLayouts<'a> = &'a [Layout];

    fn field_layouts(context: &Self::Context) -> Self::FieldLayouts<'_> {
        let DynSoaContext { field_layouts, .. } = context;
        field_layouts.as_ref()
    }

    type Ptrs = DynSoaPtrs<Fields>;

    type MutPtrs = DynSoaMutPtrs<Fields>;

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
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (field_layout.clone(), ptr)
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
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (field_layout.clone(), ptr)
            })
            .collect();
        DynSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn ptrs_erase(
        context: &Self::Context,
        ptrs: Self::Ptrs,
    ) -> impl IntoIterator<Item = *const u8> {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        ptrs.into_vec().into_iter().map(|(_, ptr)| ptr.cast())
    }

    fn ptrs_erase_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
    ) -> impl IntoIterator<Item = *mut u8> {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        ptrs.into_vec().into_iter().map(|(_, ptr)| ptr.cast())
    }

    fn ptrs_restore(
        context: &Self::Context,
        ptrs: impl IntoIterator<Item = *const u8>,
    ) -> Self::Ptrs {
        let DynSoaContext { field_layouts, .. } = context;

        let ptrs: Box<[_]> = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let data = ptr.cast();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts(data, len);
                (field_layout.clone(), ptr)
            })
            .collect();
        assert_eq!(field_layouts.len(), ptrs.len());

        DynSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn ptrs_restore_mut(
        context: &Self::Context,
        ptrs: impl IntoIterator<Item = *mut u8>,
    ) -> Self::MutPtrs {
        let DynSoaContext { field_layouts, .. } = context;

        let ptrs: Box<[_]> = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let data = ptr.cast();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (field_layout.clone(), ptr)
            })
            .collect();
        assert_eq!(field_layouts.len(), ptrs.len());

        DynSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn ptrs_cast_const(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::Ptrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                (layout, ptr.cast_const())
            })
            .collect();
        DynSoaPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn ptrs_cast_mut(context: &Self::Context, ptrs: Self::Ptrs) -> Self::MutPtrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                (layout, ptr.cast_mut())
            })
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
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                // assert_eq!(field_layout.size(), ptr.len());

                let count = offset * field_layout.pad_to_align().size();
                let data = unsafe { ptr.cast::<u8>().add(count) };
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts(data, len);
                (layout, ptr)
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
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                // assert_eq!(field_layout.size(), ptr.len());

                let count = offset * field_layout.pad_to_align().size();
                let data = unsafe { ptr.cast::<u8>().add(count) };
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (layout, ptr)
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

        let mut offsets = field_layouts.iter().zip(ptrs).zip(origin).map(
            |((field_layout, (ptr_layout, ptr)), (origin_layout, origin))| {
                assert_eq!(field_layout, &ptr_layout);
                assert_eq!(field_layout, &origin_layout);
                assert_eq!(field_layout.size(), ptr.len());
                assert_eq!(ptr.len(), origin.len());

                let offset = unsafe { ptr.cast::<u8>().offset_from(origin.cast()) };
                let field_size = field_layout
                    .size()
                    .try_into()
                    .expect("layout size should not exceed `isize::MAX`");
                offset
                    .checked_div(field_size)
                    .expect("self should not be a ZST")
            },
        );

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

        let mut offsets = field_layouts.iter().zip(ptrs).zip(origin).map(
            |((field_layout, (ptr_layout, ptr)), (origin_layout, origin))| {
                assert_eq!(field_layout, &ptr_layout);
                assert_eq!(field_layout, &origin_layout);
                assert_eq!(field_layout.size(), ptr.len());
                assert_eq!(ptr.len(), origin.len());

                let offset = unsafe { ptr.cast::<u8>().offset_from(origin.cast()) };
                let field_size = field_layout
                    .size()
                    .try_into()
                    .expect("layout size should not exceed `isize::MAX`");
                offset
                    .checked_div(field_size)
                    .expect("self should not be a ZST")
            },
        );

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
        for ((field_layout, (a_layout, a)), (b_layout, b)) in field_layouts.iter().zip(a).zip(b) {
            assert_eq!(field_layout, &a_layout);
            assert_eq!(field_layout, &b_layout);
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
        for ((field_layout, (src_layout, src)), (dst_layout, dst)) in
            field_layouts.iter().zip(src).zip(dst)
        {
            assert_eq!(field_layout, &src_layout);
            assert_eq!(field_layout, &dst_layout);
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
        for ((field_layout, (src_layout, src)), (dst_layout, dst)) in
            field_layouts.iter().zip(src).zip(dst).rev()
        {
            assert_eq!(field_layout, &src_layout);
            assert_eq!(field_layout, &dst_layout);
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

        for ((field_layout, (src_layout, src)), (dst_layout, dst)) in
            field_layouts.iter().zip(src).zip(dst)
        {
            assert_eq!(field_layout, &src_layout);
            assert_eq!(field_layout, &dst_layout);

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

        let (buffer_layout, offsets) =
            Self::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout.size().div_ceil(size_of::<Byte<Fields>>());

        let mut buffer = Box::new_uninit_slice(buffer_len);
        for ((field_layout, (src_layout, src)), offset) in
            field_layouts.iter().zip(src).zip(offsets)
        {
            assert_eq!(field_layout, &src_layout);
            let src = src.cast();
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
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaMutPtrs { ptrs: dst, .. } = dst;
        let Self {
            buffer,
            field_layouts: value_layouts,
        } = value;

        assert_eq!(field_layouts.len(), dst.len());
        assert_eq!(field_layouts.as_ref(), value_layouts.as_ref());

        let (buffer_layout, offsets) =
            Self::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout.size().div_ceil(size_of::<Byte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        for ((field_layout, (dst_layout, dst)), offset) in
            field_layouts.iter().zip(dst).zip(offsets)
        {
            assert_eq!(field_layout, &dst_layout);

            let src = unsafe { buffer.as_ptr().cast::<u8>().add(offset) };
            let dst = dst.cast();

            let len = field_layout.size();
            unsafe {
                ptr::copy_nonoverlapping(src, dst, len);
            }
        }
    }

    type NonNullPtrs = DynSoaNonNullPtrs<Fields>;

    unsafe fn ptrs_to_nonnull(context: &Self::Context, ptrs: Self::MutPtrs) -> Self::NonNullPtrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                // assert_eq!(field_layout.size(), ptr.len());
                (layout, unsafe { NonNull::new_unchecked(ptr) })
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
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                // assert_eq!(field_layout.size(), ptr.len());
                (field_layout.clone(), ptr.as_ptr())
            })
            .collect();
        DynSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    type Vecs = DynSoaVecs<Fields>;

    fn vecs_with_capacity(context: &Self::Context, capacity: usize) -> Self::Vecs {
        let DynSoaContext { field_layouts, .. } = context;

        let vecs = field_layouts
            .iter()
            .map(|field_layout| {
                let capacity = (capacity * field_layout.size()).div_ceil(size_of::<Byte<Fields>>());
                DynFieldVec {
                    buffer: Vec::with_capacity(capacity),
                    layout: field_layout.clone(),
                }
            })
            .collect();
        DynSoaVecs { len: 0, vecs }
    }

    fn vecs_as_ptrs(context: &Self::Context, vecs: &Self::Vecs) -> Self::Ptrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaVecs { vecs, .. } = vecs;

        assert_eq!(field_layouts.len(), vecs.len());

        let ptrs = field_layouts
            .iter()
            .zip(vecs)
            .map(|(field_layout, vec)| {
                let DynFieldVec {
                    buffer,
                    layout: vec_field_layout,
                    ..
                } = vec;
                assert_eq!(field_layout, vec_field_layout);

                let data = buffer.as_ptr().cast();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts(data, len);
                (field_layout.clone(), ptr)
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
                let DynFieldVec {
                    buffer,
                    layout: vec_field_layout,
                    ..
                } = vec;
                assert_eq!(field_layout, vec_field_layout);

                let data = buffer.as_mut_ptr().cast();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (field_layout.clone(), ptr)
            })
            .collect();
        DynSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn vecs_len(context: &Self::Context, vecs: &Self::Vecs) -> usize {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaVecs { vecs, len, .. } = vecs;

        assert_eq!(field_layouts.len(), vecs.len());

        // let mut lens = field_layouts.iter().zip(vecs).map(|(field_layout, vec)| {
        //     let DynFieldVec {
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
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaVecs {
            vecs, len: vec_len, ..
        } = vecs;

        assert_eq!(field_layouts.len(), vecs.len());

        for (field_layout, vec) in field_layouts.iter().zip(vecs) {
            let DynFieldVec {
                buffer: field_buffer,
                layout: vec_field_layout,
            } = vec;
            assert_eq!(field_layout, vec_field_layout);

            *vec_len = len;
            let len = (len * vec_field_layout.size()).div_ceil(size_of::<Byte<Fields>>());
            unsafe {
                field_buffer.set_len(len);
            }
        }
    }

    type Refs<'a>
        = DynSoaRefs<'a, Fields>
    where
        Self: 'a;

    type RefsMut<'a>
        = DynSoaRefsMut<'a, Fields>
    where
        Self: 'a;

    unsafe fn ptrs_to_refs<'a>(context: &Self::Context, ptrs: Self::Ptrs) -> Self::Refs<'a> {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let refs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(field_layout.size(), ptr.len());

                let r#ref = unsafe { slice::from_raw_parts(ptr.cast(), ptr.len()) };
                (field_layout.clone(), r#ref)
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
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(field_layout.size(), ptr.len());

                let r#ref = unsafe { slice::from_raw_parts_mut(ptr.cast(), ptr.len()) };
                (layout, r#ref)
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
            .map(|(field_layout, (layout, r#ref))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(field_layout.size(), r#ref.len());

                (layout, ptr::from_ref(r#ref))
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
            .map(|(field_layout, (layout, r#ref))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(field_layout.size(), r#ref.len());
                (layout, ptr::from_mut(r#ref))
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
            .map(|(field_layout, (layout, r#ref))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(field_layout.size(), r#ref.len());
                (layout, &*r#ref)
            })
            .collect();
        DynSoaRefs {
            refs,
            phantom: PhantomData,
        }
    }

    type SlicePtrs = DynSoaSlicePtrs<Fields>;

    type SliceMutPtrs = DynSoaSliceMutPtrs<Fields>;

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
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                // assert_eq!(field_layout.size(), ptr.len());

                let data = ptr.cast();
                let len = len * field_layout.size();
                let slice = ptr::slice_from_raw_parts(data, len);
                (layout, slice)
            })
            .collect();
        DynSoaSlicePtrs {
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
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        let slices = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(field_layout.size(), ptr.len());

                let data = ptr.cast();
                let len = len * field_layout.size();
                let slice = ptr::slice_from_raw_parts_mut(data, len);
                (layout, slice)
            })
            .collect();
        DynSoaSliceMutPtrs {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn slice_ptrs_cast_const(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicePtrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSliceMutPtrs { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                (layout, slice.cast_const())
            })
            .collect();
        DynSoaSlicePtrs {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn slice_ptrs_cast_mut(context: &Self::Context, slices: Self::SlicePtrs) -> Self::SliceMutPtrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSlicePtrs { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                (layout, slice.cast_mut())
            })
            .collect();
        DynSoaSliceMutPtrs {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn slice_ptrs_len(context: &Self::Context, slices: Self::SlicePtrs) -> usize {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSlicePtrs { slices, len, .. } = slices;

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
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSliceMutPtrs { slices, len, .. } = slices;

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
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSlicePtrs { slices, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.cast();
                let len = slice.len().checked_div(field_layout.size()).unwrap_or(0);
                let ptr = ptr::slice_from_raw_parts(data, len);
                (layout, ptr)
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
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.cast();
                let len = slice.len().checked_div(field_layout.size()).unwrap_or(0);
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (layout, ptr)
            })
            .collect();
        DynSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    type Slices<'a>
        = DynSoaSlices<'a, Fields>
    where
        Self: 'a;

    type SlicesMut<'a>
        = DynSoaSlicesMut<'a, Fields>
    where
        Self: 'a;

    unsafe fn slice_ptrs_to_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicePtrs,
    ) -> Self::Slices<'a> {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSlicePtrs { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.cast();
                let len = slice.len();
                let slice = unsafe { slice::from_raw_parts(data, len) };
                (layout, slice)
            })
            .collect();
        DynSoaSlices {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    unsafe fn slice_ptrs_to_slices_mut<'a>(
        context: &Self::Context,
        slices: Self::SliceMutPtrs,
    ) -> Self::SlicesMut<'a> {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSliceMutPtrs { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.cast();
                let len = slice.len();
                let slice = unsafe { slice::from_raw_parts_mut(data, len) };
                (layout, slice)
            })
            .collect();
        DynSoaSlicesMut {
            len,
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
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, layout);
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
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, layout);
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
        let DynSoaSlices { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                (layout, ptr::from_ref(slice))
            })
            .collect();
        DynSoaSlicePtrs {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn mut_slice_refs_as_slice_ptrs(
        context: &Self::Context,
        slices: Self::SlicesMut<'_>,
    ) -> Self::SliceMutPtrs {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSlicesMut { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                (layout, ptr::from_mut(slice))
            })
            .collect();
        DynSoaSliceMutPtrs {
            len,
            slices,
            phantom: PhantomData,
        }
    }

    fn mut_slices_as_slices<'a>(
        context: &Self::Context,
        slices: Self::SlicesMut<'a>,
    ) -> Self::Slices<'a> {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaSlicesMut { slices, len, .. } = slices;

        assert_eq!(field_layouts.len(), slices.len());

        let slices = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
                (layout, &*slice)
            })
            .collect();
        DynSoaSlices {
            len,
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
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.as_ptr();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts(data, len);
                (layout, ptr)
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
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.as_mut_ptr();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (layout, ptr)
            })
            .collect();
        DynSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }
}
