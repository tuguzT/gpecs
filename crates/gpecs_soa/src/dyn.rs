use alloc::{boxed::Box, vec::Vec};
use core::{
    alloc::Layout,
    borrow::Borrow,
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
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
}

impl<Fields> DynSoa<Fields> {
    #[inline]
    pub fn new<'a, I>(context: &DynSoaContext<Fields>, fields: I) -> Self
    where
        I: IntoIterator<Item = &'a [u8]>,
    {
        let DynSoaContext { field_layouts, .. } = context;

        let (buffer_layout, offsets) =
            Self::buffer_layout(&context, 1).expect("layout size should not exceed `isize::MAX`");
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
        Self { buffer }
    }

    #[inline]
    pub fn as_refs(&self, context: &DynSoaContext<Fields>) -> DynSoaRefs<'_, Fields> {
        let Self { buffer } = self;
        let DynSoaContext { field_layouts, .. } = context;

        let (buffer_layout, offsets) =
            Self::buffer_layout(&context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout.size().div_ceil(size_of::<Byte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        let refs = field_layouts
            .iter()
            .zip(offsets)
            .map(|(field_layout, offset)| unsafe {
                let data = buffer.as_ptr().cast::<u8>().add(offset);
                let len = field_layout.size();
                slice::from_raw_parts(data, len)
            })
            .collect();
        DynSoaRefs {
            refs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn as_refs_mut(&mut self, context: &DynSoaContext<Fields>) -> DynSoaRefsMut<'_, Fields> {
        let Self { buffer } = self;
        let DynSoaContext { field_layouts, .. } = context;

        let (buffer_layout, offsets) =
            Self::buffer_layout(&context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout.size().div_ceil(size_of::<Byte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        let refs = field_layouts
            .iter()
            .zip(offsets)
            .map(|(field_layout, offset)| unsafe {
                let data = buffer.as_mut_ptr().cast::<u8>().add(offset);
                let len = field_layout.size();
                slice::from_raw_parts_mut(data, len)
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
        let field_layouts = field_layouts
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
            .collect();
        Self {
            field_layouts,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn of<T>(context: T::Context) -> Self
    where
        T: Soa<Fields = Fields>,
    {
        let mut field_layouts: Box<[_]> = T::field_layouts(&context)
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
            .collect();

        let (_, offsets) =
            T::buffer_layout(&context, 1).expect("layout size should not exceed `isize::MAX`");
        let offsets: Box<[_]> = offsets.into_iter().collect();

        let mut permutation: Box<_> = (0..offsets.len()).collect();
        permutation.sort_by_key(|&index| offsets[index]);

        for src in 0..permutation.len() {
            let dst = permutation[src];
            if src == dst {
                continue;
            }
            field_layouts.swap(src, dst);
            permutation.swap(src, dst);
        }

        Self {
            field_layouts,
            phantom: PhantomData,
        }
    }
}

impl<Fields> AsRef<[Layout]> for DynSoaContext<Fields> {
    fn as_ref(&self) -> &[Layout] {
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

impl<Fields> Hash for DynSoaContext<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.field_layouts.hash(state);
        self.phantom.hash(state);
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

type DynFieldPtr = *const [u8];

pub struct DynSoaPtrs<Fields> {
    ptrs: Box<[DynFieldPtr]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> DynSoaPtrs<Fields> {
    pub fn new<I>(ptrs: I) -> Self
    where
        I: IntoIterator<Item = DynFieldPtr>,
    {
        Self {
            ptrs: ptrs.into_iter().collect(),
            phantom: PhantomData,
        }
    }
}

impl<Fields> AsRef<[DynFieldPtr]> for DynSoaPtrs<Fields> {
    fn as_ref(&self) -> &[DynFieldPtr] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }
}

impl<Fields> AsMut<[DynFieldPtr]> for DynSoaPtrs<Fields> {
    fn as_mut(&mut self) -> &mut [DynFieldPtr] {
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

impl<Fields> PartialOrd for DynSoaPtrs<Fields> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.ptrs.partial_cmp(&other.ptrs) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.phantom.partial_cmp(&other.phantom)
    }
}

impl<Fields> Ord for DynSoaPtrs<Fields> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.ptrs.cmp(&other.ptrs) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.phantom.cmp(&other.phantom)
    }
}

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
    ptrs: Box<[DynFieldMutPtr]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> DynSoaMutPtrs<Fields> {
    pub fn new<I>(ptrs: I) -> Self
    where
        I: IntoIterator<Item = DynFieldMutPtr>,
    {
        Self {
            ptrs: ptrs.into_iter().collect(),
            phantom: PhantomData,
        }
    }
}

impl<Fields> AsRef<[DynFieldMutPtr]> for DynSoaMutPtrs<Fields> {
    fn as_ref(&self) -> &[DynFieldMutPtr] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }
}

impl<Fields> AsMut<[DynFieldMutPtr]> for DynSoaMutPtrs<Fields> {
    fn as_mut(&mut self) -> &mut [DynFieldMutPtr] {
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

impl<Fields> PartialOrd for DynSoaMutPtrs<Fields> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.ptrs.partial_cmp(&other.ptrs) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.phantom.partial_cmp(&other.phantom)
    }
}

impl<Fields> Ord for DynSoaMutPtrs<Fields> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.ptrs.cmp(&other.ptrs) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.phantom.cmp(&other.phantom)
    }
}

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
    ptrs: Box<[DynFieldNonNullPtr]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> DynSoaNonNullPtrs<Fields> {
    pub fn new<I>(ptrs: I) -> Self
    where
        I: IntoIterator<Item = DynFieldNonNullPtr>,
    {
        Self {
            ptrs: ptrs.into_iter().collect(),
            phantom: PhantomData,
        }
    }
}

impl<Fields> AsRef<[DynFieldNonNullPtr]> for DynSoaNonNullPtrs<Fields> {
    fn as_ref(&self) -> &[DynFieldNonNullPtr] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }
}

impl<Fields> AsMut<[DynFieldNonNullPtr]> for DynSoaNonNullPtrs<Fields> {
    fn as_mut(&mut self) -> &mut [DynFieldNonNullPtr] {
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

impl<Fields> PartialOrd for DynSoaNonNullPtrs<Fields> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.ptrs.partial_cmp(&other.ptrs) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.phantom.partial_cmp(&other.phantom)
    }
}

impl<Fields> Ord for DynSoaNonNullPtrs<Fields> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.ptrs.cmp(&other.ptrs) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.phantom.cmp(&other.phantom)
    }
}

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
    len: usize,
}

pub struct DynSoaVecs<Fields> {
    vecs: Box<[DynFieldVec<Fields>]>,
}

type DynFieldRef<'a> = &'a [u8];

pub struct DynSoaRefs<'a, Fields>
where
    Fields: 'a,
{
    refs: Box<[DynFieldRef<'a>]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> DynSoaRefs<'a, Fields> {
    pub fn new<I>(refs: I) -> Self
    where
        I: IntoIterator<Item = DynFieldRef<'a>>,
    {
        Self {
            refs: refs.into_iter().collect(),
            phantom: PhantomData,
        }
    }
}

impl<'a, Fields> AsRef<[DynFieldRef<'a>]> for DynSoaRefs<'a, Fields> {
    fn as_ref(&self) -> &[DynFieldRef<'a>] {
        let Self { refs, .. } = self;
        refs.as_ref()
    }
}

impl<'a, Fields> AsMut<[DynFieldRef<'a>]> for DynSoaRefs<'a, Fields> {
    fn as_mut(&mut self) -> &mut [DynFieldRef<'a>] {
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

impl<'a, Fields> PartialOrd for DynSoaRefs<'a, Fields> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.refs.partial_cmp(&other.refs) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.phantom.partial_cmp(&other.phantom)
    }
}

impl<'a, Fields> Ord for DynSoaRefs<'a, Fields> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.refs.cmp(&other.refs) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.phantom.cmp(&other.phantom)
    }
}

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
    refs: Box<[DynFieldRefMut<'a>]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> DynSoaRefsMut<'a, Fields> {
    pub fn new<I>(refs: I) -> Self
    where
        I: IntoIterator<Item = DynFieldRefMut<'a>>,
    {
        Self {
            refs: refs.into_iter().collect(),
            phantom: PhantomData,
        }
    }
}

impl<'a, Fields> AsRef<[DynFieldRefMut<'a>]> for DynSoaRefsMut<'a, Fields> {
    fn as_ref(&self) -> &[DynFieldRefMut<'a>] {
        let Self { refs, .. } = self;
        refs.as_ref()
    }
}

impl<'a, Fields> AsMut<[DynFieldRefMut<'a>]> for DynSoaRefsMut<'a, Fields> {
    fn as_mut(&mut self) -> &mut [DynFieldRefMut<'a>] {
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

impl<'a, Fields> PartialOrd for DynSoaRefsMut<'a, Fields> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.refs.partial_cmp(&other.refs) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.phantom.partial_cmp(&other.phantom)
    }
}

impl<'a, Fields> Ord for DynSoaRefsMut<'a, Fields> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.refs.cmp(&other.refs) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.phantom.cmp(&other.phantom)
    }
}

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
    slices: Box<[DynFieldSlicePtr]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> DynSoaSlicePtrs<Fields> {
    pub fn new<I>(slices: I) -> Self
    where
        I: IntoIterator<Item = DynFieldSlicePtr>,
    {
        Self {
            slices: slices.into_iter().collect(),
            phantom: PhantomData,
        }
    }
}

impl<Fields> AsRef<[DynFieldSlicePtr]> for DynSoaSlicePtrs<Fields> {
    fn as_ref(&self) -> &[DynFieldSlicePtr] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }
}

impl<Fields> AsMut<[DynFieldSlicePtr]> for DynSoaSlicePtrs<Fields> {
    fn as_mut(&mut self) -> &mut [DynFieldSlicePtr] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }
}

impl<Fields> Debug for DynSoaSlicePtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DynSoaSlicePtrs")
            .field(&self.slices)
            .finish()
    }
}

impl<Fields> PartialEq for DynSoaSlicePtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.slices == other.slices && self.phantom == other.phantom
    }
}

impl<Fields> Eq for DynSoaSlicePtrs<Fields> {}

impl<Fields> PartialOrd for DynSoaSlicePtrs<Fields> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.slices.partial_cmp(&other.slices) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.phantom.partial_cmp(&other.phantom)
    }
}

impl<Fields> Ord for DynSoaSlicePtrs<Fields> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.slices.cmp(&other.slices) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.phantom.cmp(&other.phantom)
    }
}

impl<Fields> Hash for DynSoaSlicePtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.slices.hash(state);
        self.phantom.hash(state);
    }
}

impl<Fields> Clone for DynSoaSlicePtrs<Fields> {
    fn clone(&self) -> Self {
        Self {
            slices: self.slices.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

// data is stored inline in a single buffer
type DynFieldSliceMutPtr = *mut [u8];

pub struct DynSoaSliceMutPtrs<Fields> {
    slices: Box<[DynFieldSliceMutPtr]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> DynSoaSliceMutPtrs<Fields> {
    pub fn new<I>(slices: I) -> Self
    where
        I: IntoIterator<Item = DynFieldSliceMutPtr>,
    {
        Self {
            slices: slices.into_iter().collect(),
            phantom: PhantomData,
        }
    }
}

impl<Fields> AsRef<[DynFieldSliceMutPtr]> for DynSoaSliceMutPtrs<Fields> {
    fn as_ref(&self) -> &[DynFieldSliceMutPtr] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }
}

impl<Fields> AsMut<[DynFieldSliceMutPtr]> for DynSoaSliceMutPtrs<Fields> {
    fn as_mut(&mut self) -> &mut [DynFieldSliceMutPtr] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }
}

impl<Fields> Debug for DynSoaSliceMutPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DynSoaSliceMutPtrs")
            .field(&self.slices)
            .finish()
    }
}

impl<Fields> PartialEq for DynSoaSliceMutPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.slices == other.slices && self.phantom == other.phantom
    }
}

impl<Fields> Eq for DynSoaSliceMutPtrs<Fields> {}

impl<Fields> PartialOrd for DynSoaSliceMutPtrs<Fields> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.slices.partial_cmp(&other.slices) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.phantom.partial_cmp(&other.phantom)
    }
}

impl<Fields> Ord for DynSoaSliceMutPtrs<Fields> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.slices.cmp(&other.slices) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.phantom.cmp(&other.phantom)
    }
}

impl<Fields> Hash for DynSoaSliceMutPtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.slices.hash(state);
        self.phantom.hash(state);
    }
}

impl<Fields> Clone for DynSoaSliceMutPtrs<Fields> {
    fn clone(&self) -> Self {
        Self {
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
    slices: Box<[DynFieldSlice<'a>]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> DynSoaSlices<'a, Fields> {
    pub fn new<I>(slices: I) -> Self
    where
        I: IntoIterator<Item = DynFieldSlice<'a>>,
    {
        Self {
            slices: slices.into_iter().collect(),
            phantom: PhantomData,
        }
    }
}

impl<'a, Fields> AsRef<[DynFieldSlice<'a>]> for DynSoaSlices<'a, Fields> {
    fn as_ref(&self) -> &[DynFieldSlice<'a>] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }
}

impl<'a, Fields> AsMut<[DynFieldSlice<'a>]> for DynSoaSlices<'a, Fields> {
    fn as_mut(&mut self) -> &mut [DynFieldSlice<'a>] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }
}

impl<'a, Fields> Debug for DynSoaSlices<'a, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DynSoaSlices").field(&self.slices).finish()
    }
}

impl<'a, Fields> PartialEq for DynSoaSlices<'a, Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.slices == other.slices && self.phantom == other.phantom
    }
}

impl<'a, Fields> Eq for DynSoaSlices<'a, Fields> {}

impl<'a, Fields> PartialOrd for DynSoaSlices<'a, Fields> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.slices.partial_cmp(&other.slices) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.phantom.partial_cmp(&other.phantom)
    }
}

impl<'a, Fields> Ord for DynSoaSlices<'a, Fields> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.slices.cmp(&other.slices) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.phantom.cmp(&other.phantom)
    }
}

impl<'a, Fields> Hash for DynSoaSlices<'a, Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.slices.hash(state);
        self.phantom.hash(state);
    }
}

impl<'a, Fields> Clone for DynSoaSlices<'a, Fields> {
    fn clone(&self) -> Self {
        Self {
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
    slices: Box<[DynFieldSliceMut<'a>]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> DynSoaSlicesMut<'a, Fields> {
    pub fn new<I>(slices: I) -> Self
    where
        I: IntoIterator<Item = DynFieldSliceMut<'a>>,
    {
        Self {
            slices: slices.into_iter().collect(),
            phantom: PhantomData,
        }
    }
}

impl<'a, Fields> AsRef<[DynFieldSliceMut<'a>]> for DynSoaSlicesMut<'a, Fields> {
    fn as_ref(&self) -> &[DynFieldSliceMut<'a>] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }
}

impl<'a, Fields> AsMut<[DynFieldSliceMut<'a>]> for DynSoaSlicesMut<'a, Fields> {
    fn as_mut(&mut self) -> &mut [DynFieldSliceMut<'a>] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }
}

impl<'a, Fields> Debug for DynSoaSlicesMut<'a, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DynSoaSlicesMut")
            .field(&self.slices)
            .finish()
    }
}

impl<'a, Fields> PartialEq for DynSoaSlicesMut<'a, Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.slices == other.slices && self.phantom == other.phantom
    }
}

impl<'a, Fields> Eq for DynSoaSlicesMut<'a, Fields> {}

impl<'a, Fields> PartialOrd for DynSoaSlicesMut<'a, Fields> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.slices.partial_cmp(&other.slices) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.phantom.partial_cmp(&other.phantom)
    }
}

impl<'a, Fields> Ord for DynSoaSlicesMut<'a, Fields> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.slices.cmp(&other.slices) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.phantom.cmp(&other.phantom)
    }
}

impl<'a, Fields> Hash for DynSoaSlicesMut<'a, Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
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
                // assert_eq!(field_layout.size(), ptr.len());

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
                // assert_eq!(field_layout.size(), ptr.len());

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
        let buffer = unsafe {
            for ((field_layout, src), offset) in field_layouts.iter().zip(src).zip(offsets) {
                let src = src.cast();
                let dst = buffer.as_mut_ptr().cast::<u8>().add(offset);

                let len = field_layout.size();
                ptr::copy_nonoverlapping(src, dst, len);
            }
            buffer.assume_init()
        };
        Self { buffer }
    }

    unsafe fn ptrs_write(context: &Self::Context, dst: Self::MutPtrs, value: Self) {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaMutPtrs { ptrs: dst, .. } = dst;
        let Self { buffer } = value;

        assert_eq!(field_layouts.len(), dst.len());

        let (buffer_layout, offsets) =
            Self::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout.size().div_ceil(size_of::<Byte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        for ((field_layout, dst), offset) in field_layouts.iter().zip(dst).zip(offsets) {
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
            .map(|(_field_layout, ptr)| {
                // assert_eq!(field_layout.size(), ptr.len());
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
            .map(|(_field_layout, ptr)| {
                // assert_eq!(field_layout.size(), ptr.len());
                ptr.as_ptr()
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

        // let vecs = iter::repeat_n(Vec::with_capacity(capacity), field_layouts.len()).collect();
        let vecs = field_layouts
            .iter()
            .map(|field_layout| {
                let capacity = (capacity * field_layout.size()).div_ceil(size_of::<Byte<Fields>>());
                DynFieldVec {
                    buffer: Vec::with_capacity(capacity),
                    layout: field_layout.clone(),
                    len: 0,
                }
            })
            .collect();
        DynSoaVecs { vecs }
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
                ptr::slice_from_raw_parts(data, field_layout.size())
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
                ptr::slice_from_raw_parts_mut(data, field_layout.size())
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

        let mut lens = field_layouts.iter().zip(vecs).map(|(field_layout, vec)| {
            let DynFieldVec {
                layout: vec_field_layout,
                len,
                ..
            } = vec;
            assert_eq!(field_layout, vec_field_layout);
            *len
        });
        let len = lens.next().unwrap_or(0);
        assert!(lens.all(|item| item == len));
        len
    }

    unsafe fn vecs_set_len(context: &Self::Context, vecs: &mut Self::Vecs, len: usize) {
        let DynSoaContext { field_layouts, .. } = context;
        let DynSoaVecs { vecs, .. } = vecs;

        assert_eq!(field_layouts.len(), vecs.len());

        for (field_layout, vec) in field_layouts.iter().zip(vecs) {
            let DynFieldVec {
                buffer: field_buffer,
                layout: vec_field_layout,
                len: vec_len,
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
            .map(|(field_layout, ptr)| {
                // assert_eq!(field_layout.size(), ptr.len());

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
}
