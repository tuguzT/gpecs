use alloc::{boxed::Box, vec::Vec};
use core::{
    alloc::Layout,
    borrow::Borrow,
    fmt::{self, Debug},
    hash::{self, Hash},
    iter,
    marker::PhantomData,
    mem::{self, ManuallyDrop, MaybeUninit},
    ptr::{self, NonNull},
    slice,
};

use crate::traits::Soa;

union Byte<Fields> {
    _byte: u8,
    _size_align: ManuallyDrop<MaybeUninit<Fields>>,
}

unsafe impl<Fields> Send for Byte<Fields> where Fields: Send {}
unsafe impl<Fields> Sync for Byte<Fields> where Fields: Sync {}

type ErasedFields<Fields> = Box<[Byte<Fields>]>;

pub struct ErasedSoa<Fields> {
    buffer: ErasedFields<Fields>,
    field_layouts: Box<[Layout]>,
}

impl<Fields> ErasedSoa<Fields> {
    #[inline]
    pub fn new<'a, I>(context: &ErasedSoaContext<Fields>, fields: I) -> Self
    where
        I: IntoIterator<Item = &'a [u8]>,
    {
        let ErasedSoaContext { field_layouts, .. } = context;
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
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<T::Fields, _>)
            .collect();

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

        let target_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<T::Fields, _>);
        assert!(target_layouts.eq(field_layouts));

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
    pub fn into_fields(self, context: &ErasedSoaContext<Fields>) -> Box<[(Layout, Box<[u8]>)]> {
        let Self {
            buffer,
            field_layouts: value_layouts,
        } = self;
        let ErasedSoaContext { field_layouts, .. } = context;

        assert_eq!(field_layouts.as_ref(), value_layouts.as_ref());

        let (buffer_layout, offsets) =
            Self::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout.size().div_ceil(size_of::<Byte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        iter::zip(value_layouts, offsets)
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
    pub fn as_refs(&self, context: &ErasedSoaContext<Fields>) -> ErasedSoaRefs<'_, Fields> {
        let Self {
            buffer,
            field_layouts: value_layouts,
        } = self;
        let ErasedSoaContext { field_layouts, .. } = context;

        assert_eq!(field_layouts.as_ref(), value_layouts.as_ref());

        let (buffer_layout, offsets) =
            Self::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout.size().div_ceil(size_of::<Byte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        let refs = field_layouts
            .iter()
            .zip(offsets)
            .map(|(field_layout, offset)| {
                let data = unsafe { buffer.as_ptr().cast::<u8>().add(offset) };
                let len = field_layout.size();
                let r#ref = unsafe { slice::from_raw_parts(data, len) };
                (field_layout.clone(), r#ref)
            })
            .collect();
        ErasedSoaRefs {
            refs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn as_refs_mut(
        &mut self,
        context: &ErasedSoaContext<Fields>,
    ) -> ErasedSoaRefsMut<'_, Fields> {
        let Self {
            buffer,
            field_layouts: value_layouts,
        } = self;
        let ErasedSoaContext { field_layouts, .. } = context;

        assert_eq!(field_layouts.as_ref(), value_layouts.as_ref());

        let (buffer_layout, offsets) =
            Self::buffer_layout(context, 1).expect("layout size should not exceed `isize::MAX`");
        let buffer_len = buffer_layout.size().div_ceil(size_of::<Byte<Fields>>());
        assert_eq!(buffer_len, buffer.len());

        let refs = field_layouts
            .iter()
            .zip(offsets)
            .map(|(field_layout, offset)| {
                let data = unsafe { buffer.as_mut_ptr().cast::<u8>().add(offset) };
                let len = field_layout.size();
                let r#ref = unsafe { slice::from_raw_parts_mut(data, len) };
                (field_layout.clone(), r#ref)
            })
            .collect();
        ErasedSoaRefsMut {
            refs,
            phantom: PhantomData,
        }
    }
}

type ErasedDropFnParam<'a> = &'a [(Layout, *mut [u8])];
type ErasedDropFn = Box<dyn Fn(ErasedDropFnParam<'_>)>;

pub struct ErasedSoaContext<Fields> {
    field_layouts: Box<[Layout]>,
    drop_fields: Option<ErasedDropFn>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaContext<Fields> {
    #[inline]
    pub fn new<I, O>(field_layouts: I, drop_fields: O) -> Self
    where
        I: IntoIterator<Item: Borrow<Layout>>,
        O: Into<Option<ErasedDropFn>>,
    {
        Self {
            field_layouts: field_layouts
                .into_iter()
                .map(validate_layout::<Fields, _>)
                .collect(),
            drop_fields: drop_fields.into(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn of<T>(context: T::Context) -> Self
    where
        T: Soa<Fields = Fields>,
        T::Context: 'static,
    {
        let field_layouts = T::field_layouts(&context)
            .into_iter()
            .map(validate_layout::<T::Fields, _>)
            .collect();

        let drop_fields = move |data: ErasedDropFnParam<'_>| unsafe {
            let ptrs = data.iter().map(|(_, ptr)| ptr.cast());
            let ptrs = T::ptrs_restore_mut(&context, ptrs);
            T::ptrs_drop_in_place(&context, ptrs);
        };
        let drop_fields: Option<ErasedDropFn> = if mem::needs_drop::<T::Fields>() {
            Some(Box::new(drop_fields))
        } else {
            None
        };

        Self {
            field_layouts,
            drop_fields,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn layouts(&self) -> &[Layout] {
        let Self { field_layouts, .. } = self;
        field_layouts.as_ref()
    }
}

impl<Fields> Debug for ErasedSoaContext<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ErasedSoaContext")
            .field(&self.field_layouts)
            .finish()
    }
}

type ErasedFieldPtr = *const [u8];

pub struct ErasedSoaPtrs<Fields> {
    ptrs: Box<[(Layout, ErasedFieldPtr)]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaPtrs<Fields> {
    #[inline]
    pub fn new<I>(context: &ErasedSoaContext<Fields>, ptrs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldPtr>,
    {
        let ErasedSoaContext { field_layouts, .. } = context;
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
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let ptrs = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts(ptr.cast(), len);
                (field_layout, ptr)
            })
            .collect();
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

        let field_layouts: Box<[_]> = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>)
            .collect();
        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                ptr.cast()
            });
        T::ptrs_restore(context, ptrs)
    }
}

impl<Fields> AsRef<[(Layout, ErasedFieldPtr)]> for ErasedSoaPtrs<Fields> {
    fn as_ref(&self) -> &[(Layout, ErasedFieldPtr)] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }
}

impl<Fields> AsMut<[(Layout, ErasedFieldPtr)]> for ErasedSoaPtrs<Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, ErasedFieldPtr)] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut()
    }
}

impl<Fields> Debug for ErasedSoaPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ErasedSoaPtrs").field(&self.ptrs).finish()
    }
}

impl<Fields> PartialEq for ErasedSoaPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.ptrs == other.ptrs && self.phantom == other.phantom
    }
}

impl<Fields> Eq for ErasedSoaPtrs<Fields> {}

impl<Fields> Hash for ErasedSoaPtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.ptrs.hash(state);
        self.phantom.hash(state);
    }
}

impl<Fields> Clone for ErasedSoaPtrs<Fields> {
    fn clone(&self) -> Self {
        Self {
            ptrs: self.ptrs.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

type ErasedFieldMutPtr = *mut [u8];

pub struct ErasedSoaMutPtrs<Fields> {
    ptrs: Box<[(Layout, ErasedFieldMutPtr)]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaMutPtrs<Fields> {
    pub fn new<I>(context: &ErasedSoaContext<Fields>, ptrs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldMutPtr>,
    {
        let ErasedSoaContext { field_layouts, .. } = context;
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
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let ptrs: Box<[_]> = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let data = ptr.cast();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (field_layout.clone(), ptr)
            })
            .collect();
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

        let field_layouts: Box<[_]> = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>)
            .collect();
        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                ptr.cast()
            });
        T::ptrs_restore_mut(context, ptrs)
    }
}

impl<Fields> AsRef<[(Layout, ErasedFieldMutPtr)]> for ErasedSoaMutPtrs<Fields> {
    fn as_ref(&self) -> &[(Layout, ErasedFieldMutPtr)] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }
}

impl<Fields> AsMut<[(Layout, ErasedFieldMutPtr)]> for ErasedSoaMutPtrs<Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, ErasedFieldMutPtr)] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut()
    }
}

impl<Fields> Debug for ErasedSoaMutPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ErasedSoaMutPtrs").field(&self.ptrs).finish()
    }
}

impl<Fields> PartialEq for ErasedSoaMutPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.ptrs == other.ptrs && self.phantom == other.phantom
    }
}

impl<Fields> Eq for ErasedSoaMutPtrs<Fields> {}

impl<Fields> Hash for ErasedSoaMutPtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.ptrs.hash(state);
        self.phantom.hash(state);
    }
}

impl<Fields> Clone for ErasedSoaMutPtrs<Fields> {
    fn clone(&self) -> Self {
        Self {
            ptrs: self.ptrs.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

type ErasedFieldNonNullPtr = NonNull<[u8]>;

pub struct ErasedSoaNonNullPtrs<Fields> {
    ptrs: Box<[(Layout, ErasedFieldNonNullPtr)]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaNonNullPtrs<Fields> {
    pub fn new<I>(context: &ErasedSoaContext<Fields>, ptrs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldNonNullPtr>,
    {
        let ErasedSoaContext { field_layouts, .. } = context;
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
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let ptrs = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(ptr.cast(), len);
                (field_layout.clone(), unsafe { NonNull::new_unchecked(ptr) })
            })
            .collect();
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

        let field_layouts: Box<[_]> = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>)
            .collect();
        assert_eq!(field_layouts.len(), ptrs.len());

        let ptrs = field_layouts
            .iter()
            .zip(ptrs)
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                ptr.as_ptr().cast()
            });
        let ptrs = T::ptrs_restore_mut(context, ptrs);
        unsafe { T::ptrs_to_nonnull(context, ptrs) }
    }
}

impl<Fields> AsRef<[(Layout, ErasedFieldNonNullPtr)]> for ErasedSoaNonNullPtrs<Fields> {
    fn as_ref(&self) -> &[(Layout, ErasedFieldNonNullPtr)] {
        let Self { ptrs, .. } = self;
        ptrs.as_ref()
    }
}

impl<Fields> AsMut<[(Layout, ErasedFieldNonNullPtr)]> for ErasedSoaNonNullPtrs<Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, ErasedFieldNonNullPtr)] {
        let Self { ptrs, .. } = self;
        ptrs.as_mut()
    }
}

impl<Fields> Debug for ErasedSoaNonNullPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ErasedSoaNonNullPtrs")
            .field(&self.ptrs)
            .finish()
    }
}

impl<Fields> PartialEq for ErasedSoaNonNullPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.ptrs == other.ptrs && self.phantom == other.phantom
    }
}

impl<Fields> Eq for ErasedSoaNonNullPtrs<Fields> {}

impl<Fields> Hash for ErasedSoaNonNullPtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.ptrs.hash(state);
        self.phantom.hash(state);
    }
}

impl<Fields> Clone for ErasedSoaNonNullPtrs<Fields> {
    fn clone(&self) -> Self {
        Self {
            ptrs: self.ptrs.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

// data is stored inline in a single buffer
struct ErasedFieldVec<Fields> {
    buffer: Vec<Byte<Fields>>,
    layout: Layout,
}

pub struct ErasedSoaVecs<Fields> {
    len: usize,
    vecs: Box<[ErasedFieldVec<Fields>]>,
}

type ErasedFieldRef<'a> = &'a [u8];

pub struct ErasedSoaRefs<'a, Fields>
where
    Fields: 'a,
{
    refs: Box<[(Layout, ErasedFieldRef<'a>)]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaRefs<'a, Fields> {
    pub fn new<I>(context: &ErasedSoaContext<Fields>, refs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldRef<'a>>,
    {
        let ErasedSoaContext { field_layouts, .. } = context;
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
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let refs: Box<[_]> = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| unsafe {
                let len = field_layout.size();
                (field_layout.clone(), slice::from_raw_parts(ptr.cast(), len))
            })
            .collect();
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

        let field_layouts: Box<[_]> = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>)
            .collect();
        assert_eq!(field_layouts.len(), refs.len());

        let ptrs = field_layouts
            .iter()
            .zip(refs)
            .map(|(field_layout, (layout, r#ref))| {
                assert_eq!(field_layout, &layout);
                r#ref.as_ptr()
            });
        let ptrs = T::ptrs_restore(context, ptrs);
        unsafe { T::ptrs_to_refs(context, ptrs) }
    }
}

impl<'a, Fields> AsRef<[(Layout, ErasedFieldRef<'a>)]> for ErasedSoaRefs<'a, Fields> {
    fn as_ref(&self) -> &[(Layout, ErasedFieldRef<'a>)] {
        let Self { refs, .. } = self;
        refs.as_ref()
    }
}

impl<'a, Fields> AsMut<[(Layout, ErasedFieldRef<'a>)]> for ErasedSoaRefs<'a, Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, ErasedFieldRef<'a>)] {
        let Self { refs, .. } = self;
        refs.as_mut()
    }
}

impl<'a, Fields> Debug for ErasedSoaRefs<'a, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ErasedSoaRefs").field(&self.refs).finish()
    }
}

impl<'a, Fields> Clone for ErasedSoaRefs<'a, Fields> {
    fn clone(&self) -> Self {
        Self {
            refs: self.refs.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<'r, 'a, Fields> IntoIterator for &'r ErasedSoaRefs<'a, Fields> {
    type Item = &'r (Layout, ErasedFieldRef<'a>);

    type IntoIter = core::slice::Iter<'r, (Layout, ErasedFieldRef<'a>)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaRefs { refs, .. } = self;
        refs.iter()
    }
}

impl<'r, 'a, Fields> IntoIterator for &'r mut ErasedSoaRefs<'a, Fields> {
    type Item = &'r mut (Layout, ErasedFieldRef<'a>);

    type IntoIter = core::slice::IterMut<'r, (Layout, ErasedFieldRef<'a>)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaRefs { refs, .. } = self;
        refs.iter_mut()
    }
}

impl<'a, Fields> IntoIterator for ErasedSoaRefs<'a, Fields> {
    type Item = (Layout, ErasedFieldRef<'a>);

    type IntoIter = alloc::vec::IntoIter<(Layout, ErasedFieldRef<'a>)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaRefs { refs, .. } = self;
        refs.into_vec().into_iter()
    }
}

unsafe impl<'a, Fields> Send for ErasedSoaRefs<'a, Fields> where Fields: Sync {}
unsafe impl<'a, Fields> Sync for ErasedSoaRefs<'a, Fields> where Fields: Sync {}

type ErasedFieldRefMut<'a> = &'a mut [u8];

pub struct ErasedSoaRefsMut<'a, Fields>
where
    Fields: 'a,
{
    refs: Box<[(Layout, ErasedFieldRefMut<'a>)]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaRefsMut<'a, Fields> {
    pub fn new<I>(context: &ErasedSoaContext<Fields>, refs: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldRefMut<'a>>,
    {
        let ErasedSoaContext { field_layouts, .. } = context;
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
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let refs = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let data = ptr.cast();
                let len = field_layout.size();
                let r#ref = unsafe { slice::from_raw_parts_mut(data, len) };
                (field_layout.clone(), r#ref)
            })
            .collect();
        Self {
            refs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn into<T>(self, context: &T::Context) -> T::RefsMut<'a>
    where
        T: Soa<Fields = Fields>,
    {
        let Self { refs, .. } = self;

        let field_layouts: Box<[_]> = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>)
            .collect();
        assert_eq!(field_layouts.len(), refs.len());

        let ptrs = field_layouts
            .iter()
            .zip(refs)
            .map(|(field_layout, (layout, r#ref))| {
                assert_eq!(field_layout, &layout);
                r#ref.as_mut_ptr()
            });
        let ptrs = T::ptrs_restore_mut(context, ptrs);
        unsafe { T::ptrs_to_refs_mut(context, ptrs) }
    }
}

impl<'a, Fields> AsRef<[(Layout, ErasedFieldRefMut<'a>)]> for ErasedSoaRefsMut<'a, Fields> {
    fn as_ref(&self) -> &[(Layout, ErasedFieldRefMut<'a>)] {
        let Self { refs, .. } = self;
        refs.as_ref()
    }
}

impl<'a, Fields> AsMut<[(Layout, ErasedFieldRefMut<'a>)]> for ErasedSoaRefsMut<'a, Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, ErasedFieldRefMut<'a>)] {
        let Self { refs, .. } = self;
        refs.as_mut()
    }
}

impl<'a, Fields> Debug for ErasedSoaRefsMut<'a, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ErasedSoaRefsMut").field(&self.refs).finish()
    }
}

impl<'r, 'a, Fields> IntoIterator for &'r ErasedSoaRefsMut<'a, Fields> {
    type Item = &'r (Layout, ErasedFieldRefMut<'a>);

    type IntoIter = core::slice::Iter<'r, (Layout, ErasedFieldRefMut<'a>)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaRefsMut { refs, .. } = self;
        refs.iter()
    }
}

impl<'r, 'a, Fields> IntoIterator for &'r mut ErasedSoaRefsMut<'a, Fields> {
    type Item = &'r mut (Layout, ErasedFieldRefMut<'a>);

    type IntoIter = core::slice::IterMut<'r, (Layout, ErasedFieldRefMut<'a>)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaRefsMut { refs, .. } = self;
        refs.iter_mut()
    }
}

impl<'a, Fields> IntoIterator for ErasedSoaRefsMut<'a, Fields> {
    type Item = (Layout, ErasedFieldRefMut<'a>);

    type IntoIter = alloc::vec::IntoIter<(Layout, ErasedFieldRefMut<'a>)>;

    fn into_iter(self) -> Self::IntoIter {
        let ErasedSoaRefsMut { refs, .. } = self;
        refs.into_vec().into_iter()
    }
}

unsafe impl<'a, Fields> Send for ErasedSoaRefsMut<'a, Fields> where Fields: Send {}
unsafe impl<'a, Fields> Sync for ErasedSoaRefsMut<'a, Fields> where Fields: Sync {}

// data is stored inline in a single buffer
type ErasedFieldSlicePtr = *const [u8];

pub struct ErasedSoaSlicePtrs<Fields> {
    len: usize,
    slices: Box<[(Layout, ErasedFieldSlicePtr)]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaSlicePtrs<Fields> {
    pub fn new<I>(context: &ErasedSoaContext<Fields>, len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSlicePtr>,
    {
        let ErasedSoaContext { field_layouts, .. } = context;
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
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let slices = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size() * len;
                let slice = ptr::slice_from_raw_parts(ptr.cast(), len);
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
    pub unsafe fn into<T>(self, context: &T::Context) -> T::SlicePtrs
    where
        T: Soa<Fields = Fields>,
    {
        let Self { slices, len, .. } = self;

        let field_layouts: Box<[_]> = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>)
            .collect();
        assert_eq!(slices.len(), field_layouts.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                slice.cast()
            });
        let ptrs = T::ptrs_restore(context, ptrs);
        T::slices_from_raw_parts(context, ptrs, len)
    }
}

impl<Fields> AsRef<[(Layout, ErasedFieldSlicePtr)]> for ErasedSoaSlicePtrs<Fields> {
    fn as_ref(&self) -> &[(Layout, ErasedFieldSlicePtr)] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }
}

impl<Fields> AsMut<[(Layout, ErasedFieldSlicePtr)]> for ErasedSoaSlicePtrs<Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, ErasedFieldSlicePtr)] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }
}

impl<Fields> Debug for ErasedSoaSlicePtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ErasedSoaSlicePtrs")
            .field("len", &self.len)
            .field("slices", &self.slices)
            .finish()
    }
}

impl<Fields> PartialEq for ErasedSoaSlicePtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.slices == other.slices && self.phantom == other.phantom
    }
}

impl<Fields> Eq for ErasedSoaSlicePtrs<Fields> {}

impl<Fields> Hash for ErasedSoaSlicePtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.len.hash(state);
        self.slices.hash(state);
        self.phantom.hash(state);
    }
}

impl<Fields> Clone for ErasedSoaSlicePtrs<Fields> {
    fn clone(&self) -> Self {
        Self {
            len: self.len.clone(),
            slices: self.slices.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

// data is stored inline in a single buffer
type ErasedFieldSliceMutPtr = *mut [u8];

pub struct ErasedSoaSliceMutPtrs<Fields> {
    len: usize,
    slices: Box<[(Layout, ErasedFieldSliceMutPtr)]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<Fields> ErasedSoaSliceMutPtrs<Fields> {
    pub fn new<I>(context: &ErasedSoaContext<Fields>, len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSliceMutPtr>,
    {
        let ErasedSoaContext { field_layouts, .. } = context;
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
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let slices = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size() * len;
                let slice = ptr::slice_from_raw_parts_mut(ptr.cast(), len);
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
    pub unsafe fn into<T>(self, context: &T::Context) -> T::SliceMutPtrs
    where
        T: Soa<Fields = Fields>,
    {
        let Self { slices, len, .. } = self;

        let field_layouts: Box<[_]> = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>)
            .collect();
        assert_eq!(slices.len(), field_layouts.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                slice.cast()
            });
        let ptrs = T::ptrs_restore_mut(context, ptrs);
        T::slices_from_raw_parts_mut(context, ptrs, len)
    }
}

impl<Fields> AsRef<[(Layout, ErasedFieldSliceMutPtr)]> for ErasedSoaSliceMutPtrs<Fields> {
    fn as_ref(&self) -> &[(Layout, ErasedFieldSliceMutPtr)] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }
}

impl<Fields> AsMut<[(Layout, ErasedFieldSliceMutPtr)]> for ErasedSoaSliceMutPtrs<Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, ErasedFieldSliceMutPtr)] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }
}

impl<Fields> Debug for ErasedSoaSliceMutPtrs<Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ErasedSoaSliceMutPtrs")
            .field("len", &self.len)
            .field("slices", &self.slices)
            .finish()
    }
}

impl<Fields> PartialEq for ErasedSoaSliceMutPtrs<Fields> {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.slices == other.slices && self.phantom == other.phantom
    }
}

impl<Fields> Eq for ErasedSoaSliceMutPtrs<Fields> {}

impl<Fields> Hash for ErasedSoaSliceMutPtrs<Fields> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.len.hash(state);
        self.slices.hash(state);
        self.phantom.hash(state);
    }
}

impl<Fields> Clone for ErasedSoaSliceMutPtrs<Fields> {
    fn clone(&self) -> Self {
        Self {
            len: self.len.clone(),
            slices: self.slices.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

// data is stored inline in a single buffer
type ErasedFieldSlice<'a> = &'a [u8];

pub struct ErasedSoaSlices<'a, Fields>
where
    Fields: 'a,
{
    len: usize,
    slices: Box<[(Layout, ErasedFieldSlice<'a>)]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaSlices<'a, Fields> {
    pub fn new<I>(context: &ErasedSoaContext<Fields>, len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSlice<'a>>,
    {
        let ErasedSoaContext { field_layouts, .. } = context;
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
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let slices = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size() * len;
                let slice = unsafe { slice::from_raw_parts(ptr.cast(), len) };
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
    pub unsafe fn into<T>(self, context: &T::Context) -> T::Slices<'a>
    where
        T: Soa<Fields = Fields>,
    {
        let Self { slices, len, .. } = self;

        let field_layouts: Box<[_]> = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>)
            .collect();
        assert_eq!(slices.len(), field_layouts.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                slice.as_ptr()
            });
        let ptrs = T::ptrs_restore(context, ptrs);
        let slices = T::slices_from_raw_parts(context, ptrs, len);
        unsafe { T::slice_ptrs_to_slices(context, slices) }
    }
}

impl<'a, Fields> AsRef<[(Layout, ErasedFieldSlice<'a>)]> for ErasedSoaSlices<'a, Fields> {
    fn as_ref(&self) -> &[(Layout, ErasedFieldSlice<'a>)] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }
}

impl<'a, Fields> AsMut<[(Layout, ErasedFieldSlice<'a>)]> for ErasedSoaSlices<'a, Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, ErasedFieldSlice<'a>)] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }
}

impl<'a, Fields> Debug for ErasedSoaSlices<'a, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ErasedSoaSlices")
            .field("len", &self.len)
            .field("slices", &self.slices)
            .finish()
    }
}

impl<'a, Fields> Clone for ErasedSoaSlices<'a, Fields> {
    fn clone(&self) -> Self {
        Self {
            len: self.len.clone(),
            slices: self.slices.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

unsafe impl<'a, Fields> Send for ErasedSoaSlices<'a, Fields> where Fields: Sync {}
unsafe impl<'a, Fields> Sync for ErasedSoaSlices<'a, Fields> where Fields: Sync {}

// data is stored inline in a single buffer
type ErasedFieldSliceMut<'a> = &'a mut [u8];

pub struct ErasedSoaSlicesMut<'a, Fields>
where
    Fields: 'a,
{
    len: usize,
    slices: Box<[(Layout, ErasedFieldSliceMut<'a>)]>,
    phantom: PhantomData<fn() -> Fields>,
}

impl<'a, Fields> ErasedSoaSlicesMut<'a, Fields> {
    pub fn new<I>(context: &ErasedSoaContext<Fields>, len: usize, slices: I) -> Self
    where
        I: IntoIterator<Item = ErasedFieldSliceMut<'a>>,
    {
        let ErasedSoaContext { field_layouts, .. } = context;
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
        let field_layouts = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>);

        let slices = field_layouts
            .zip(ptrs)
            .map(|(field_layout, ptr)| {
                let len = field_layout.size() * len;
                let slice = unsafe { slice::from_raw_parts_mut(ptr.cast(), len) };
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
    pub unsafe fn into<T>(self, context: &T::Context) -> T::SlicesMut<'a>
    where
        T: Soa<Fields = Fields>,
    {
        let Self { slices, len, .. } = self;

        let field_layouts: Box<[_]> = T::field_layouts(context)
            .into_iter()
            .map(validate_layout::<Fields, _>)
            .collect();
        assert_eq!(slices.len(), field_layouts.len());

        let ptrs = field_layouts
            .iter()
            .zip(slices)
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                slice.as_mut_ptr()
            });

        let ptrs = T::ptrs_restore_mut(context, ptrs);
        let slices = T::slices_from_raw_parts_mut(context, ptrs, len);
        unsafe { T::slice_ptrs_to_slices_mut(context, slices) }
    }
}

impl<'a, Fields> AsRef<[(Layout, ErasedFieldSliceMut<'a>)]> for ErasedSoaSlicesMut<'a, Fields> {
    fn as_ref(&self) -> &[(Layout, ErasedFieldSliceMut<'a>)] {
        let Self { slices, .. } = self;
        slices.as_ref()
    }
}

impl<'a, Fields> AsMut<[(Layout, ErasedFieldSliceMut<'a>)]> for ErasedSoaSlicesMut<'a, Fields> {
    fn as_mut(&mut self) -> &mut [(Layout, ErasedFieldSliceMut<'a>)] {
        let Self { slices, .. } = self;
        slices.as_mut()
    }
}

impl<'a, Fields> Debug for ErasedSoaSlicesMut<'a, Fields> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ErasedSoaSlicesMut")
            .field("len", &self.len)
            .field("slices", &self.slices)
            .finish()
    }
}

unsafe impl<'a, Fields> Send for ErasedSoaSlicesMut<'a, Fields> where Fields: Send {}
unsafe impl<'a, Fields> Sync for ErasedSoaSlicesMut<'a, Fields> where Fields: Sync {}

unsafe impl<Fields> Soa for ErasedSoa<Fields> {
    type Context = ErasedSoaContext<Fields>;

    type Fields = Fields;

    type FieldLayouts<'a> = &'a [Layout];

    fn field_layouts(context: &Self::Context) -> Self::FieldLayouts<'_> {
        let ErasedSoaContext { field_layouts, .. } = context;
        field_layouts.as_ref()
    }

    type Ptrs = ErasedSoaPtrs<Fields>;

    type MutPtrs = ErasedSoaMutPtrs<Fields>;

    unsafe fn ptrs(
        context: &Self::Context,
        ptr: *mut u8,
        offsets: impl IntoIterator<Item = usize>,
    ) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;

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

        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn ptrs_dangling(context: &Self::Context) -> Self::MutPtrs {
        let ErasedSoaContext { field_layouts, .. } = context;

        let ptrs = field_layouts
            .iter()
            .map(|field_layout| {
                let data = ptr::without_provenance_mut(field_layout.align());
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (field_layout.clone(), ptr)
            })
            .collect();
        ErasedSoaMutPtrs {
            ptrs,
            phantom: PhantomData,
        }
    }

    fn ptrs_erase(
        context: &Self::Context,
        ptrs: Self::Ptrs,
    ) -> impl IntoIterator<Item = *const u8> {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        ptrs.into_vec().into_iter().map(|(_, ptr)| ptr.cast())
    }

    fn ptrs_erase_mut(
        context: &Self::Context,
        ptrs: Self::MutPtrs,
    ) -> impl IntoIterator<Item = *mut u8> {
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs, .. } = ptrs;

        assert_eq!(field_layouts.len(), ptrs.len());

        ptrs.into_vec().into_iter().map(|(_, ptr)| ptr.cast())
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
                let data = ptr.cast();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts(data, len);
                (field_layout.clone(), ptr)
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
                let data = ptr.cast();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (field_layout.clone(), ptr)
            })
            .collect();
        assert_eq!(field_layouts.len(), ptrs.len());

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
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                (layout, ptr.cast_const())
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
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                (layout, ptr.cast_mut())
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
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs, .. } = ptrs;
        let ErasedSoaPtrs { ptrs: origin, .. } = origin;

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
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaMutPtrs { ptrs: a, .. } = a;
        let ErasedSoaMutPtrs { ptrs: b, .. } = b;

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
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs: src, .. } = src;
        let ErasedSoaMutPtrs { ptrs: dst, .. } = dst;

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
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs: src, .. } = src;
        let ErasedSoaMutPtrs { ptrs: dst, .. } = dst;

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
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs: src, .. } = src;
        let ErasedSoaMutPtrs { ptrs: dst, .. } = dst;

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
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaPtrs { ptrs: src, .. } = src;
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
        assert_eq!(field_layouts.len(), ptrs.len());

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
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                // assert_eq!(field_layout.size(), ptr.len());
                (layout, unsafe { NonNull::new_unchecked(ptr) })
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
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                // assert_eq!(field_layout.size(), ptr.len());
                (field_layout.clone(), ptr.as_ptr())
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
                let capacity = (capacity * field_layout.size()).div_ceil(size_of::<Byte<Fields>>());
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
                let ptr = ptr::slice_from_raw_parts(data, len);
                (field_layout.clone(), ptr)
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
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (field_layout.clone(), ptr)
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
            let len = (len * vec_field_layout.size()).div_ceil(size_of::<Byte<Fields>>());
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
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(field_layout.size(), ptr.len());

                let r#ref = unsafe { slice::from_raw_parts(ptr.cast(), ptr.len()) };
                (field_layout.clone(), r#ref)
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
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(field_layout.size(), ptr.len());

                let r#ref = unsafe { slice::from_raw_parts_mut(ptr.cast(), ptr.len()) };
                (layout, r#ref)
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
            .map(|(field_layout, (layout, r#ref))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(field_layout.size(), r#ref.len());

                (layout, ptr::from_ref(r#ref))
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
            .map(|(field_layout, (layout, r#ref))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(field_layout.size(), r#ref.len());
                (layout, ptr::from_mut(r#ref))
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
            .map(|(field_layout, (layout, r#ref))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(field_layout.size(), r#ref.len());
                (layout, &*r#ref)
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
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                // assert_eq!(field_layout.size(), ptr.len());

                let data = ptr.cast();
                let len = len * field_layout.size();
                let slice = ptr::slice_from_raw_parts(data, len);
                (layout, slice)
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
            .map(|(field_layout, (layout, ptr))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(field_layout.size(), ptr.len());

                let data = ptr.cast();
                let len = len * field_layout.size();
                let slice = ptr::slice_from_raw_parts_mut(data, len);
                (layout, slice)
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
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                (layout, slice.cast_const())
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
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                (layout, slice.cast_mut())
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
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.cast();
                let len = slice.len().checked_div(field_layout.size()).unwrap_or(0);
                let ptr = ptr::slice_from_raw_parts(data, len);
                (layout, ptr)
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
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.cast();
                let len = slice.len().checked_div(field_layout.size()).unwrap_or(0);
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (layout, ptr)
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
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.cast();
                let len = slice.len();
                let slice = unsafe { slice::from_raw_parts(data, len) };
                (layout, slice)
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
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.cast();
                let len = slice.len();
                let slice = unsafe { slice::from_raw_parts_mut(data, len) };
                (layout, slice)
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
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlicesMut { slices, .. } = slices;

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
        let ErasedSoaContext { field_layouts, .. } = context;
        let ErasedSoaSlices { slices, len, .. } = slices;

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
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                (layout, ptr::from_mut(slice))
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
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);
                (layout, &*slice)
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
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.as_ptr();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts(data, len);
                (layout, ptr)
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
            .map(|(field_layout, (layout, slice))| {
                assert_eq!(field_layout, &layout);
                assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

                let data = slice.as_mut_ptr();
                let len = field_layout.size();
                let ptr = ptr::slice_from_raw_parts_mut(data, len);
                (layout, ptr)
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

        let ErasedSoaSliceMutPtrs {
            mut slices, len, ..
        } = slices;
        assert_eq!(field_layouts.len(), slices.len());

        for ((ref layout, ref mut slice), field_layout) in iter::zip(&mut slices, field_layouts) {
            assert_eq!(layout, field_layout);
            assert_eq!(slice.len().checked_rem(field_layout.size()).unwrap_or(0), 0);

            let data = slice.cast();
            let len = slice.len().checked_div(field_layout.size()).unwrap_or(0);
            *slice = ptr::slice_from_raw_parts_mut(data, len);
        }

        for _ in 0..len {
            drop_fields(slices.as_ref());

            for (ref field_layout, ref mut slice) in slices.iter_mut() {
                let len = field_layout.size();
                let data = unsafe { slice.cast::<u8>().add(len) };
                *slice = ptr::slice_from_raw_parts_mut(data, len);
            }
        }
    }
}

#[inline]
fn validate_layout<Fields, I>(item: I) -> Layout
where
    I: Borrow<Layout>,
{
    let layout: &Layout = item.borrow();

    let input_align = layout.align();
    let max_align = align_of::<Fields>();
    assert!(
        input_align <= max_align,
        "input alignment must be less than or equal to {max_align}, but got {input_align}",
    );
    layout.clone()
}
