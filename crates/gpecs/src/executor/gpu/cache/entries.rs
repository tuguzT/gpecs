use wgpu::BindGroupEntry;

pub trait AdditionalEntries {
    type Output<'a>: AsRef<[BindGroupEntry<'a>]>
    where
        Self: 'a;

    fn additional_entries(&self) -> Self::Output<'_>;
}

impl<T> AdditionalEntries for &T
where
    T: AdditionalEntries + ?Sized,
{
    type Output<'a>
        = T::Output<'a>
    where
        Self: 'a;

    #[inline]
    fn additional_entries(&self) -> Self::Output<'_> {
        (**self).additional_entries()
    }
}

impl<T> AdditionalEntries for &mut T
where
    T: AdditionalEntries + ?Sized,
{
    type Output<'a>
        = T::Output<'a>
    where
        Self: 'a;

    #[inline]
    fn additional_entries(&self) -> Self::Output<'_> {
        (**self).additional_entries()
    }
}

impl AdditionalEntries for [BindGroupEntry<'_>] {
    type Output<'a>
        = &'a [BindGroupEntry<'a>]
    where
        Self: 'a;

    #[inline]
    fn additional_entries(&self) -> Self::Output<'_> {
        self
    }
}

impl<const N: usize> AdditionalEntries for [BindGroupEntry<'_>; N] {
    type Output<'a>
        = &'a [BindGroupEntry<'a>]
    where
        Self: 'a;

    #[inline]
    fn additional_entries(&self) -> Self::Output<'_> {
        self
    }
}

impl AdditionalEntries for () {
    type Output<'a>
        = [BindGroupEntry<'a>; 0]
    where
        Self: 'a;

    #[inline]
    fn additional_entries(&self) -> Self::Output<'_> {
        []
    }
}
