use core::{
    alloc::{Layout, LayoutError},
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr,
};

use crate::{
    CovariantFieldLayouts, ErasedSoaMutPtrs, ErasedSoaPtrs,
    error::{InsufficientAlignError, check_sufficient_align},
    ptr::slice::SliceItemPtrs,
    soa::{
        field::{
            BufferLayout, FieldLayouts, FieldLayoutsItem, FieldLayoutsOutput, FieldLayoutsOwned,
            buffer_layout,
        },
        layout::WithLayout,
        traits::AllocSoa,
    },
};

#[cfg(feature = "alloc")]
pub type BoxedErasedSoaContext<P> = ErasedSoaContext<alloc::boxed::Box<[Layout]>, P>;

pub struct ErasedSoaContext<D, P>
where
    D: ?Sized,
    P: SliceItemPtrs,
{
    phantom: PhantomData<P>,
    layouts: D,
}

impl<D, P> ErasedSoaContext<D, P>
where
    P: SliceItemPtrs,
{
    #[inline]
    pub const unsafe fn from_inner(layouts: D) -> Self {
        Self {
            phantom: PhantomData,
            layouts,
        }
    }

    #[inline]
    pub fn into_inner(self) -> D {
        let Self { layouts, .. } = self;
        layouts
    }
}

impl<D, P> ErasedSoaContext<D, P>
where
    D: FieldLayoutsOwned,
    P: SliceItemPtrs,
{
    #[inline]
    pub fn new(layouts: D) -> Result<Self, InsufficientAlignError> {
        layouts
            .field_layouts()
            .into_iter()
            .try_for_each(|item| check_sufficient_align(item.layout(), Layout::new::<P::Item>()))?;

        let me = unsafe { Self::from_inner(layouts) };
        Ok(me)
    }
}

impl<D, P> ErasedSoaContext<D, P>
where
    D: ?Sized,
    P: SliceItemPtrs,
{
    #[inline]
    pub const fn as_inner(&self) -> &D {
        let Self { layouts, .. } = self;
        layouts
    }

    #[inline]
    pub const fn as_inner_mut(&mut self) -> &mut D {
        let Self { layouts, .. } = self;
        layouts
    }
}

impl<D, P> ErasedSoaContext<D, P>
where
    P: SliceItemPtrs,
{
    #[inline]
    pub fn of<'a, T>(context: &'a T::Context) -> Result<Self, InsufficientAlignError>
    where
        T: AllocSoa + ?Sized,
        D: FromIterator<FieldLayoutsItem<'a, T::Context, T>>,
    {
        let layouts = context
            .field_layouts()
            .into_iter()
            .map(|item| {
                check_sufficient_align(item.layout(), Layout::new::<P::Item>())?;
                Ok(item)
            })
            .collect::<Result<_, _>>()?;

        let me = unsafe { Self::from_inner(layouts) };
        Ok(me)
    }
}

impl<'a, D, P> ErasedSoaContext<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: SliceItemPtrs,
{
    #[inline]
    pub fn field_layouts(&'a self) -> D::Output {
        let Self { layouts, .. } = self;
        layouts.field_layouts()
    }

    #[inline]
    pub fn buffer_layout(&'a self, capacity: usize) -> Result<Layout, LayoutError> {
        let fields = self.field_layouts();
        buffer_layout(fields, capacity).map(BufferLayout::layout)
    }

    #[inline]
    pub unsafe fn ptrs_from_buffer(
        &'a self,
        buffer: *const u8,
        capacity: usize,
    ) -> ErasedSoaPtrs<D::Output, P::Const> {
        let field_layouts = self.field_layouts();
        let layout = unsafe { self.buffer_layout(capacity).unwrap_unchecked() };
        let buffer = ptr::slice_from_raw_parts(buffer.cast(), layout.size());
        unsafe { ErasedSoaPtrs::new_unchecked(field_layouts, buffer, capacity, 0) }
    }

    #[inline]
    pub unsafe fn ptrs_from_buffer_mut(
        &'a self,
        buffer: *mut u8,
        capacity: usize,
    ) -> ErasedSoaMutPtrs<D::Output, P::Mut> {
        let field_layouts = self.field_layouts();
        let layout = unsafe { self.buffer_layout(capacity).unwrap_unchecked() };
        let buffer = ptr::slice_from_raw_parts_mut(buffer.cast(), layout.size());
        unsafe { ErasedSoaMutPtrs::new_unchecked(field_layouts, buffer, capacity, 0) }
    }
}

impl<D, P> Debug for ErasedSoaContext<D, P>
where
    D: Debug + ?Sized,
    P: SliceItemPtrs,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { layouts, .. } = self;
        f.debug_tuple("ErasedSoaContext").field(&layouts).finish()
    }
}

impl<D, P> Clone for ErasedSoaContext<D, P>
where
    D: Clone,
    P: SliceItemPtrs,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { layouts, .. } = self;
        unsafe { Self::from_inner(layouts.clone()) }
    }
}

impl<D, P> Copy for ErasedSoaContext<D, P>
where
    D: Copy,
    P: SliceItemPtrs,
{
}

impl<D, P> PartialEq for ErasedSoaContext<D, P>
where
    D: PartialEq + ?Sized,
    P: SliceItemPtrs,
{
    fn eq(&self, other: &Self) -> bool {
        let Self { phantom, layouts } = self;
        *phantom == other.phantom && *layouts == other.layouts
    }
}

impl<D, P> Eq for ErasedSoaContext<D, P>
where
    D: Eq + ?Sized,
    P: SliceItemPtrs,
{
}

impl<D, P> PartialOrd for ErasedSoaContext<D, P>
where
    D: PartialOrd + ?Sized,
    P: SliceItemPtrs,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let Self { phantom, layouts } = self;

        match phantom.partial_cmp(&other.phantom) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        layouts.partial_cmp(&other.layouts)
    }
}

impl<D, P> Ord for ErasedSoaContext<D, P>
where
    D: Ord + ?Sized,
    P: SliceItemPtrs,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self { phantom, layouts } = self;

        match phantom.cmp(&other.phantom) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        layouts.cmp(&other.layouts)
    }
}

impl<D, P> Hash for ErasedSoaContext<D, P>
where
    D: Hash + ?Sized,
    P: SliceItemPtrs,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self { phantom, layouts } = self;

        phantom.hash(state);
        layouts.hash(state);
    }
}

impl<D, P> Deref for ErasedSoaContext<D, P>
where
    D: ?Sized,
    P: SliceItemPtrs,
{
    type Target = D;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_inner()
    }
}

impl<D, P> DerefMut for ErasedSoaContext<D, P>
where
    D: ?Sized,
    P: SliceItemPtrs,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_inner_mut()
    }
}

impl<'a, D, P> FieldLayouts<'a> for ErasedSoaContext<D, P>
where
    D: FieldLayouts<'a> + ?Sized,
    P: SliceItemPtrs,
{
    type Output = D::Output;
    type OutputIter = D::OutputIter;
    type OutputItem = D::OutputItem;

    #[inline]
    fn field_layouts(&'a self) -> Self::Output {
        Self::field_layouts(self)
    }
}

impl<D, P> CovariantFieldLayouts for ErasedSoaContext<D, P>
where
    D: CovariantFieldLayouts + ?Sized,
    P: SliceItemPtrs,
{
    #[inline]
    fn upcast_field_layouts<'short, 'long: 'short>(
        from: FieldLayoutsOutput<'long, Self>,
    ) -> FieldLayoutsOutput<'short, Self> {
        D::upcast_field_layouts(from)
    }
}
