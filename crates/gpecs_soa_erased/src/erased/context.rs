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
    soa::{
        field::{
            FieldDescriptor, FieldDescriptors, FieldDescriptorsOwned, IntoCopiedFieldDescriptors,
        },
        traits::RawSoa,
    },
    storage::AddressableUnit,
};

#[cfg(feature = "alloc")]
pub type BoxedErasedSoaContext<P> = ErasedSoaContext<alloc::boxed::Box<[FieldDescriptor]>, P, u8>;

pub struct ErasedSoaContext<D, P, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    phantom: PhantomData<fn() -> (P, A)>,
    descriptors: D,
}

impl<D, P, A> ErasedSoaContext<D, P, A>
where
    A: AddressableUnit,
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

impl<D, P, A> ErasedSoaContext<D, P, A>
where
    A: AddressableUnit,
    D: FieldDescriptorsOwned,
{
    #[inline]
    pub fn new(descriptors: D) -> Result<Self, InsufficientAlignError> {
        descriptors
            .field_descriptors()
            .copied_field_descriptors()
            .try_for_each(|desc| check_sufficient_align(desc.layout(), Layout::new::<A>()))?;

        let me = unsafe { Self::new_unchecked(descriptors) };
        Ok(me)
    }
}

impl<D, P, A> ErasedSoaContext<D, P, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    #[inline]
    pub fn as_inner(&self) -> &D {
        let Self { descriptors, .. } = self;
        descriptors
    }
}

impl<D, P, A> ErasedSoaContext<D, P, A>
where
    A: AddressableUnit,
    D: FromIterator<FieldDescriptor>,
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
                check_sufficient_align(desc.layout(), Layout::new::<A>())?;
                Ok(desc)
            })
            .collect::<Result<_, _>>()?;

        let me = unsafe { Self::new_unchecked(descriptors) };
        Ok(me)
    }
}

impl<D, P, A> Debug for ErasedSoaContext<D, P, A>
where
    A: AddressableUnit,
    D: Debug + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { descriptors, .. } = self;
        f.debug_tuple("ErasedSoaContext")
            .field(&descriptors)
            .finish()
    }
}

impl<D, P, A> Clone for ErasedSoaContext<D, P, A>
where
    A: AddressableUnit,
    D: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let Self { descriptors, .. } = self;
        unsafe { Self::new_unchecked(descriptors.clone()) }
    }
}

impl<D, P, A> Copy for ErasedSoaContext<D, P, A>
where
    A: AddressableUnit,
    D: Copy,
{
}

impl<D, P, A> PartialEq for ErasedSoaContext<D, P, A>
where
    A: AddressableUnit,
    D: PartialEq + ?Sized,
{
    fn eq(&self, other: &Self) -> bool {
        let Self {
            phantom,
            descriptors,
        } = self;
        *phantom == other.phantom && *descriptors == other.descriptors
    }
}

impl<D, P, A> Eq for ErasedSoaContext<D, P, A>
where
    A: AddressableUnit,
    D: Eq + ?Sized,
{
}

impl<D, P, A> PartialOrd for ErasedSoaContext<D, P, A>
where
    A: AddressableUnit,
    D: PartialOrd + ?Sized,
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

impl<D, P, A> Ord for ErasedSoaContext<D, P, A>
where
    A: AddressableUnit,
    D: Ord + ?Sized,
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

impl<D, P, A> Hash for ErasedSoaContext<D, P, A>
where
    A: AddressableUnit,
    D: Hash + ?Sized,
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

impl<'a, D, P, A> FieldDescriptors<'a> for ErasedSoaContext<D, P, A>
where
    A: AddressableUnit,
    D: FieldDescriptors<'a> + ?Sized,
{
    type Output = D::Output;

    #[inline]
    fn field_descriptors(&'a self) -> Self::Output {
        let Self { descriptors, .. } = self;
        descriptors.field_descriptors()
    }
}

impl<D, P, A> CovariantFieldDescriptors for ErasedSoaContext<D, P, A>
where
    A: AddressableUnit,
    D: CovariantFieldDescriptors + ?Sized,
{
    #[inline]
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        D::upcast_field_descriptors(from)
    }
}
