use core::iter::FusedIterator;

use gpecs_soa_erased::{
    CovariantFieldDescriptors,
    soa::field::{FieldDescriptor, FieldDescriptors},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GpuFieldDescriptors<T>
where
    T: ?Sized,
{
    index: usize,
    descriptors: T,
}

impl<T> From<T> for GpuFieldDescriptors<T> {
    fn from(descriptors: T) -> Self {
        Self {
            index: 0,
            descriptors,
        }
    }
}

impl<'a, T> FieldDescriptors<'a> for GpuFieldDescriptors<T>
where
    T: AsRef<[FieldDescriptor]> + Clone,
{
    type Output = Self;

    fn field_descriptors(&'a self) -> Self::Output {
        self.clone()
    }
}

impl<T> CovariantFieldDescriptors for GpuFieldDescriptors<T>
where
    T: AsRef<[FieldDescriptor]> + Clone,
{
    fn upcast_field_descriptors<'short, 'long: 'short>(
        from: <Self as FieldDescriptors<'long>>::Output,
    ) -> <Self as FieldDescriptors<'short>>::Output {
        from
    }
}

impl<T> Iterator for GpuFieldDescriptors<T>
where
    T: AsRef<[FieldDescriptor]> + ?Sized,
{
    type Item = FieldDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            ref descriptors,
            ref mut index,
        } = *self;

        let descriptors = descriptors.as_ref();
        if descriptors.len() <= *index {
            return None;
        }

        let item = descriptors[*index];
        *index += 1;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }
}

impl<T> ExactSizeIterator for GpuFieldDescriptors<T>
where
    T: AsRef<[FieldDescriptor]> + ?Sized,
{
    fn len(&self) -> usize {
        let Self {
            ref descriptors,
            index,
        } = *self;

        let descriptors = descriptors.as_ref();
        descriptors.len() - index
    }
}

impl<T> FusedIterator for GpuFieldDescriptors<T> where T: AsRef<[FieldDescriptor]> + ?Sized {}
