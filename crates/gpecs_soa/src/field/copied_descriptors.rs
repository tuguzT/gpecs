use core::iter::FusedIterator;

use crate::field::FieldDescriptor;

pub trait IntoCopiedFieldDescriptors: IntoIterator + Sized {
    #[inline]
    fn copied_field_descriptors(self) -> CopiedFieldDescriptors<Self::IntoIter> {
        self.into_iter().into()
    }
}

impl<T> IntoCopiedFieldDescriptors for T where T: IntoIterator {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CopiedFieldDescriptors<T>(pub T)
where
    T: ?Sized;

impl<T> CopiedFieldDescriptors<T> {
    #[inline]
    pub fn into_inner(self) -> T {
        let Self(inner) = self;
        inner
    }
}

impl<T> CopiedFieldDescriptors<T>
where
    T: ?Sized,
{
    #[inline]
    pub const fn as_inner(&self) -> &T {
        let Self(inner) = self;
        inner
    }

    #[inline]
    pub const fn as_mut_inner(&mut self) -> &mut T {
        let Self(inner) = self;
        inner
    }
}

impl<T> From<T> for CopiedFieldDescriptors<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Iterator for CopiedFieldDescriptors<T>
where
    T: Iterator + ?Sized,
    T::Item: AsRef<FieldDescriptor>,
{
    type Item = FieldDescriptor;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self(inner) = self;
        inner.next().map(|desc| *desc.as_ref())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self(inner) = self;
        inner.size_hint()
    }
}

impl<T> DoubleEndedIterator for CopiedFieldDescriptors<T>
where
    T: DoubleEndedIterator + ?Sized,
    T::Item: AsRef<FieldDescriptor>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self(inner) = self;
        inner.next_back().map(|desc| *desc.as_ref())
    }
}

impl<T> ExactSizeIterator for CopiedFieldDescriptors<T>
where
    T: ExactSizeIterator + ?Sized,
    T::Item: AsRef<FieldDescriptor>,
{
    #[inline]
    fn len(&self) -> usize {
        let Self(inner) = self;
        inner.len()
    }
}

impl<T> FusedIterator for CopiedFieldDescriptors<T>
where
    T: FusedIterator + ?Sized,
    T::Item: AsRef<FieldDescriptor>,
{
}
