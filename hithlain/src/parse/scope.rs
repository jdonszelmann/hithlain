use crate::parse::ast::Variable;
use derivative::Derivative;
use miette::{Diagnostic, NamedSource, SourceSpan};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("duplicate definition of variable {}", variable.0)]
#[diagnostic()]
pub struct DuplicateDefinition {
    #[source_code]
    src: NamedSource,

    variable: Variable,

    #[label("first definition")]
    prev: SourceSpan,

    #[label("second definition")]
    def: SourceSpan,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum VariableType {
    In,
    Out,
    Temp,
    Intermediate,
}

#[derive(Derivative)]
#[derivative(Hash, PartialEq)]
#[derive(Debug)]
pub struct InnerVariableRef {
    pub(crate) variable: Variable,
    pub(crate) variable_type: VariableType,

    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    pub(crate) read: AtomicBool,
    #[derivative(PartialEq = "ignore")]
    #[derivative(Hash = "ignore")]
    pub(crate) written: AtomicBool,
}
impl Eq for InnerVariableRef {}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct VariableRef(pub(crate) Rc<InnerVariableRef>);

pub struct Scope {
    pub(crate) variables: HashMap<Variable, VariableRef>,

    temps: usize,
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    #[must_use]
    pub fn new() -> Self {
        Self {
            variables: HashMap::default(),
            temps: 0,
        }
    }

    pub fn lookup_variable_read(
        &mut self,
        variable: &Variable,
    ) -> Result<VariableRef, DuplicateDefinition> {
        let vr = if let Some(i) = self.variables.get(variable) {
            i.clone()
        } else {
            self.define_variable(variable, VariableType::Intermediate)?
        };

        vr.0.read.store(true, Ordering::SeqCst);
        Ok(vr)
    }

    pub fn lookup_variable_write(
        &mut self,
        variable: &Variable,
    ) -> Result<VariableRef, DuplicateDefinition> {
        let vr = if let Some(i) = self.variables.get(variable) {
            i.clone()
        } else {
            self.define_variable(variable, VariableType::Intermediate)?
        };

        vr.0.written.store(true, Ordering::SeqCst);
        Ok(vr)
    }

    pub fn define_temp_variable(&mut self) -> Result<VariableRef, DuplicateDefinition> {
        let variable = Variable(format!("tmp_{}", self.temps), None);
        self.temps += 1;

        self.define_variable(&variable, VariableType::Temp)
    }

    pub fn define_variable(
        &mut self,
        variable: &Variable,
        variable_type: VariableType,
    ) -> Result<VariableRef, DuplicateDefinition> {
        if let Some(i) = self.variables.get(variable) {
            let span_1 = variable.1.clone().expect("must have span");
            let span_2 = i.0.variable.1.clone().expect("must have span");

            return Err(DuplicateDefinition {
                src: span_1.source().clone().into(),
                variable: variable.clone(),
                prev: span_2.into(),
                def: span_1.into(),
            });
        }

        let vr = VariableRef(Rc::new(InnerVariableRef {
            variable: variable.clone(),
            variable_type,
            read: AtomicBool::new(false),
            written: AtomicBool::new(false),
        }));
        self.variables.insert(variable.clone(), vr.clone());

        Ok(vr)
    }
}
