use alloc::boxed::Box;
use core::iter::FusedIterator;

use crate::{
    erased::{NewBytes, context::BorrowBytes},
    soa::traits::{FieldDescriptor, Soa},
};

pub type BoxedErasedSoaContext = ErasedSoaContext<Box<[FieldDescriptor]>, NewBytes>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ErasedSoaContext<D, B>
where
    B: ?Sized,
{
    descriptors: D,
    borrow_bytes: B,
}

impl<D, B> ErasedSoaContext<D, B> {
    #[inline]
    pub fn new(descriptors: D, borrow_bytes: B) -> Self {
        Self {
            descriptors,
            borrow_bytes,
        }
    }

    #[inline]
    pub fn into_parts(self) -> (D, B) {
        let Self {
            descriptors,
            borrow_bytes,
        } = self;
        (descriptors, borrow_bytes)
    }
}

impl<D, B> ErasedSoaContext<D, B>
where
    D: AsRef<[FieldDescriptor]>,
    B: ?Sized,
{
    #[inline]
    pub fn field_descriptors(&self) -> &[FieldDescriptor] {
        let Self { descriptors, .. } = self;
        descriptors.as_ref()
    }
}

impl<D, B> ErasedSoaContext<D, B>
where
    B: BorrowBytes + ?Sized,
{
    #[inline]
    pub fn borrow_bytes(&self, count: usize) -> Result<B::Output<'_>, B::Error> {
        let Self { borrow_bytes, .. } = self;
        borrow_bytes.borrow_bytes(count)
    }
}

impl<D, B> ErasedSoaContext<D, B>
where
    D: FromIterator<FieldDescriptor>,
    B: Default,
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

impl<A, D, B> FromIterator<A> for ErasedSoaContext<D, B>
where
    A: AsRef<FieldDescriptor>,
    D: FromIterator<FieldDescriptor>,
    B: Default,
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

pub struct ErasedSoaContextIntoIter<I, B>
where
    B: ?Sized,
{
    iter: I,
    borrow_bytes: B,
}

impl<I, B> ErasedSoaContextIntoIter<I, B> {
    #[inline]
    pub fn into_parts(self) -> (I, B) {
        let Self { iter, borrow_bytes } = self;
        (iter, borrow_bytes)
    }
}

impl<I, B> Iterator for ErasedSoaContextIntoIter<I, B>
where
    I: Iterator,
    B: ?Sized,
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

impl<I, B> DoubleEndedIterator for ErasedSoaContextIntoIter<I, B>
where
    I: DoubleEndedIterator,
    B: ?Sized,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        let Self { iter, .. } = self;
        iter.next_back()
    }
}

impl<I, B> ExactSizeIterator for ErasedSoaContextIntoIter<I, B>
where
    I: ExactSizeIterator,
    B: ?Sized,
{
    #[inline]
    fn len(&self) -> usize {
        let Self { iter, .. } = self;
        iter.len()
    }
}

impl<I, B> FusedIterator for ErasedSoaContextIntoIter<I, B>
where
    I: FusedIterator,
    B: ?Sized,
{
}

impl<D, B> IntoIterator for ErasedSoaContext<D, B>
where
    D: IntoIterator<Item = FieldDescriptor>,
{
    type Item = FieldDescriptor;
    type IntoIter = ErasedSoaContextIntoIter<D::IntoIter, B>;

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
