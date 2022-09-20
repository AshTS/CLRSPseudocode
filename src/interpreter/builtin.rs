use std::{rc::Rc, cell::RefCell};

use crate::{tokenizer::Token, error::GenericError, parser::ParseTreeNode};

use super::{Value, RuntimeError, Executor};

pub fn get_args1<'a, T: Clone>(args: Vec<T>) -> Result<T, RuntimeError<'a>> {
    if args.len() != 1 {
        Err(RuntimeError::ArgumentCountError { expected: 1, got: args.len() })
    }
    else {
        Ok(args[0].clone())
    }
}

pub fn get_args2<'a, T: Clone>(args: Vec<T>) -> Result<(T, T), RuntimeError<'a>> {
    if args.len() != 2 {
        Err(RuntimeError::ArgumentCountError { expected: 2, got: args.len() })
    }
    else {
        Ok((args[0].clone(), args[1].clone()))
    }
}

pub fn builtin_assert_eq(name: Option<Token<'_>>, args: Vec<Value>) -> Result<Value, RuntimeError<'_>> {
    let (a, b) = get_args2(args)?;

    if a == b {
        Ok(Value::None)
    }
    else {
        Err(GenericError::tokenable_error(name.clone(), format!("assertation failed: values {} and {} do not match", a, b))
                .arrow(format!("values {} and {} do not match", a, b)).into())
    }
}

pub fn builtin_array<'file>(args: Vec<Value>) -> Result<Value, RuntimeError<'file>> {
    let mut vector = Vec::new();

    for v in args {
        vector.push(v);
    }

    Ok(Value::Array(Rc::new(RefCell::new((vector, Value::Number(0.0))))))
}

pub fn builtin_print<'file>(args: Vec<Value>) -> Result<Value, RuntimeError<'file>> {
    for (i, arg) in args.iter().enumerate() {
        if i != 0 {
            print!(", ");
        }
        print!("{}", arg);
    }

    println!();

    Ok(Value::None)
}

pub fn builtin_logical_and<'file>(args: Vec<ParseTreeNode<'file>>, executor: &mut Executor<'file>) -> Result<Value, RuntimeError<'file>> {
    let (a, b) = get_args2(args)?;

    let a = a.execute(executor)?.0;

    if let Value::Boolean(a) = a {
        if !a {
            Ok(Value::Boolean(false))
        }
        else {
            let b = b.execute(executor)?.0;

            if let Value::Boolean(b) = b {
                Ok(Value::Boolean(b))
            }
            else {
                Err(RuntimeError::MessageError(format!("cannot and value of type {}", b.get_type_name())))
            }
        }
    }
    else {
        Err(RuntimeError::MessageError(format!("cannot and value of type {}", a.get_type_name())))
    }
}

pub fn builtin_logical_or<'file>(args: Vec<ParseTreeNode<'file>>, executor: &mut Executor<'file>) -> Result<Value, RuntimeError<'file>> {
    let (a, b) = get_args2(args)?;

    let a = a.execute(executor)?.0;

    if let Value::Boolean(a) = a {
        if a {
            Ok(Value::Boolean(true))
        }
        else {
            let b = b.execute(executor)?.0;

            if let Value::Boolean(b) = b {
                Ok(Value::Boolean(b))
            }
            else {
                Err(RuntimeError::MessageError(format!("cannot or value of type {}", b.get_type_name())))
            }
        }
    }
    else {
        Err(RuntimeError::MessageError(format!("cannot or value of type {}", a.get_type_name())))
    }
}

pub fn builtin_add<'file>(args: Vec<Value>) -> Result<Value, RuntimeError<'file>> {
    let (a, b) = get_args2(args)?;

    if let (Value::Number(a), Value::Number(b)) = (&a, &b) {
        Ok(Value::Number(a + b))
    }
    else {
        Err(RuntimeError::MessageError(format!("cannot add values of type {} and {}", a.get_type_name(), b.get_type_name())))
    }
}

pub fn builtin_sub<'file>(args: Vec<Value>) -> Result<Value, RuntimeError<'file>> {
    let (a, b) = get_args2(args)?;

    if let (Value::Number(a), Value::Number(b)) = (&a, &b) {
        Ok(Value::Number(a - b))
    }
    else {
        Err(RuntimeError::MessageError(format!("cannot subtract values of type {} and {}", a.get_type_name(), b.get_type_name())))
    }
}

pub fn builtin_mul<'file>(args: Vec<Value>) -> Result<Value, RuntimeError<'file>> {
    let (a, b) = get_args2(args)?;

    if let (Value::Number(a), Value::Number(b)) = (&a, &b) {
        Ok(Value::Number(a * b))
    }
    else {
        Err(RuntimeError::MessageError(format!("cannot multiply values of type {} and {}", a.get_type_name(), b.get_type_name())))
    }
}

pub fn builtin_div<'file>(args: Vec<Value>) -> Result<Value, RuntimeError<'file>> {
    let (a, b) = get_args2(args)?;

    if let (Value::Number(a), Value::Number(b)) = (&a, &b) {
        Ok(Value::Number(a / b))
    }
    else {
        Err(RuntimeError::MessageError(format!("cannot divide values of type {} and {}", a.get_type_name(), b.get_type_name())))
    }
}

pub fn builtin_greater_than<'file>(args: Vec<Value>) -> Result<Value, RuntimeError<'file>> {
    let (a, b) = get_args2(args)?;

    if let (Value::Number(a), Value::Number(b)) = (&a, &b) {
        Ok(Value::Boolean(a > b))
    }
    else {
        Err(RuntimeError::MessageError(format!("cannot compare values of type {} and {}", a.get_type_name(), b.get_type_name())))
    }
}

pub fn builtin_less_than<'file>(args: Vec<Value>) -> Result<Value, RuntimeError<'file>> {
    let (a, b) = get_args2(args)?;

    if let (Value::Number(a), Value::Number(b)) = (&a, &b) {
        Ok(Value::Boolean(a < b))
    }
    else {
        Err(RuntimeError::MessageError(format!("cannot compare values of type {} and {}", a.get_type_name(), b.get_type_name())))
    }
}

pub fn builtin_greater_than_equal<'file>(args: Vec<Value>) -> Result<Value, RuntimeError<'file>> {
    let (a, b) = get_args2(args)?;

    if let (Value::Number(a), Value::Number(b)) = (&a, &b) {
        Ok(Value::Boolean(a >= b))
    }
    else {
        Err(RuntimeError::MessageError(format!("cannot compare values of type {} and {}", a.get_type_name(), b.get_type_name())))
    }
}

pub fn builtin_less_than_equal<'file>(args: Vec<Value>) -> Result<Value, RuntimeError<'file>> {
    let (a, b) = get_args2(args)?;

    if let (Value::Number(a), Value::Number(b)) = (&a, &b) {
        Ok(Value::Boolean(a <= b))
    }
    else {
        Err(RuntimeError::MessageError(format!("cannot compare values of type {} and {}", a.get_type_name(), b.get_type_name())))
    }
}

pub fn builtin_equality<'file>(args: Vec<Value>) -> Result<Value, RuntimeError<'file>> {
    let (a, b) = get_args2(args)?;

    Ok(Value::Boolean(a == b))
}

pub fn builtin_inequality<'file>(args: Vec<Value>) -> Result<Value, RuntimeError<'file>> {
    let (a, b) = get_args2(args)?;

    Ok(Value::Boolean(a != b))
}

pub fn builtin_indexing<'file>(args: Vec<Value>) -> Result<Value, RuntimeError<'file>> {
    let (a, b) = get_args2(args)?;

    if let Value::Array(array) = a {
        if let Value::Number(index) = b {
            if index.fract() == 0.0 && index > 0.0 {
                if let Some(value) = array.borrow().0.get(index as usize - 1) {
                    Ok(value.clone())
                }
                else {
                    Err(RuntimeError::MessageError(format!("index {} is out of bounds", b)))
                }
            }
            else {
                Err(RuntimeError::MessageError(format!("index {} is not a positive integer", b)))
            }
        }
        else {
            Err(RuntimeError::MessageError(format!("cannot index using type {}", b.get_type_name())))
        }
    }
    else {
        Err(RuntimeError::MessageError(format!("cannot index into type {}", a.get_type_name())))
    }
}

pub fn builtin_mutable_indexing<'file>(args: Vec<Value>, value_to_assign: Value) -> Result<(), RuntimeError<'file>> {
    let (a, b) = get_args2(args)?;

    if let Value::Array(array) = a{
        if let Value::Number(index) = b {
            if index.fract() == 0.0 && index > 0.0 {
                if let Some(value) = array.borrow_mut().0.get_mut(index as usize - 1) {
                    *value = value_to_assign;
                    Ok(())
                }
                else {
                    Err(RuntimeError::MessageError(format!("index {} is out of bounds", b)))
                }
            }
            else {
                Err(RuntimeError::MessageError(format!("index {} is not a positive integer", b)))
            }
        }
        else {
            Err(RuntimeError::MessageError(format!("cannot index using type {}", b.get_type_name())))
        }
    }
    else {
        Err(RuntimeError::MessageError(format!("cannot index into type {}", a.get_type_name())))
    }
}

pub fn builtin_member_access(arg0: Value, member_token: Token<'_>) -> Result<Value, RuntimeError<'_>> {
    let member = member_token.extract_text().to_string();

    let error_msg = format!("cannot access member '{}' of ", member);
    
    match arg0 {
        Value::Number(_) => 
        {
            let error_text = format!("{} number", error_msg);
            Err(GenericError::error(member_token, error_text.clone()).arrow(error_text).into())
        },
        Value::Array(array) => 
        {
            if member == "length" {
                Ok(Value::Number(array.borrow().0.len() as f64))
            }
            else if member == "heapsize" {
                Ok(array.borrow().1.clone())
            }
            else {
                let error_text = format!("{} none", error_msg);
                Err(GenericError::error(member_token, error_text.clone()).arrow(error_text).into())
            }
        },
        Value::None => 
        {
            let error_text = format!("{} none", error_msg);
            Err(GenericError::error(member_token, error_text.clone()).arrow(error_text).into())
        },
        Value::Boolean(_) => 
        {
            let error_text = format!("{} bool", error_msg);
            Err(GenericError::error(member_token, error_text.clone()).arrow(error_text).into())
        }
    }
}

pub fn builtin_mutable_member_access(arg0: Value, member_token: Token<'_>, value: Value) -> Result<(), RuntimeError<'_>> {
    let member = member_token.extract_text().to_string();

    let error_msg = format!("cannot access member '{}' of ", member);
    
    match arg0 {
        Value::Number(_) => 
        {
            let error_text = format!("{} number", error_msg);
            Err(GenericError::error(member_token, error_text.clone()).arrow(error_text).into())
        },
        Value::Array(array) => 
        {
            if member == "length" {
                Err(GenericError::error(member_token, "member length of array is immutable".to_string()).arrow("member is immutable".to_string()).into())
            }
            else if member == "heapsize" {
                array.borrow_mut().1 = value;
                Ok(())
            }
            else {
                let error_text = format!("{} none", error_msg);
                Err(GenericError::error(member_token, error_text.clone()).arrow(error_text).into())
            }
        },
        Value::None => 
        {
            let error_text = format!("{} none", error_msg);
            Err(GenericError::error(member_token, error_text.clone()).arrow(error_text).into())
        },
        Value::Boolean(_) => 
        {
            let error_text = format!("{} bool", error_msg);
            Err(GenericError::error(member_token, error_text.clone()).arrow(error_text).into())
        }
    }
}

pub fn builtin_floor<'file>(args: Vec<Value>) -> Result<Value, RuntimeError<'file>> {
    let v = get_args1(args)?;

    if let Value::Number(v) = v {
        Ok(Value::Number(v.floor()))
    }
    else {
        Err(RuntimeError::MessageError(format!("cannot take floor of type {}", v.get_type_name())))
    }
}

pub fn builtin_ceil<'file>(args: Vec<Value>) -> Result<Value, RuntimeError<'file>> {
    let v = get_args1(args)?;

    if let Value::Number(v) = v {
        Ok(Value::Number(v.ceil()))
    }
    else {
        Err(RuntimeError::MessageError(format!("cannot take ceiling of type {}", v.get_type_name())))
    }
}