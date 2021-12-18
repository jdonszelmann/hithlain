pub mod sim;
pub mod parse;
pub mod error;
pub mod time;
pub mod vcd;

pub use miette;

#[cfg(test)]
pub mod fuzz;

// TODO: vhdl translation
// mod vhdl;
// TODO: verilog translation
// mod verilog;



