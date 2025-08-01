use crate::soa::traits::{FieldDescriptor, Soa};

#[cfg(feature = "alloc")]
pub type BoxedErasedSoaContext = ErasedSoaContext<alloc::boxed::Box<[FieldDescriptor]>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ErasedSoaContext<D> {
    descriptors: D,
}

impl<D> ErasedSoaContext<D> {
    #[inline]
    pub fn new(descriptors: D) -> Self {
        Self { descriptors }
    }

    #[inline]
    pub fn into_field_descriptors(self) -> D {
        let Self { descriptors } = self;
        descriptors
    }
}

impl<D> ErasedSoaContext<D>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors } = self;
        descriptors.as_ref()
    }
}

impl<D> ErasedSoaContext<D>
where
    D: FromIterator<FieldDescriptor>,
{
    #[inline]
    pub fn of<T>(context: &T::Context) -> Self
    where
        T: Soa + ?Sized,
    {
        let descriptors = T::field_descriptors(context);
        descriptors.into_iter().collect()
    }
}

impl<A, D> FromIterator<A> for ErasedSoaContext<D>
where
    A: AsRef<FieldDescriptor>,
    D: FromIterator<FieldDescriptor>,
{
    #[inline]
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = A>,
    {
        let descriptors = iter.into_iter().map(|desc| *desc.as_ref()).collect();
        Self { descriptors }
    }
}

impl<D> IntoIterator for ErasedSoaContext<D>
where
    D: IntoIterator<Item = FieldDescriptor>,
{
    type Item = FieldDescriptor;
    type IntoIter = D::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let Self { descriptors } = self;
        descriptors.into_iter()
    }
}
