use core::fmt::{self, Debug};

#[derive(Clone, Copy)]
pub struct DebugBytesUpperHex<'a>(pub &'a [u8]);

impl Debug for DebugBytesUpperHex<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self([bytes @ .., byte]) = *self else {
            return Ok(());
        };

        for byte in bytes {
            write!(f, "{byte:#X?} ")?;
        }
        write!(f, "{byte:#X?}")
    }
}
