pub mod vcd_ast;

use vcd::{TimescaleUnit, Writer, IdCode, SimulationCommand};
use crate::sim::config::{VcdPath, FileNotFound};
use thiserror::Error;
use miette::Diagnostic;
use std::io::Write;
use std::collections::HashMap;
use crate::sim::instantiate::UniqueVariableRef;
use crate::sim::instantiated_ast as inst;
use crate::vcd::vcd_ast::VcdModule;
use crate::sim::value::Value;
use crate::time::{Instant, Duration};

#[derive(Debug, Error, Diagnostic)]
pub enum VcdError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    FileNotFound(#[from] FileNotFound),

    #[error(transparent)]
    #[diagnostic(transparent)]
    FileWriteError(#[from] FileWriteError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    VariableNotDeclared(#[from] VariableNotDeclared),
}

#[derive(Error, Debug, Diagnostic)]
#[error("couldn't write to vcd file: {:?}", _0)]
#[diagnostic()]
pub struct FileWriteError(std::io::Error);


#[derive(Error, Debug, Diagnostic)]
#[error("variable not declared when setting up vcd file (this is a bug): {:?}", _0)]
#[diagnostic()]
pub struct VariableNotDeclared(UniqueVariableRef);

pub struct VcdGenerator {
    writer: Writer<Box<dyn Write>>,

    variable_mapping: HashMap<UniqueVariableRef, IdCode>,
}

impl VcdGenerator {
    pub fn new(path: &VcdPath, timescale: TimescaleUnit, toplevel: VcdModule) -> Result<Self, VcdError> {
        let mut w = path.writer()?;
        let mut writer = Writer::new(w);

        writer.version("Generated by Hithlain").map_err(FileWriteError)?;
        // TODO: add proper date
        writer.date("17/12/2021").map_err(FileWriteError)?;
        writer.timescale(1, timescale).map_err(FileWriteError)?;

        let mut variable_mapping = HashMap::new();
        Self::write_modules(&mut writer, &mut variable_mapping, toplevel, true)?;

        writer.enddefinitions().map_err(FileWriteError)?;

        // writer.begin(SimulationCommand::Dumpvars).map_err(FileWriteError)?;
        // for i in variable_mapping.values() {
        //     writer.change_scalar(*i, vcd::Value::V0);
        // }
        // writer.end().map_err(FileWriteError)?;

        Ok(Self {
            writer,
            variable_mapping,
        })
    }


    fn write_modules(writer: &mut Writer<Box<dyn Write>>, variable_mapping: &mut HashMap<UniqueVariableRef, IdCode>, module: VcdModule, top: bool) -> Result<(), VcdError> {
        if top {
            writer.add_module("TOP").map_err(FileWriteError)?;
        } else {
            writer.add_module(module.name.0.as_str()).map_err(FileWriteError)?;
        }

        for i in module.variables {
            let wire = writer.add_wire(1, &i.original.variable.0).map_err(FileWriteError)?;

            variable_mapping.insert(i, wire);
        }

        for i in module.submodules {
            Self::write_modules(writer, variable_mapping, i, false)?;
        }

        writer.upscope().map_err(FileWriteError)?;

        Ok(())
    }

    pub fn update_wire(&mut self, variable: &UniqueVariableRef, value: Value, time: Instant) -> Result<(), VcdError> {
        match value {
            Value::Bit(b) => {
                println!("{:?}: set {:?} to {:?}", time, variable, value);

                let wire = if let Some(i) = self.variable_mapping.get(variable) {
                    i
                } else {
                    return Ok(())
                };

                self.writer.timestamp(time.nanos()).map_err(FileWriteError)?;
                self.writer.change_scalar(*wire, if b {
                    vcd::Value::V1
                } else {
                    vcd::Value::V0
                }).map_err(FileWriteError)?;
            }
        }

        Ok(())
    }

    pub fn finalize(&mut self, time: Instant, overshoot: Duration) -> Result<(), VcdError> {
        self.writer.timestamp(time.nanos() + overshoot.nanos()).map_err(FileWriteError)?;

        Ok(())
    }
}