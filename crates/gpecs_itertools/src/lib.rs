//! Nothing too special for now...

#![cfg_attr(not(test), no_std)]

pub trait Itertools: Iterator {
    fn get_pair(mut self, a: usize, b: usize) -> Option<(Self::Item, Self::Item)>
    where
        Self: Sized,
    {
        let (first, second) = (usize::min(a, b), usize::max(a, b));

        let first_item = self.nth(first)?;
        let second_item = self.nth((second - first).checked_sub(1)?)?;

        let pair = if a < b {
            (first_item, second_item)
        } else {
            (second_item, first_item)
        };
        Some(pair)
    }
}

impl<T> Itertools for T where T: Iterator + ?Sized {}
