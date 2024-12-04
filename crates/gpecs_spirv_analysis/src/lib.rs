//! Nothing too special, too =)

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_code)]
// TODO `#![no_std]` with `alloc` enabled

use std::{
    error::Error,
    fmt::{self, Display},
};

use rspirv::{binary::Disassemble, dr::Module};

#[derive(Debug, Clone, Copy)]
pub struct WorkError;

impl Display for WorkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Some error during work calculation")
    }
}

impl Error for WorkError {}

pub trait Work {
    fn work(&self) -> Result<(), WorkError>;
}

impl Work for Module {
    fn work(&self) -> Result<(), WorkError> {
        println!("{}", self.disassemble());
        Ok(())
    }
}
