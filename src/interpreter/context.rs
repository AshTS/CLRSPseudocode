use std::{collections::HashMap, rc::Rc, cell::RefCell};

use crate::{parser::ParseTreeNode, tokenizer::Token, error::GenericError};

use super::{Function, Value, RuntimeError};


#[derive(Debug, Clone)]
pub struct RunTime<'file> {
    functions: HashMap<String, Function<'file>>
}

impl<'file> RunTime<'file> {
    pub fn new(parsed_functions: Vec<ParseTreeNode<'file>>) -> Self {
        let mut functions = HashMap::new();

        for func in parsed_functions {
            let func = Function::new(func);

            functions.insert(func.name.clone(), func);
        }

        Self {
            functions
        }
    }

    pub fn execute_function(runtime: Rc<RefCell<Self>>, func_name: &Token<'file>, arguments: Vec<Value>) -> Result<Value, RuntimeError<'file>> {
        let name = func_name.extract_text();

        if name == "AssertEqual" {
            return super::builtin_assert_eq(Some(func_name.clone()), arguments);
        }
        else if name == "Array" {
            return super::builtin_array(arguments);
        }
        else if name == "Print" {
            return super::builtin_print(arguments);
        }
        else if name == "ceil" {
            return super::builtin_ceil(arguments);
        }
        else if name == "floor" {
            return super::builtin_floor(arguments);
        }

        if let Some(v) = Self::inner_execute_function(runtime, name.to_string(), arguments)? {
            Ok(v)
        }
        else {
            Err(GenericError::error(func_name.clone(), format!("function '{}' not defined", name)).into())
        }
    }

    pub fn inner_execute_function(runtime: Rc<RefCell<Self>>, func_name: String, arguments: Vec<Value>) -> Result<Option<Value>, RuntimeError<'file>> {
        #[allow(clippy::manual_map)]
        if let Some(func) = runtime.borrow().functions.get(&func_name) {
            if arguments.len() != func.arguments.len() {
                return Err(RuntimeError::ArgumentCountError { expected: func.arguments.len(), got: arguments.len() });
            }

            Ok(Some(func.clone().execute(arguments, runtime.clone())?))
        } 
        else {
            Ok(None)
        }
    }
}