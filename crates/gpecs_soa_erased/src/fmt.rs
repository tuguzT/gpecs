use core::fmt::{self, Debug, UpperHex};

pub struct SliceUpperHex<'a, A>(pub &'a [A]);

impl<A> Debug for SliceUpperHex<'_, A>
where
    A: UpperHex,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self([bytes @ .., byte]) = *self else {
            return Ok(());
        };

        for byte in bytes {
            write!(f, "{byte:#X} ")?;
        }
        write!(f, "{byte:#X}")
    }
}
