use std::{collections::HashMap, cell::RefCell, rc::Rc};

use crate::{error::GenericError, tokenizer::Token};

use super::{Value, RunTime};

#[derive(Debug)]
pub struct Executor<'file> {
    pub variables: HashMap<String, Value>,
    context:  Rc<RefCell<RunTime<'file>>>
}

impl<'file> Executor<'file> {
    pub fn new(context: Rc<RefCell<RunTime<'file>>>) -> Self {
        Self {
            variables: HashMap::new(),
            context
        }
    }

    pub fn get_variable(&self, name: &Token<'file>) -> Result<Value, GenericError<'file>> {
        if let Some(value) = self.variables.get(name.extract_text() as &str) {
            Ok(value.clone())
        }
        else {
            let t = name.extract_text().to_string();
            Err(GenericError::error(name.clone(), format!("variable '{}' does not exist", t)))
        }
    }

    pub fn get_mut_variable(&mut self, name: &Token<'file>) -> Result<&mut Value, GenericError<'file>> {
        if !self.variables.contains_key(name.extract_text() as &str) {
            self.variables.insert(name.extract_text().to_string(), Value::None);
        }

        if let Some(value) = self.variables.get_mut(name.extract_text() as &str) {
            Ok(value)
        }
        else {
            unimplemented!()
        }
    }

    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn execute_function(&mut self, func_name: Token<'file>, arguments: Vec<Value>) -> Result<Value, GenericError<'file>> {
        RunTime::execute_function(self.context.clone(), &func_name, arguments).map_err(|e| e.finish(func_name))
    }
}
