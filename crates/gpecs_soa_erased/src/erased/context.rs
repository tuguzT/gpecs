use core::{
    alloc::Layout,
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
};

use crate::{
    erased::CovariantFieldDescriptors,
    error::{InsufficientAlignError, check_sufficient_align},
    ptr::slice::SliceItemPtrs,
    soa::{
        field::{
            FieldDescriptor, FieldDescriptors, FieldDescriptorsOwned, IntoCopiedFieldDescriptors,
        },
        traits::RawSoa,
    },
};

#[cfg(feature = "alloc")]
pub type BoxedErasedSoaContext<P> = ErasedSoaContext<alloc::boxed::Box<[FieldDescriptor]>, P>;

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
    pub unsafe fn new_unchecked(descriptors: D) -> Self {
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

        let me = unsafe { Self::new_unchecked(descriptors) };
        Ok(me)
    }
}

impl<D, P> ErasedSoaContext<D, P>
where
    D: ?Sized,
    P: SliceItemPtrs,
{
    #[inline]
    pub fn as_inner(&self) -> &D {
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
    pub fn of<'a, T>(context: &'a T::Context) -> Result<Self, InsufficientAlignError>
    where
        T: RawSoa + ?Sized,
        T::Context: FieldDescriptors<'a>,
    {
        let descriptors = context
            .field_descriptors()
            .copied_field_descriptors()
            .map(|desc| {
                check_sufficient_align(desc.layout(), Layout::new::<P::Item>())?;
                Ok(desc)
            })
            .collect::<Result<_, _>>()?;

        let me = unsafe { Self::new_unchecked(descriptors) };
        Ok(me)
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
        unsafe { Self::new_unchecked(descriptors.clone()) }
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

impl<'a, D, P> FieldDescriptors<'a> for ErasedSoaContext<D, P>
where
    D: FieldDescriptors<'a> + ?Sized,
    P: SliceItemPtrs,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { descriptors, .. } = self;
        descriptors.field_descriptors()
    }
}

impl<D, P> CovariantFieldDescriptors for ErasedSoaContext<D, P>
where
    D: CovariantFieldDescriptors + ?Sized,
    P: SliceItemPtrs,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        D::upcast_field_descriptors(from)
    }
}
