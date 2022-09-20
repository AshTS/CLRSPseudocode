use crate::{parser::{ParseTreeNode, ExpressionType}, error::GenericError};

use super::{Value, Executor, builtin::*};

impl<'file> ParseTreeNode<'file> {
    pub fn execute<'a>(&self, executor: &'a mut Executor<'file>) -> Result<(Value, bool), GenericError<'file>> {
        match self {
            Self::Block { statements } => {
                let mut last = (Value::None, false);
                for statement in statements {
                    last = statement.execute(executor)?;

                    if last.1 {
                        break;
                    }
                }

                Ok(last)
            },
            Self::Expression { expression_type, symbols, children } => {
                match expression_type {
                    ExpressionType::FunctionCall => {
                        let f = &children[0];
                        let args_iter = children[1..].iter().map(|c| c.execute(executor));
                        let args = args_iter.collect::<Result<Vec<_>, GenericError<'file>>>()?.iter().map(|v| v.0.clone()).collect();

                        if let ParseTreeNode::IdentifierValue { token } = f {
                            Ok((executor.execute_function(token.clone(), args)?, false))
                        }
                        else {
                            Err(GenericError::error(f.get_token().clone(), "unable to execute non-function value".to_string()).arrow("unable to execute non-function value".to_string()))
                        }
                    },
                    ExpressionType::Assignment => {
                        let value = children[1].execute(executor)?.0;
                        children[0].execute_mutable(executor, value.clone())?;

                        Ok((value, false))
                    }
                    ExpressionType::Add => {
                        let args_iter = children.iter().map(|c| c.execute(executor));
                        let args = args_iter.collect::<Result<Vec<_>, GenericError<'file>>>()?.iter().map(|v| v.0.clone()).collect();

                        builtin_add(args).map_err(|e| e.finish(symbols[0].clone())).map(|v| (v, false))
                    }
                    ExpressionType::Subtract => {
                        let args_iter = children.iter().map(|c| c.execute(executor));
                        let args = args_iter.collect::<Result<Vec<_>, GenericError<'file>>>()?.iter().map(|v| v.0.clone()).collect();

                        builtin_sub(args).map_err(|e| e.finish(symbols[0].clone())).map(|v| (v, false))
                    }
                    ExpressionType::Multiply => {
                        let args_iter = children.iter().map(|c| c.execute(executor));
                        let args = args_iter.collect::<Result<Vec<_>, GenericError<'file>>>()?.iter().map(|v| v.0.clone()).collect();

                        builtin_mul(args).map_err(|e| e.finish(symbols[0].clone())).map(|v| (v, false))
                    }
                    ExpressionType::Divide => {
                        let args_iter = children.iter().map(|c| c.execute(executor));
                        let args = args_iter.collect::<Result<Vec<_>, GenericError<'file>>>()?.iter().map(|v| v.0.clone()).collect();

                        builtin_div(args).map_err(|e| e.finish(symbols[0].clone())).map(|v| (v, false))
                    }
                    ExpressionType::GreaterThan => {
                        let args_iter = children.iter().map(|c| c.execute(executor));
                        let args = args_iter.collect::<Result<Vec<_>, GenericError<'file>>>()?.iter().map(|v| v.0.clone()).collect();

                        builtin_greater_than(args).map_err(|e| e.finish(symbols[0].clone())).map(|v| (v, false))
                    }
                    ExpressionType::LessThan => {
                        let args_iter = children.iter().map(|c| c.execute(executor));
                        let args = args_iter.collect::<Result<Vec<_>, GenericError<'file>>>()?.iter().map(|v| v.0.clone()).collect();

                        builtin_less_than(args).map_err(|e| e.finish(symbols[0].clone())).map(|v| (v, false))
                    }
                    ExpressionType::GreaterThanEqual => {
                        let args_iter = children.iter().map(|c| c.execute(executor));
                        let args = args_iter.collect::<Result<Vec<_>, GenericError<'file>>>()?.iter().map(|v| v.0.clone()).collect();

                        builtin_greater_than_equal(args).map_err(|e| e.finish(symbols[0].clone())).map(|v| (v, false))
                    }
                    ExpressionType::LessThanEqual => {
                        let args_iter = children.iter().map(|c| c.execute(executor));
                        let args = args_iter.collect::<Result<Vec<_>, GenericError<'file>>>()?.iter().map(|v| v.0.clone()).collect();

                        builtin_less_than_equal(args).map_err(|e| e.finish(symbols[0].clone())).map(|v| (v, false))
                    }
                    ExpressionType::Equality => {
                        let args_iter = children.iter().map(|c| c.execute(executor));
                        let args = args_iter.collect::<Result<Vec<_>, GenericError<'file>>>()?.iter().map(|v| v.0.clone()).collect();

                        builtin_equality(args).map_err(|e| e.finish(symbols[0].clone())).map(|v| (v, false))
                    }
                    ExpressionType::Inequality => {
                        let args_iter = children.iter().map(|c| c.execute(executor));
                        let args = args_iter.collect::<Result<Vec<_>, GenericError<'file>>>()?.iter().map(|v| v.0.clone()).collect();

                        builtin_inequality(args).map_err(|e| e.finish(symbols[0].clone())).map(|v| (v, false))
                    }
                    ExpressionType::LogicalAnd => {
                        builtin_logical_and(children.clone(), executor).map_err(|e| e.finish(symbols[0].clone())).map(|v| (v, false))
                    }
                    ExpressionType::LogicalOr => {
                        builtin_logical_or(children.clone(), executor).map_err(|e| e.finish(symbols[0].clone())).map(|v| (v, false))
                    }
                    ExpressionType::Indexing => {
                        let args_iter = children.iter().map(|c| c.execute(executor));
                        let args = args_iter.collect::<Result<Vec<_>, GenericError<'file>>>()?.iter().map(|v| v.0.clone()).collect();

                        builtin_indexing(args).map_err(|e| e.finish(symbols[0].clone())).map(|v| (v, false))
                    }
                    ExpressionType::MemberAccess => {
                        let v = children[0].execute(executor)?.0;
                        builtin_member_access(v, children[1].get_token().clone()).map_err(|e| e.finish(symbols[0].clone())).map(|v| (v, false))
                    }
                }
            },
            Self::NumericValue { value, .. } => {
                Ok((Value::Number(*value), false))
            },
            Self::IdentifierValue { token } => {
                if token.extract_text() == "True" {
                    Ok((Value::Boolean(true), false))
                }
                else if token.extract_text() == "False" {
                    Ok((Value::Boolean(false), false))
                }
                else {
                    Ok((executor.get_variable(token)?, false))
                }
            }
            Self::ReturnStatement { expression, ..} => {
                if let Some(inner) = expression {
                    let mut v = inner.execute(executor)?;
                    v.1 = true;

                    Ok(v)
                }
                else {
                    Ok((Value::None, true))
                }
            }
            Self::ForLoop { loop_variable, bound0, bound1, reverse, block, .. } => {
                let value0 = bound0.execute(executor)?.0;
                let value1 = bound1.execute(executor)?.0;

                let value0_number = if let Value::Number(v) = value0 {
                    if v.fract() != 0.0 {
                        return Err(GenericError::error(bound0.get_token().clone(), 
                                        format!("first bound {} is not an integer", v)))
                    }
                    else {
                        v as i64
                    }
                }
                else {
                    return Err(GenericError::error(bound0.get_token().clone(), 
                                        "first bound is not a number".to_string()))
                };

                let value1_number = if let Value::Number(v) = value1 {
                    if v.fract() != 0.0 {
                        return Err(GenericError::error(bound1.get_token().clone(), 
                                        format!("second bound {} is not an integer", v)))
                    }
                    else {
                        v as i64
                    }
                }
                else {
                    return Err(GenericError::error(bound1.get_token().clone(), 
                                        "second bound is not a number".to_string()))
                };

                let mut i = value0_number;
                while !reverse && i <= value1_number || *reverse && i >= value1_number {
                    executor.set_variable(loop_variable.extract_text().to_string(), Value::Number(i as f64));
                    block.execute(executor)?;

                    if !reverse {
                        i += 1;
                    }
                    else {
                        i -= 1;
                    }
                }

                Ok((Value::None, false))
            },
            Self::IfStatement { ifs, else_block, .. } => {
                let mut found = false;
                for (_, condition, block) in ifs {
                    if let (Value::Boolean(cond), _) = condition.execute(executor)? {
                        if cond {
                            block.execute(executor)?;
                            found = true;
                            break;
                        }
                    }
                    else {
                        return Err(GenericError::error(condition.get_token().clone(), "condition is not a boolean".to_string()));
                    }
                }

                if !found {
                    if let Some(else_block) = else_block {
                        else_block.execute(executor)?;
                    }
                }

                Ok((Value::None, false))
            }
            _ => 
            {
                dbg!(self);

                todo!()
            }
        }
    }

    pub fn execute_mutable<'a>(&'a self, executor: &'a mut Executor<'file>, value: Value) -> Result<(), GenericError<'file>> {
        match self {
            ParseTreeNode::IdentifierValue { token } => {
                if token.extract_text() == "True" || token.extract_text() == "False" {
                    Err(GenericError::error(self.get_token().clone(), "unable to assign to boolean".to_string()))
                }
                else {
                    *(executor.get_mut_variable(token)?) = value;
                    Ok(())
                }
            }
            ParseTreeNode::Expression { expression_type: ExpressionType::Indexing, symbols, children } => {
                let args_iter = children.iter().map(|c| c.execute(executor));
                let args = args_iter.collect::<Result<Vec<_>, GenericError<'file>>>()?.iter().map(|v| v.0.clone()).collect();

                builtin_mutable_indexing(args, value).map_err(|e| e.finish(symbols[0].clone()))?;
                Ok(())
            }
            ParseTreeNode::Expression { expression_type: ExpressionType::MemberAccess, symbols, children } => {
                let v = children[0].execute(executor)?.0;
                builtin_mutable_member_access(v, children[1].get_token().clone(), value).map_err(|e| e.finish(symbols[0].clone()))?;
                Ok(())
            }
            _ => {
                Err(GenericError::error(self.get_token().clone(), "unable to assign to immutable left hand side".to_string()))
            }
        }
    }
}