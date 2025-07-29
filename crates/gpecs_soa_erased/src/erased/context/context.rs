use alloc::boxed::Box;
use core::iter::FusedIterator;

use crate::{
    erased::{NewBytes, context::BorrowBytes},
    soa::traits::{FieldDescriptor, Soa},
};

pub type BoxedErasedSoaContext = ErasedSoaContext<Box<[FieldDescriptor]>, NewBytes>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ErasedSoaContext<D, R> {
    descriptors: D,
    borrow_bytes: R,
}

impl<D, R> ErasedSoaContext<D, R> {
    #[inline]
    pub fn new(descriptors: D, borrow_bytes: R) -> Self {
        Self {
            descriptors,
            borrow_bytes,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (D, R) {
        let Self {
            descriptors,
            borrow_bytes,
        } = self;
        (descriptors, borrow_bytes)
    }
}

impl<D, R> ErasedSoaContext<D, R>
where
    D: AsRef<[FieldDescriptor]>,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }
}

impl<D, R> ErasedSoaContext<D, R>
where
    R: BorrowBytes,
{
    #[inline]
    pub fn borrow_bytes(&self, count: usize) -> Result<R::Output<'_>, R::Error> {
        let Self { borrow_bytes, .. } = self;
        borrow_bytes.borrow_bytes(count)
    }
}

impl<D, R> ErasedSoaContext<D, R>
where
    D: FromIterator<FieldDescriptor>,
    R: Default,
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

impl<A, D, R> FromIterator<A> for ErasedSoaContext<D, R>
where
    A: AsRef<FieldDescriptor>,
    D: FromIterator<FieldDescriptor>,
    R: Default,
{
    #[inline]
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = A>,
    {
        Self {
            descriptors: iter.into_iter().map(|desc| *desc.as_ref()).collect(),
            borrow_bytes: Default::default(),
        }
    }
}

pub struct ErasedSoaContextIntoIter<I, R> {
    iter: I,
    borrow_bytes: R,
}

impl<I, R> ErasedSoaContextIntoIter<I, R> {
    #[inline]
    pub fn into_parts(self) -> (I, R) {
        let Self { iter, borrow_bytes } = self;
        (iter, borrow_bytes)
    }
}

impl<I, R> Iterator for ErasedSoaContextIntoIter<I, R>
where
    I: Iterator,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Self { iter, .. } = self;
        iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self { iter, .. } = self;
        iter.size_hint()
    }
}

impl<I, R> DoubleEndedIterator for ErasedSoaContextIntoIter<I, R>
where
    I: DoubleEndedIterator,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { iter, .. } = self;
        iter.next_back()
    }
}

impl<I, R> ExactSizeIterator for ErasedSoaContextIntoIter<I, R>
where
    I: ExactSizeIterator,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { iter, .. } = self;
        iter.len()
    }
}

impl<I, R> FusedIterator for ErasedSoaContextIntoIter<I, R> where I: FusedIterator {}

impl<D, R> IntoIterator for ErasedSoaContext<D, R>
where
    D: IntoIterator<Item = FieldDescriptor>,
{
    type Item = FieldDescriptor;
    type IntoIter = ErasedSoaContextIntoIter<D::IntoIter, R>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let Self {
            descriptors,
            borrow_bytes,
        } = self;
        ErasedSoaContextIntoIter {
            iter: descriptors.into_iter(),
            borrow_bytes,
        }
    }
}
