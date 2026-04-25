use core::iter::{FusedIterator, Take};

pub enum MaybeTake<I> {
    Bound(Take<I>),
    Unbound(I),
}

impl<I> Iterator for MaybeTake<I>
where
    I: Iterator,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Bound(iter) => iter.next(),
            Self::Unbound(iter) => iter.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Bound(iter) => iter.size_hint(),
            Self::Unbound(iter) => iter.size_hint(),
        }
    }
}

impl<I> DoubleEndedIterator for MaybeTake<I>
where
    I: DoubleEndedIterator + ExactSizeIterator,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Self::Bound(iter) => iter.next_back(),
            Self::Unbound(iter) => iter.next_back(),
        }
    }
}

impl<I> ExactSizeIterator for MaybeTake<I> where I: ExactSizeIterator {}

impl<I> FusedIterator for MaybeTake<I> where I: FusedIterator {}
