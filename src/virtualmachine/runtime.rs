use std::collections::HashMap;

use crate::{VMFunction, VMInstructionType, interpreter::{Value, builtin::*}, error::GenericError, VMValue, tokenizer::Token, VMInstruction, VMVariable};

pub struct Runtime<'file> {
    functions: HashMap<String, VMFunction<'file>>,
    stack: Vec<ExecutionFrame<'file>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateData {
    var: String,
    index: Option<usize>
}

impl UpdateData {
    pub fn variable(name: String) -> Self {
        Self {
            var: name,
            index: None
        }
    }
    pub fn indexed(name: String, index: usize) -> Self {
        Self {
            var: name,
            index: Some(index)
        }
    }
}

pub struct ExecutionFrame<'file> {
    pub variables: HashMap<String, Value>,
    pub function: VMFunction<'file>,
    pub line: usize,
    pub last_line: Option<usize>,
    pub last_updated: Vec<UpdateData>,
    pub last_read: Vec<UpdateData>,
    pub return_value: Option<Value>,
    pub passed_return: Option<Value>,
    pub last_lines: Vec<usize>
}

impl<'file> Runtime<'file> {
    pub fn load(functions: Vec<VMFunction<'file>>) -> Self {
        let mut hashmap = HashMap::new();

        for func in functions {
            let name = func.name.extract_text().to_string();
            hashmap.insert(name, func);
        }

        Self {
            functions: hashmap,
            stack: Vec::new()
        }
    }

    pub fn add_stack_frame(&mut self, function_name: VMVariable<'file>, arguments: Vec<Value>) -> Result<(), GenericError<'file>> {
        let name = function_name.extract_text().to_string();
        if let Some(f) = self.functions.get(&name) {
            let mut v = vec![];

            if let Some(last) = self.stack.last() {
                v = last.last_lines.clone();
                if let Some(inst) = last.next_instruction(){
                    v.push(inst.associated_line);
                }
            }

            self.stack.push(ExecutionFrame::new(f.clone(), arguments, v));
            Ok(())
        }
        else {
            Err(GenericError::tokenable_error(function_name.get_token(), format!("function '{}' not defined", function_name)))
        }
    }

    pub fn start_execution(&mut self, function_name: &str) -> Result<(), GenericError<'file>> {
        if let Some(f) = self.functions.get(function_name) {
            self.stack.push(ExecutionFrame::new(f.clone(), vec![], vec![]));
            Ok(())
        }
        else {
            Err(GenericError::tokenless_error(format!("function '{}' not defined", function_name)))
        }
    }

    pub fn single_step(&mut self) -> Result<bool, GenericError<'file>> {
        if let Some(last) = self.stack.last_mut() {
            let at_start = last.next_instruction().map(|i| i.associated_line);
            if last.return_value.is_some() {
                let value = self.stack.pop().unwrap().return_value.unwrap();
                if let Some(new_last) = self.stack.last_mut() {
                    new_last.passed_return = Some(value);
                }
                self.single_step()?;
                Ok(true)
            }
            else if let Some((name, args)) = last.single_step()? {
                self.add_stack_frame(name, args)?;
                Ok(true)
            }
            else {
                let at_end = last.next_instruction().map(|i| i.associated_line);
                Ok(at_start != at_end)
            }   
        }
        else {
            Ok(true)
        }
    }

    pub fn is_done(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn clear(&mut self) {
        if let Some(last) = self.stack.last_mut() {
            last.clear();
        }
    }
}

impl<'file> ExecutionFrame<'file> {
    pub fn new(function: VMFunction<'file>, arguments: Vec<Value>, last_lines: Vec<usize>) -> Self {
        let arg_names = function.arguments.clone();
        let line = function.name.location.line;
        let mut result = Self {
            function,
            variables: HashMap::new(),
            line: 0,
            last_line: Some(line),
            last_updated: Vec::new(),
            last_read: Vec::new(),
            return_value: None,
            passed_return: None,
            last_lines
        };

        for (name, arg) in arg_names.into_iter().zip(arguments.into_iter()) {
            result.variables.insert(name.extract_text().to_string(), arg);
        }

        result
    }

    pub fn clear(&mut self) {
        self.last_updated.clear();
        self.last_read.clear();
    }

    pub fn load_value(&mut self, value: VMValue<'file>, report: bool) -> Result<Value, GenericError<'file>> {
        match value {
            VMValue::MemberAccess(m, a) => {
                let t = a.get_token().unwrap();
                let m = self.load_value(*m, report)?;
                if report {
                    self.touch_variable(t.extract_text())?;
                }
                builtin_member_access(m, t.clone()).map_err(|e| e.finish(t))
            },
            VMValue::Indexing(m, i) => {
                let t = m.get_token().unwrap();
                let m = self.load_value(*m, false)?;
                let i = self.load_value(*i, report)?;

                if let Value::Number(n) = i {
                    if report {
                        self.read_variable_index(t.extract_text(), n as usize)?;
                    }
                }

                builtin_indexing(vec![m, i]).map_err(|e| e.finish(t))
            },
            VMValue::Value(v, _) => Ok(v),
            VMValue::Variable(v) => self.read_variable(v.extract_text(), v.get_token(), report),
        }
    }

    pub fn store_value_into(&mut self, value: VMValue<'file>, to_store: Value) -> Result<(), GenericError<'file>> {
        match value {
            VMValue::MemberAccess(m, i) => {
                self.touch_variable(m.get_token().unwrap().extract_text())?;
                let m = self.load_value(*m, false)?;
                let t = i.get_token().unwrap();


                builtin_mutable_member_access(m, t.clone(), to_store).map_err(|e| e.finish(t))
            },
            VMValue::Indexing(m, i) => {
                let t = m.get_token().unwrap();
                let m = self.load_value(*m, false)?;
                let i = self.load_value(*i, true)?;
                if let Value::Number(n) = i {
                    self.touch_variable_index(t.extract_text(), n as usize)?;
                }

                builtin_mutable_indexing(vec![m, i], to_store).map_err(|e| e.finish(t))
            },
            VMValue::Value(v, t) => Err(GenericError::tokenable_error(t, format!("unable to assign to immutable value '{}'", v))),
            VMValue::Variable(v) => {
                self.assign_to_variable(v.extract_text(), to_store)
            },
        }
    }

    pub fn assign_to_variable(&mut self, var_name: &str, value: Value) -> Result<(), GenericError<'file>> {
        self.variables.insert(var_name.to_string(), value);
        self.last_updated.push(UpdateData::variable(var_name.to_string()));
        Ok(())
    }

    pub fn touch_variable(&mut self, var_name: &str) -> Result<(), GenericError<'file>> {
        self.last_updated.push(UpdateData::variable(var_name.to_string()));
        Ok(())
    }

    pub fn touch_variable_index(&mut self, var_name: &str, index: usize) -> Result<(), GenericError<'file>> {
        self.last_updated.push(UpdateData::indexed(var_name.to_string(), index));
        Ok(())
    }

    pub fn read_variable_index(&mut self, var_name: &str, index: usize) -> Result<(), GenericError<'file>> {
        self.last_read.push(UpdateData::indexed(var_name.to_string(), index));
        Ok(())
    }

    pub fn read_variable(&mut self, var_name: &str, token: Option<Token<'file>>, report: bool) -> Result<Value, GenericError<'file>> {
        if let Some(v) = self.variables.get(var_name) {
            if report {
                self.last_read.push(UpdateData::variable(var_name.to_string()));
            }
            Ok(v.clone())
        }
        else {
            Err(GenericError::tokenable_error(token, format!("variable '{}' is not defined", var_name)))
        }
    }

    pub fn next_instruction(&self) -> Option<&VMInstruction<'file>> {
        self.function.instructions.get(self.line)
    }

    pub fn builtin_function_call(&mut self, function_name: VMVariable<'file>, arguments: Vec<Value>) -> Result<Option<Value>, GenericError<'file>> {
        let name = function_name.extract_text();

        if name == "Print" {
            Ok(Some(builtin_print(arguments).map_err(|e| e.finish_maybe(function_name.get_token()))?))
        }
        else if name == "Array" {
            Ok(Some(builtin_array(arguments).map_err(|e| e.finish_maybe(function_name.get_token()))?))
        }
        else if name == "AssertEqual" {
            Ok(Some(builtin_assert_eq(function_name.get_token(), arguments).map_err(|e| e.finish_maybe(function_name.get_token()))?))
        }
        else if name == "floor" {
            Ok(Some(builtin_floor(arguments).map_err(|e| e.finish_maybe(function_name.get_token()))?))
        }
        else if name == "ceil" {
            Ok(Some(builtin_ceil(arguments).map_err(|e| e.finish_maybe(function_name.get_token()))?))
        }
        else {
            Ok(None)
        }
    }

    pub fn single_step(&mut self) -> Result<Option<(VMVariable<'file>, Vec<Value>)>, GenericError<'file>> {
        let instruction = self.function.instructions[self.line].clone();
        println!("{}", instruction);

        match instruction.instruction_type {
            VMInstructionType::Assign(a, b) => {
                let v = self.load_value(b, true)?;
                self.store_value_into(a, v)?;
                self.line += 1;
            }
            VMInstructionType::BinaryOperation(op, dest, a, b) => {
                let a = self.load_value(a, true)?;
                let b = self.load_value(b, true)?;

                let to_store = match op {
                    crate::VMBinaryOperation::Add => builtin_add(vec![a, b]),
                    crate::VMBinaryOperation::Subtract => builtin_sub(vec![a, b]),
                    crate::VMBinaryOperation::Multiply => builtin_mul(vec![a, b]),
                    crate::VMBinaryOperation::Divide => builtin_div(vec![a, b]),
                    crate::VMBinaryOperation::LessThan => builtin_less_than(vec![a, b]),
                    crate::VMBinaryOperation::GreaterThan => builtin_greater_than(vec![a, b]),
                    crate::VMBinaryOperation::LessThanEqual => builtin_less_than_equal(vec![a, b]),
                    crate::VMBinaryOperation::GreaterThanEqual => builtin_greater_than_equal(vec![a, b]),
                    crate::VMBinaryOperation::Equality => builtin_equality(vec![a, b]),
                    crate::VMBinaryOperation::Inequality => builtin_inequality(vec![a, b]),
                } .map_err(|e| e.finish_no_token())?;

                self.store_value_into(dest.into(),to_store)?;
                self.line += 1;
            }
            VMInstructionType::Return(value) => {
                self.return_value = Some(self.load_value(value, true)?);
            }
            VMInstructionType::FunctionCall(function, dest, arguments) => {
                if let Some(returned_value) = self.passed_return.take() {
                    self.store_value_into(dest.into(), returned_value)?;
                    self.line += 1;
                }
                else {
                    let mut argument_values = Vec::new();

                    for arg in arguments {
                        argument_values.push(self.load_value(arg, true)?);
                    }

                    if let Some(v) = self.builtin_function_call(function.clone(), argument_values.clone())? {
                        self.store_value_into(dest.into(), v)?;
                        self.line += 1;
                    }
                    else {
                        return Ok(Some((function, argument_values)));
                    }
                }
            }
            VMInstructionType::Branch(cond, true_branch, false_branch) => {
                let cond = self.load_value(cond, true)?;
                
                if let Value::Boolean(b) = cond {
                    if b {
                        self.line = true_branch;
                    }
                    else {
                        self.line = false_branch;
                    }
                }
                else {
                    return Err(GenericError::tokenless_error(format!("value '{}' is not a boolean", cond)));
                }
            }
            VMInstructionType::Goto(branch) => {
                self.line = branch;
            }
        }

        self.last_line = Some(instruction.associated_line);

        Ok(None)
    }
}

impl<'file> std::fmt::Display for Runtime<'file> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(last) = self.stack.last() {
            write!(f, "{}", last)
        }
        else {
            write!(f, "Runtime not executing program")
        }
    }
} 