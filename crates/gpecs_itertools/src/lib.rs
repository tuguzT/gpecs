//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]

pub use itertools;

pub trait Itertools: itertools::Itertools {
    fn get_pair(self, a: usize, b: usize) -> Option<(Self::Item, Self::Item)>
    where
        Self: Sized,
    {
        let (first, second) = (usize::min(a, b), usize::max(a, b));

        let mut range = self.get(first..=second);
        let first = range.next()?;
        let second = range.last()?;

        let pair = if a < b {
            (first, second)
        } else {
            (second, first)
        };
        Some(pair)
    }
}

impl<T> Itertools for T where T: itertools::Itertools + ?Sized {}
