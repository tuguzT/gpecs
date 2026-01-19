use core::{
    alloc::Layout,
    cmp,
    fmt::{self, Debug},
    hash::{self, Hash},
    marker::PhantomData,
};

use crate::{
    error::{InsufficientAlignError, check_sufficient_align},
    soa::{
        field::FieldDescriptor,
        traits::{AllocSoa, AllocSoaContext},
    },
    storage::AddressableUnit,
};

#[cfg(feature = "alloc")]
pub type BoxedErasedSoaContext = ErasedSoaContext<alloc::boxed::Box<[FieldDescriptor]>, u8>;

pub struct ErasedSoaContext<D, A>
where
    A: AddressableUnit,
    D: ?Sized,
{
    phantom: PhantomData<fn() -> A>,
    descriptors: D,
}

impl<D, A> ErasedSoaContext<D, A>
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
    pub fn into_field_descriptors(self) -> D {
        let Self { descriptors, .. } = self;
        descriptors
    }
}

impl<D, A> ErasedSoaContext<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn new(descriptors: D) -> Result<Self, InsufficientAlignError> {
        descriptors
            .as_ref()
            .iter()
            .try_for_each(|desc| check_sufficient_align(desc.layout(), Layout::new::<A>()))?;

        let me = unsafe { Self::new_unchecked(descriptors) };
        Ok(me)
    }
}

impl<D, A> ErasedSoaContext<D, A>
where
    A: AddressableUnit,
    D: AsRef<[FieldDescriptor]> + ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }
}

impl<D, A> ErasedSoaContext<D, A>
where
    A: AddressableUnit,
    D: FromIterator<FieldDescriptor>,
{
    #[inline]
    pub fn of<T>(context: &T::Context) -> Result<Self, InsufficientAlignError>
    where
        T: AllocSoa + ?Sized,
    {
        let descriptors = context
            .field_descriptors()
            .into_iter()
            .map(|desc| {
                let desc = desc.as_ref();
                check_sufficient_align(desc.layout(), Layout::new::<A>())?;
                Ok(*desc)
            })
            .collect::<Result<_, _>>()?;

        let me = unsafe { Self::new_unchecked(descriptors) };
        Ok(me)
    }
}

impl<D, A> Debug for ErasedSoaContext<D, A>
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

impl<D, A> Clone for ErasedSoaContext<D, A>
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

impl<D, A> Copy for ErasedSoaContext<D, A>
where
    A: AddressableUnit,
    D: Copy,
{
}

impl<D, A> PartialEq for ErasedSoaContext<D, A>
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

impl<D, A> Eq for ErasedSoaContext<D, A>
where
    A: AddressableUnit,
    D: Eq + ?Sized,
{
}

impl<D, A> PartialOrd for ErasedSoaContext<D, A>
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

impl<D, A> Ord for ErasedSoaContext<D, A>
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

impl<D, A> Hash for ErasedSoaContext<D, A>
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
