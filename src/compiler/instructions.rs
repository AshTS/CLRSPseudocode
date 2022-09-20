use crate::{tokenizer::Token, interpreter::Value, parser::ExpressionType, error::GenericError};

#[derive(Debug, Clone)]
pub struct VMInstruction<'file> {
    pub associated_line: usize,
    pub instruction_type: VMInstructionType<'file>
}

#[derive(Debug, Clone)]
pub enum VMValue<'file> {
    MemberAccess(Box<VMValue<'file>>, Box<VMValue<'file>>),
    Indexing(Box<VMValue<'file>>, Box<VMValue<'file>>),
    Value(Value, Option<Token<'file>>),
    Variable(VMVariable<'file>)
}

#[derive(Debug, Clone)]
pub enum VMVariable<'file> {
    Token(Token<'file>),
    Custom(String)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VMBinaryOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    Equality,
    Inequality,
}

#[derive(Debug, Clone)]
pub enum VMInstructionType<'file> {
    Return(VMValue<'file>),
    Assign(VMValue<'file>, VMValue<'file>),
    BinaryOperation(VMBinaryOperation, VMVariable<'file>, VMValue<'file>, VMValue<'file>),
    FunctionCall(VMVariable<'file>, VMVariable<'file>, Vec<VMValue<'file>>),
    Branch(VMValue<'file>, usize, usize),
    Goto(usize),
}

#[derive(Debug, Clone)]
pub struct VMFunction<'file> {
    pub instructions: Vec<VMInstruction<'file>>,
    pub name: Token<'file>,
    pub arguments: Vec<Token<'file>>,
    next_name: usize,
    pub raw_file: Option<&'file str>
}

impl<'file> std::fmt::Display for VMInstruction<'file> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:<3}]  {}", self.associated_line, self.instruction_type)
    }
}

impl<'file> VMInstruction<'file> {
    pub fn new(associated_line: usize, instruction_type: VMInstructionType<'file>) -> Self {
        Self {
            associated_line,
            instruction_type
        }
    }
}

impl<'file> std::convert::From<Value> for VMValue<'file> {
    fn from(v: Value) -> Self {
        VMValue::Value(v, None)
    }
}

impl<'file> std::convert::From<(Value, Token<'file>)> for VMValue<'file> {
    fn from(v: (Value, Token<'file>)) -> Self {
        VMValue::Value(v.0, Some(v.1))
    }
}

impl<'file> std::convert::From<(Value, Option<Token<'file>>)> for VMValue<'file> {
    fn from(v: (Value, Option<Token<'file>>)) -> Self {
        VMValue::Value(v.0, v.1)
    }
}

impl<'file, T: Into<VMVariable<'file>>> std::convert::From<T> for VMValue<'file> {
    fn from(v: T) -> Self {
        VMValue::Variable(v.into())
    }
}

impl<'file> std::fmt::Display for VMValue<'file> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMValue::Indexing(m, i) => write!(f, "{}[{}]", m, i),
            VMValue::MemberAccess(m, i) => write!(f, "{}.{}", m, i),
            VMValue::Value(v, _) => write!(f, "{}", v),
            VMValue::Variable(v) => write!(f, "{}", v),
        }
    }
}

impl<'file> VMValue<'file> {
    pub fn get_token(&self) -> Option<Token<'file>> {
        match self {
            VMValue::MemberAccess(v, _) => v.get_token(),
            VMValue::Indexing(v, _) => v.get_token(),
            VMValue::Value(_, t) => t.as_ref().cloned(),
            VMValue::Variable(v) => v.get_token(),
        }
    }
}

impl<'file> std::fmt::Display for VMVariable<'file> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.extract_text())
    }
}

impl<'file> std::convert::From<String> for VMVariable<'file> {
    fn from(s: String) -> Self {
        VMVariable::Custom(s)
    }
}

impl<'file> std::convert::From<Token<'file>> for VMVariable<'file> {
    fn from(t: Token<'file>) -> Self {
        VMVariable::Token(t)
    }
}

impl<'file> VMVariable<'file> {
    pub fn extract_text(&self) -> &str {
        match self {
            VMVariable::Token(t) => t.extract_text(),
            VMVariable::Custom(s) => s
        }
    }

    pub fn get_token(&self) -> Option<Token<'file>> {
        match self {
            VMVariable::Token(t) => Some(t.clone()),
            VMVariable::Custom(_) => None,
        }
    }
}

impl std::fmt::Display for VMBinaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMBinaryOperation::Add => write!(f, "add"),
            VMBinaryOperation::Subtract => write!(f, "sub"),
            VMBinaryOperation::Multiply => write!(f, "mul"),
            VMBinaryOperation::Divide => write!(f, "div"),
            VMBinaryOperation::LessThan => write!(f, "lt"),
            VMBinaryOperation::GreaterThan => write!(f, "gt"),
            VMBinaryOperation::LessThanEqual => write!(f, "lte"),
            VMBinaryOperation::GreaterThanEqual => write!(f, "gte"),
            VMBinaryOperation::Equality => write!(f, "equal"),
            VMBinaryOperation::Inequality => write!(f, "nequal"),
        }
    }
}

impl std::convert::TryFrom<ExpressionType> for VMBinaryOperation {
    type Error = ();

    fn try_from(value: ExpressionType) -> Result<Self, Self::Error> {
        match value {
            ExpressionType::Add => Ok(VMBinaryOperation::Add),
            ExpressionType::Subtract => Ok(VMBinaryOperation::Subtract),
            ExpressionType::Multiply => Ok(VMBinaryOperation::Multiply),
            ExpressionType::Divide => Ok(VMBinaryOperation::Divide),
            ExpressionType::LessThan => Ok(VMBinaryOperation::LessThan),
            ExpressionType::GreaterThan => Ok(VMBinaryOperation::GreaterThan),
            ExpressionType::LessThanEqual => Ok(VMBinaryOperation::LessThanEqual),
            ExpressionType::GreaterThanEqual => Ok(VMBinaryOperation::GreaterThanEqual),
            ExpressionType::Equality => Ok(VMBinaryOperation::Equality),
            ExpressionType::Inequality => Ok(VMBinaryOperation::Inequality),
            _ => Err(())
        }
    }
}

impl<'file> std::convert::TryFrom<VMValue<'file>> for VMVariable<'file> {
    type Error = GenericError<'file>;

    fn try_from(value: VMValue<'file>) -> Result<Self, Self::Error> {
        match value {
            VMValue::Value(_, Some(t)) => Err(GenericError::error(t, "expected identifier or variable".to_string())),
            VMValue::Variable(v) => Ok(v),
            _ => todo!()
        }
    }
}

fn render_instruction(f: &mut std::fmt::Formatter<'_>, opcode: &str, arguments: &[String]) -> std::fmt::Result {
    write!(f, "{:10} ", opcode)?;

    for (i, arg) in arguments.iter().enumerate() {
        if i != 0 {
            write!(f, ", ")?;
        }
        write!(f, "{}", arg)?;
    }

    Ok(())
}

impl<'file> std::fmt::Display for VMInstructionType<'file> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMInstructionType::Return(arg) => render_instruction(f, "return", &[arg.to_string()]),
            VMInstructionType::Assign(dest, src) => render_instruction(f, "assign", &[dest.to_string(), src.to_string()]),
            VMInstructionType::BinaryOperation(op, dest, a, b) => render_instruction(f, &op.to_string(), &[dest.to_string(), a.to_string(), b.to_string()]),
            VMInstructionType::FunctionCall(name, result, args) => {
                let mut arg_values = vec![name.to_string(), result.to_string()];
                for arg in args {
                    arg_values.push(arg.to_string());
                }

                render_instruction(f, "call", &arg_values)
            },
            VMInstructionType::Branch(condition, true_branch, false_branch) => render_instruction(f, "branch", &[condition.to_string(), true_branch.to_string(), false_branch.to_string()]),
            VMInstructionType::Goto(inst) => render_instruction(f, "goto", &[inst.to_string()])
        }
    }
}

impl<'file> std::fmt::Display for VMFunction<'file> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.name.extract_text())?;

        for (i, arg) in self.arguments.iter().enumerate() {
            write!(f, "{}{}", if i != 0 { ", " } else { "" }, arg.extract_text())?;
        }

        writeln!(f, ")")?;
        for (i, line) in self.instructions.iter().enumerate() {
            writeln!(f, "  {:<3} {}", i, line)?;
        }

        Ok(())
    }
}

impl<'file> VMFunction<'file> {
    pub fn new(name: Token<'file>, arguments: Vec<Token<'file>>) -> Self {
        let file_data = name.location.file_text;
        Self {
            instructions: Vec::new(),
            arguments,
            name,
            next_name: 0,
            raw_file: file_data
        }
    }

    pub fn add_instruction(&mut self, instruction: VMInstruction<'file>) {
        self.instructions.push(instruction);
    }

    pub fn add_instruction_type(&mut self, associated_line: usize, instruction_type: VMInstructionType<'file>) {
        self.add_instruction(VMInstruction::new(associated_line, instruction_type));
    }

    pub fn next_temp_variable(&mut self) -> VMVariable<'file> {
        self.next_name += 1;
        format!("temp${}", self.next_name - 1).into()
    }

    pub fn next_instruction_index(&self) -> usize {
        self.instructions.len()
    }
}