use std::{rc::Rc, cell::RefCell};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Array(Rc<RefCell<(Vec<Value>, Value)>>),
    Boolean(bool),
    None
}

impl std::convert::From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Number(v)
    }
}

impl std::convert::From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Boolean(v)
    }
}

impl std::convert::From<Option<Value>> for Value {
    fn from(v: Option<Value>) -> Self {
        if let Some(v) = v {
            v
        }
        else {
            Value::None
        }
    }
}

impl Value {
    pub fn get_type_name(&self) -> &str {
        match self {
            Value::Number(_) => "number",
            Value::None => "none",
            Value::Boolean(_) => "bool",
            Value::Array(_) => "array"
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Value::Number(_))
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(v) => write!(f, "{}", v),
            Value::None => write!(f, "None"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Array(array) => {
                write!(f, "[")?;

                for (i, v) in array.borrow().0.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }

                write!(f, "]")?;
                Ok(())
            }
        }
    }
}