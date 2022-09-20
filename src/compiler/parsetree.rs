use std::convert::TryInto;

use crate::{parser::{ParseTreeNode, ExpressionType}, VMFunction, error::GenericError, VMInstructionType, VMValue, interpreter::Value, VMVariable, VMBinaryOperation};

pub fn compile_function(parsetree: ParseTreeNode<'_>) -> Result<VMFunction<'_>, GenericError<'_>> {
    if let ParseTreeNode::Function { name, arguments, block } = parsetree {
        let l = name.location.line;
        let mut result = VMFunction::new(name, arguments);

        result.compile(&block)?;

        result.add_instruction_type(l, VMInstructionType::Return(Value::None.into()));

        Ok(result)
    }
    else {
        unimplemented!()
    }
}

impl<'file> VMFunction<'file> {
    pub fn compile(&mut self, parsetree: &ParseTreeNode<'file>) -> Result<Option<VMValue<'file>>, GenericError<'file>> {
        match parsetree {
            ParseTreeNode::Block { statements } => {
                for statement in statements {
                    self.compile(statement)?;
                }

                Ok(None)
            }
            ParseTreeNode::NumericValue { token, value } => {
                Ok(Some((Value::Number(*value), token.clone()).into()))
            }
            ParseTreeNode::IdentifierValue { token } => {
                Ok(Some(token.clone().into()))
            }
            ParseTreeNode::ReturnStatement { token, expression } => {
                if let Some(expr) = expression {
                    let child = self.compile(expr)?.unwrap();
                    self.add_instruction_type(token.location.line, VMInstructionType::Return(child));
                }
                else {
                    self.add_instruction_type(token.location.line, VMInstructionType::Return(Value::None.into()))
                }

                Ok(None)
            }
            ParseTreeNode::Expression { expression_type: ExpressionType::Assignment, symbols, children } => {
                let child_a = self.compile(&children[0])?;
                let child_b = self.compile(&children[1])?;

                if let Some(variable_a) = child_a {
                    if let Some(variable_b) = child_b.clone() {
                        self.add_instruction_type(symbols[0].location.line, 
                            VMInstructionType::Assign(variable_a, variable_b));
                    }
                    else {
                        println!("{:?}", child_b);
                        panic!()
                    }
                }
                else {
                    println!("{:?}", child_a);
                    panic!()
                }

                Ok(child_b)
            }
            ParseTreeNode::Expression { expression_type: ExpressionType::FunctionCall, symbols, children } => {
                let values = children.iter().map(|c| self.compile(c)).collect::<Result<Vec<_>, _>>()?;
                
                let func_name: VMVariable<'file> = values[0].clone().unwrap().try_into()?;

                let args = values[1..].iter().map(|c| c.clone().unwrap()).collect();
                let v = self.next_temp_variable();

                self.add_instruction_type(symbols[0].location.line, VMInstructionType::FunctionCall(func_name, v.clone(), args));

                Ok(Some(v.into()))
            }
            ParseTreeNode::Expression { expression_type: ExpressionType::MemberAccess, children, .. } => {
                let value = self.compile(&children[0])?.unwrap();
                let key = self.compile(&children[1])?.unwrap();

                Ok(Some(VMValue::MemberAccess(Box::new(value), Box::new(key))))
            }
            ParseTreeNode::Expression { expression_type: ExpressionType::Indexing, children, .. } => {
                let value = self.compile(&children[0])?.unwrap();
                let key = self.compile(&children[1])?.unwrap();

                Ok(Some(VMValue::Indexing(Box::new(value), Box::new(key))))
            }
            ParseTreeNode::Expression { expression_type: ExpressionType::LogicalAnd, symbols, children } => {
                let v = self.next_temp_variable();
                let a = self.compile(&children[0])?.unwrap();

                let first_compare = self.next_instruction_index();
                self.add_instruction_type(symbols[0].location.line, VMInstructionType::Branch(a.clone(), first_compare + 1, 0));

                let b = self.compile(&children[1])?.unwrap();

                self.add_instruction_type(symbols[0].location.line, VMInstructionType::Assign(v.clone().into(), b));

                let goto_end = self.next_instruction_index();
                self.add_instruction_type(symbols[0].location.line, VMInstructionType::Goto(0));

                let rejoin = self.next_instruction_index();
                self.add_instruction_type(symbols[0].location.line, VMInstructionType::Assign(v.clone().into(), a));

                let after = self.next_instruction_index();
                if let VMInstructionType::Goto(inst) = &mut self.instructions[goto_end].instruction_type {
                    *inst = after;
                } else {unimplemented!()}

                if let VMInstructionType::Branch(_, _, inst) = &mut self.instructions[first_compare].instruction_type {
                    *inst = rejoin;
                } else {unimplemented!()}

                Ok(Some(v.into()))
            }
            ParseTreeNode::Expression { expression_type: ExpressionType::LogicalOr, symbols, children } => {
                let v = self.next_temp_variable();
                let a = self.compile(&children[0])?.unwrap();

                let first_compare = self.next_instruction_index();
                self.add_instruction_type(symbols[0].location.line, VMInstructionType::Branch(a.clone(), 0, first_compare + 1));

                let b = self.compile(&children[1])?.unwrap();

                self.add_instruction_type(symbols[0].location.line, VMInstructionType::Assign(v.clone().into(), b));

                let goto_end = self.next_instruction_index();
                self.add_instruction_type(symbols[0].location.line, VMInstructionType::Goto(0));

                let rejoin = self.next_instruction_index();
                self.add_instruction_type(symbols[0].location.line, VMInstructionType::Assign(v.clone().into(), a));

                let after = self.next_instruction_index();
                if let VMInstructionType::Goto(inst) = &mut self.instructions[goto_end].instruction_type {
                    *inst = after;
                } else {unimplemented!()}

                if let VMInstructionType::Branch(_, inst, _) = &mut self.instructions[first_compare].instruction_type {
                    *inst = rejoin;
                } else {unimplemented!()}

                Ok(Some(v.into()))
            }
            ParseTreeNode::Expression { expression_type, symbols, children } => 
            {
                let values = children.iter().map(|c| self.compile(c)).collect::<Result<Vec<_>, _>>()?;

                if let Ok(bin_op) = (*expression_type).try_into() {
                    let a = values[0].as_ref().unwrap().clone();
                    let b = values[1].as_ref().unwrap().clone();

                    let v = self.next_temp_variable();
                    self.add_instruction_type(symbols[0].location.line, VMInstructionType::BinaryOperation(bin_op, v.clone(), a, b));
                    
                    Ok(Some(v.into()))
                }
                else {
                    dbg!(expression_type);
                    todo!()
                }
            }
            ParseTreeNode::IfStatement { ifs, else_block } => {
                let mut skip_to_end: Vec<usize> = Vec::new();

                for (token, cond, block) in ifs {
                    let cond = self.compile(cond)?.unwrap();
                    let prev = self.next_instruction_index();

                    self.add_instruction_type(token.location.line, VMInstructionType::Branch(cond, prev + 1, 0));

                    self.compile(block)?;

                    skip_to_end.push(self.next_instruction_index());

                    self.add_instruction_type(token.location.line, VMInstructionType::Goto(0));

                    let next = self.next_instruction_index();
                    if let VMInstructionType::Branch(_, _, branch) = &mut self.instructions[prev].instruction_type {
                        *branch = next;
                    }
                    else {
                        unimplemented!()
                    }
                }

                if let Some(else_block) = else_block {
                    self.compile(else_block)?;
                }

                let last = self.next_instruction_index();

                for i in skip_to_end {
                    if let VMInstructionType::Goto(v) = &mut self.instructions[i].instruction_type {
                        *v = last;
                    }
                    else {
                        unimplemented!()
                    }
                }

                Ok(None)
            }
            ParseTreeNode::ForLoop { token, loop_variable, bound0, bound1, reverse, block } => {
                let b0 = self.compile(bound0)?.unwrap();
                let b1 = self.compile(bound1)?.unwrap();

                let line = token.location.line;
                let loop_variable: VMVariable<'file> = loop_variable.clone().into();

                let direction = if *reverse { VMBinaryOperation::Subtract } else { VMBinaryOperation::Add };
                let comparison = if *reverse { VMBinaryOperation::GreaterThanEqual } else { VMBinaryOperation::LessThanEqual };

                self.add_instruction_type(line, VMInstructionType::Assign(loop_variable.clone().into(), b0));

                let start = self.next_instruction_index();
                let v = self.next_temp_variable();
                self.add_instruction_type(line, VMInstructionType::BinaryOperation(comparison, v.clone(), loop_variable.clone().into(), b1));
                let compare_line = self.next_instruction_index();
                self.next_instruction_index();self.add_instruction_type(line, VMInstructionType::Branch(v.into(), compare_line + 1, 0));

                self.compile(block)?;

                self.add_instruction_type(line, VMInstructionType::BinaryOperation(direction, loop_variable.clone(), loop_variable.clone().into(), Value::Number(1.0).into()));
                self.add_instruction_type(line, VMInstructionType::Goto(start));

                let after = self.next_instruction_index();
                if let VMInstructionType::Branch(_, _, inst) = &mut self.instructions[compare_line].instruction_type {
                    *inst = after;
                } else {unimplemented!()}

                Ok(None)
            }
            ParseTreeNode::WhileLoop { token, condition, block } => {
                let line = token.location.line;
                let start = self.next_instruction_index();

                let c = self.compile(condition)?.unwrap();
                let compare_line = self.next_instruction_index();
                self.add_instruction_type(line, VMInstructionType::Branch(c, compare_line + 1, 0));

                self.compile(block)?;

                self.add_instruction_type(line, VMInstructionType::Goto(start));
                let after = self.next_instruction_index();
                if let VMInstructionType::Branch(_, _, inst) = &mut self.instructions[compare_line].instruction_type {
                    *inst = after;
                } else {unimplemented!()}

                Ok(None)
            }
            _ => {
                dbg!(parsetree);
                todo!()
            }
        }
    }
}