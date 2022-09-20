use std::{rc::Rc, cell::RefCell};

use crate::{tokenizer::Token, parser::ParseTreeNode, error::GenericError};

use super::{Value, Executor, RunTime};

#[derive(Debug, Clone)]
pub struct Function<'file> {
    pub name: String,
    _name_token: Token<'file>,
    pub arguments: Vec<Token<'file>>,
    block: ParseTreeNode<'file>
}

impl<'file> Function<'file> {
    pub fn new(node: ParseTreeNode<'file>) -> Self {
        if let ParseTreeNode::Function { name, arguments, block } = node {
            Self {
                name: name.extract_text().to_string(),
                _name_token: name,
                arguments,
                block: *block
            }
        }
        else {
            unimplemented!()
        }
    }

    pub fn execute(&self, arguments: Vec<Value>, runtime: Rc<RefCell<RunTime<'file>>>) -> Result<Value, GenericError<'file>> {
        let mut executor = Executor::new(runtime);

        for (arg, name) in arguments.iter().zip(self.arguments.iter()) {
            executor.set_variable(name.extract_text().to_string(), arg.clone());
        }

        self.block.execute(&mut executor).map(|v| v.0)
    }
}