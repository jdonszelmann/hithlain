#![allow(dead_code)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::missing_errors_doc)]

pub mod error;
pub mod parse;
pub mod sim;
pub mod time;
pub mod vcd;

pub use miette;

#[cfg(test)]
pub mod fuzz;

// TODO: vhdl translation
// mod vhdl;
// TODO: verilog translation
// mod verilog;
