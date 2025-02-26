//! Nothing too special, too =)

#![warn(clippy::all)]
// TODO `#![warn(missing_docs)]` after implementation & tests
#![forbid(unsafe_code)]

use std::{
    error::Error,
    fmt::{self, Display},
};

use rspirv::{
    binary::Disassemble,
    dr::{Block, Function, Module},
};

pub mod tmm;

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
        for function in self.functions.as_slice() {
            function.work()?;
        }

        Ok(())
    }
}

impl Work for Function {
    fn work(&self) -> Result<(), WorkError> {
        println!("Function definition:\n{:#?}", self.def);
        for block in self.blocks.as_slice() {
            block.work()?;
        }

        Ok(())
    }
}

impl Work for Block {
    fn work(&self) -> Result<(), WorkError> {
        println!("Block label:\n{:#?}", self.label);
        println!("Block assembler:\n{}", self.disassemble());

        Ok(())
    }
}
