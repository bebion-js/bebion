//! JavaScript value representation

use bebion_gc::{GcHandle, GcObjectType};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
    Object(GcHandle),
}

impl Value {
    pub fn from_gc_object_type(obj_type: &GcObjectType, handle: GcHandle) -> Self {
        match obj_type {
            GcObjectType::Number(n) => Value::Number(*n),
            GcObjectType::String(s) => Value::String(s.clone()),
            GcObjectType::Boolean(b) => Value::Boolean(*b),
            GcObjectType::Null => Value::Null,
            GcObjectType::Undefined => Value::Undefined,
            _ => Value::Object(handle),
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0 && !n.is_nan(),
            Value::String(s) => !s.is_empty(),
            Value::Null | Value::Undefined => false,
            Value::Object(_) => true,
        }
    }

    pub fn to_number(&self) -> Result<f64, crate::RuntimeError> {
        match self {
            Value::Number(n) => Ok(*n),
            Value::Boolean(true) => Ok(1.0),
            Value::Boolean(false) => Ok(0.0),
            Value::String(s) => {
                s.parse::<f64>().map_err(|_| {
                    crate::RuntimeError::TypeError(format!("Cannot convert string '{}' to number", s))
                })
            }
            Value::Null => Ok(0.0),
            Value::Undefined => Ok(f64::NAN),
            Value::Object(_) => Err(crate::RuntimeError::TypeError(
                "Cannot convert object to number".to_string()
            )),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 && n.is_finite() {
                    format!("{}", *n as i64)
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => s.clone(),
            Value::Boolean(true) => "true".to_string(),
            Value::Boolean(false) => "false".to_string(),
            Value::Null => "null".to_string(),
            Value::Undefined => "undefined".to_string(),
            Value::Object(_) => "[object Object]".to_string(),
        }
    }

    pub fn typeof_string(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Null => "object", // JavaScript quirk
            Value::Undefined => "undefined",
            Value::Object(_) => "object",
        }
    }

    pub fn is_primitive(&self) -> bool {
        !matches!(self, Value::Object(_))
    }

    pub fn strict_equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => {
                if a.is_nan() && b.is_nan() {
                    false // NaN !== NaN
                } else {
                    a == b
                }
            }
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Undefined, Value::Undefined) => true,
            (Value::Object(a), Value::Object(b)) => a == b,
            _ => false,
        }
    }

    pub fn loose_equals(&self, other: &Value) -> bool {
        // Implement JavaScript's == operator
        match (self, other) {
            // Same type comparisons
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Undefined, Value::Undefined) => true,
            (Value::Object(a), Value::Object(b)) => a == b,
            
            // null == undefined
            (Value::Null, Value::Undefined) | (Value::Undefined, Value::Null) => true,
            
            // Number and string conversion
            (Value::Number(n), Value::String(s)) | (Value::String(s), Value::Number(n)) => {
                if let Ok(s_num) = s.parse::<f64>() {
                    n == &s_num
                } else {
                    false
                }
            }
            
            // Boolean conversion
            (Value::Boolean(b), other) | (other, Value::Boolean(b)) => {
                let b_num = if *b { 1.0 } else { 0.0 };
                Value::Number(b_num).loose_equals(other)
            }
            
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::Number(n as f64)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

// Utility functions for value operations
pub fn add_values(left: &Value, right: &Value) -> Result<Value, crate::RuntimeError> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
        (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
        (Value::String(a), b) => Ok(Value::String(format!("{}{}", a, b.to_string()))),
        (a, Value::String(b)) => Ok(Value::String(format!("{}{}", a.to_string(), b))),
        (a, b) => {
            let a_num = a.to_number()?;
            let b_num = b.to_number()?;
            Ok(Value::Number(a_num + b_num))
        }
    }
}

pub fn subtract_values(left: &Value, right: &Value) -> Result<Value, crate::RuntimeError> {
    let a = left.to_number()?;
    let b = right.to_number()?;
    Ok(Value::Number(a - b))
}

pub fn multiply_values(left: &Value, right: &Value) -> Result<Value, crate::RuntimeError> {
    let a = left.to_number()?;
    let b = right.to_number()?;
    Ok(Value::Number(a * b))
}

pub fn divide_values(left: &Value, right: &Value) -> Result<Value, crate::RuntimeError> {
    let a = left.to_number()?;
    let b = right.to_number()?;
    Ok(Value::Number(a / b))
}

pub fn modulo_values(left: &Value, right: &Value) -> Result<Value, crate::RuntimeError> {
    let a = left.to_number()?;
    let b = right.to_number()?;
    Ok(Value::Number(a % b))
}

pub fn power_values(left: &Value, right: &Value) -> Result<Value, crate::RuntimeError> {
    let a = left.to_number()?;
    let b = right.to_number()?;
    Ok(Value::Number(a.powf(b)))
}