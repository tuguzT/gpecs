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
    CovariantFieldDescriptors, ErasedSoaMutPtrs, ErasedSoaPtrs,
    error::{InsufficientAlignError, check_sufficient_align},
    ptr::slice::SliceItemPtrs,
    soa::{
        field::{
            FieldDescriptor, FieldDescriptors, FieldDescriptorsOutput, FieldDescriptorsOwned,
            IntoCopiedFieldDescriptors, buffer_layout,
        },
        traits::AllocSoa,
    },
};

#[cfg(feature = "alloc")]
pub type BoxedErasedSoaContext<P> = ErasedSoaContext<alloc::boxed::Box<[FieldDescriptor]>, P>;

#[repr(transparent)]
pub struct ErasedSoaContext<D, P>
where
    D: ?Sized,
    P: SliceItemPtrs,
{
    phantom: PhantomData<P>,
    descriptors: D,
}

impl<D, P> ErasedSoaContext<D, P>
where
    P: SliceItemPtrs,
{
    #[inline]
    pub unsafe fn from_inner(descriptors: D) -> Self {
        Self {
            phantom: PhantomData,
            descriptors,
        }
    }

    #[inline]
    pub fn into_inner(self) -> D {
        let Self { descriptors, .. } = self;
        descriptors
    }
}

impl<D, P> ErasedSoaContext<D, P>
where
    D: FieldDescriptorsOwned,
    P: SliceItemPtrs,
{
    #[inline]
    pub fn new(descriptors: D) -> Result<Self, InsufficientAlignError> {
        descriptors
            .field_descriptors()
            .copied_field_descriptors()
            .try_for_each(|desc| check_sufficient_align(desc.layout(), Layout::new::<P::Item>()))?;

        let me = unsafe { Self::from_inner(descriptors) };
        Ok(me)
    }
}

impl<D, P> ErasedSoaContext<D, P>
where
    D: ?Sized,
    P: SliceItemPtrs,
{
    #[inline]
    pub const unsafe fn from_inner_ref(descriptors: &D) -> &Self {
        // SAFETY: Self is `#[repr(transparent)]` over `D`.
        unsafe { &*(ptr::from_ref(descriptors) as *const _) }
    }

    #[inline]
    pub const unsafe fn from_inner_mut(descriptors: &mut D) -> &mut Self {
        // SAFETY: Self is `#[repr(transparent)]` over `V::Context`.
        unsafe { &mut *(ptr::from_mut(descriptors) as *mut _) }
    }

    #[inline]
    pub const fn as_inner(&self) -> &D {
        let Self { descriptors, .. } = self;
        descriptors
    }

    #[inline]
    pub const fn as_inner_mut(&mut self) -> &mut D {
        let Self { descriptors, .. } = self;
        descriptors
    }
}

impl<D, P> ErasedSoaContext<D, P>
where
    D: FromIterator<FieldDescriptor>,
    P: SliceItemPtrs,
{
    #[inline]
    pub fn of<T>(context: &T::Context) -> Result<Self, InsufficientAlignError>
    where
        T: AllocSoa + ?Sized,
    {
        let descriptors = context
            .field_descriptors()
            .copied_field_descriptors()
            .map(|desc| {
                check_sufficient_align(desc.layout(), Layout::new::<P::Item>())?;
                Ok(desc)
            })
            .collect::<Result<_, _>>()?;

        let me = unsafe { Self::from_inner(descriptors) };
        Ok(me)
    }
}

impl<'a, D, P> ErasedSoaContext<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: SliceItemPtrs,
{
    #[inline]
    pub fn field_descriptors(&'a self) -> D::Output {
        let Self { descriptors, .. } = self;
        descriptors.field_descriptors()
    }

    #[inline]
    pub fn buffer_layout(&'a self, capacity: usize) -> Result<Layout, LayoutError> {
        let fields = self.field_descriptors();
        buffer_layout(fields, capacity)
    }

    #[inline]
    pub unsafe fn ptrs_from_buffer(
        &'a self,
        buffer: *const u8,
        capacity: usize,
    ) -> ErasedSoaPtrs<D::Output, P::Const> {
        let descriptors = self.field_descriptors();
        let layout = unsafe { self.buffer_layout(capacity).unwrap_unchecked() };
        let buffer = ptr::slice_from_raw_parts(buffer.cast(), layout.size());
        unsafe { ErasedSoaPtrs::new_unchecked(descriptors, buffer, capacity, 0) }
    }

    #[inline]
    pub unsafe fn ptrs_from_buffer_mut(
        &'a self,
        buffer: *mut u8,
        capacity: usize,
    ) -> ErasedSoaMutPtrs<D::Output, P::Mut> {
        let descriptors = self.field_descriptors();
        let layout = unsafe { self.buffer_layout(capacity).unwrap_unchecked() };
        let buffer = ptr::slice_from_raw_parts_mut(buffer.cast(), layout.size());
        unsafe { ErasedSoaMutPtrs::new_unchecked(descriptors, buffer, capacity, 0) }
    }
}

impl<D, P> Debug for ErasedSoaContext<D, P>
where
    D: Debug + ?Sized,
    P: SliceItemPtrs,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { descriptors, .. } = self;
        f.debug_tuple("ErasedSoaContext")
            .field(&descriptors)
            .finish()
    }
}

impl<D, P> Clone for ErasedSoaContext<D, P>
where
    D: Clone,
    P: SliceItemPtrs,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { descriptors, .. } = self;
        unsafe { Self::from_inner(descriptors.clone()) }
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
        let Self {
            phantom,
            descriptors,
        } = self;
        *phantom == other.phantom && *descriptors == other.descriptors
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
        let Self {
            phantom,
            descriptors,
        } = self;

        match phantom.partial_cmp(&other.phantom) {
            Some(cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        descriptors.partial_cmp(&other.descriptors)
    }
}

impl<D, P> Ord for ErasedSoaContext<D, P>
where
    D: Ord + ?Sized,
    P: SliceItemPtrs,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let Self {
            phantom,
            descriptors,
        } = self;

        match phantom.cmp(&other.phantom) {
            cmp::Ordering::Equal => {}
            ord => return ord,
        }
        descriptors.cmp(&other.descriptors)
    }
}

impl<D, P> Hash for ErasedSoaContext<D, P>
where
    D: Hash + ?Sized,
    P: SliceItemPtrs,
{
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self {
            phantom,
            descriptors,
        } = self;

        phantom.hash(state);
        descriptors.hash(state);
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

impl<'a, D, P> FieldDescriptors<'a> for ErasedSoaContext<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: SliceItemPtrs,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        Self::field_descriptors(self)
    }
}

impl<D, P> CovariantFieldDescriptors for ErasedSoaContext<D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    P: SliceItemPtrs,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: FieldDescriptorsOutput<'long, Self>,
    ) -> FieldDescriptorsOutput<'short, Self> {
        D::upcast_field_descriptors(from)
    }
}
