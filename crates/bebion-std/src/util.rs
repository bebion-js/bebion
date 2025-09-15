//! Utility functions module

use crate::{Module, Value};
use bebion_runtime::Runtime;
use std::collections::HashMap;

pub struct UtilModule {
    exports: HashMap<String, Value>,
}

impl UtilModule {
    pub fn new() -> Self {
        let mut exports = HashMap::new();
        
        exports.insert("inspect".to_string(), Value::Undefined);
        exports.insert("format".to_string(), Value::Undefined);
        exports.insert("isArray".to_string(), Value::Undefined);
        exports.insert("isBoolean".to_string(), Value::Undefined);
        exports.insert("isNull".to_string(), Value::Undefined);
        exports.insert("isNumber".to_string(), Value::Undefined);
        exports.insert("isString".to_string(), Value::Undefined);
        exports.insert("isUndefined".to_string(), Value::Undefined);
        exports.insert("isObject".to_string(), Value::Undefined);
        exports.insert("isFunction".to_string(), Value::Undefined);
        
        Self { exports }
    }
    
    pub fn inspect(&self, value: &Value, options: Option<InspectOptions>) -> String {
        let opts = options.unwrap_or_default();
        self.inspect_value(value, 0, &opts)
    }
    
    fn inspect_value(&self, value: &Value, depth: usize, options: &InspectOptions) -> String {
        if depth > options.depth {
            return "[object]".to_string();
        }
        
        match value {
            Value::Number(n) => {
                if options.colors {
                    format!("\x1b[33m{}\x1b[39m", n)
                } else {
                    n.to_string()
                }
            }
            Value::String(s) => {
                if options.colors {
                    format!("\x1b[32m'{}'\x1b[39m", s)
                } else {
                    format!("'{}'", s)
                }
            }
            Value::Boolean(b) => {
                if options.colors {
                    format!("\x1b[33m{}\x1b[39m", b)
                } else {
                    b.to_string()
                }
            }
            Value::Null => {
                if options.colors {
                    "\x1b[1mnull\x1b[22m".to_string()
                } else {
                    "null".to_string()
                }
            }
            Value::Undefined => {
                if options.colors {
                    "\x1b[90mundefined\x1b[39m".to_string()
                } else {
                    "undefined".to_string()
                }
            }
            Value::Object(_) => {
                // This would require access to the GC to inspect object contents
                if options.colors {
                    "\x1b[36m[Object]\x1b[39m".to_string()
                } else {
                    "[Object]".to_string()
                }
            }
        }
    }
    
    pub fn format(&self, template: &str, args: &[Value]) -> String {
        let mut result = String::new();
        let mut chars = template.chars().peekable();
        let mut arg_index = 0;
        
        while let Some(ch) = chars.next() {
            if ch == '%' && chars.peek().is_some() {
                let format_char = chars.next().unwrap();
                
                match format_char {
                    's' => {
                        // String
                        if arg_index < args.len() {
                            result.push_str(&args[arg_index].to_string());
                            arg_index += 1;
                        } else {
                            result.push_str("%s");
                        }
                    }
                    'd' | 'i' => {
                        // Integer
                        if arg_index < args.len() {
                            if let Ok(n) = args[arg_index].to_number() {
                                result.push_str(&(n as i64).to_string());
                            } else {
                                result.push_str("NaN");
                            }
                            arg_index += 1;
                        } else {
                            result.push_str(&format!("%{}", format_char));
                        }
                    }
                    'f' => {
                        // Float
                        if arg_index < args.len() {
                            if let Ok(n) = args[arg_index].to_number() {
                                result.push_str(&n.to_string());
                            } else {
                                result.push_str("NaN");
                            }
                            arg_index += 1;
                        } else {
                            result.push_str("%f");
                        }
                    }
                    'j' => {
                        // JSON
                        if arg_index < args.len() {
                            // This would need proper JSON serialization
                            result.push_str(&args[arg_index].to_string());
                            arg_index += 1;
                        } else {
                            result.push_str("%j");
                        }
                    }
                    'o' | 'O' => {
                        // Object
                        if arg_index < args.len() {
                            result.push_str(&self.inspect(&args[arg_index], None));
                            arg_index += 1;
                        } else {
                            result.push_str(&format!("%{}", format_char));
                        }
                    }
                    '%' => {
                        result.push('%');
                    }
                    _ => {
                        result.push('%');
                        result.push(format_char);
                    }
                }
            } else {
                result.push(ch);
            }
        }
        
        // Append remaining arguments
        while arg_index < args.len() {
            result.push(' ');
            result.push_str(&args[arg_index].to_string());
            arg_index += 1;
        }
        
        result
    }
    
    pub fn is_array(&self, value: &Value) -> bool {
        // This would need access to the GC to check object type
        matches!(value, Value::Object(_))
    }
    
    pub fn is_boolean(&self, value: &Value) -> bool {
        matches!(value, Value::Boolean(_))
    }
    
    pub fn is_null(&self, value: &Value) -> bool {
        matches!(value, Value::Null)
    }
    
    pub fn is_number(&self, value: &Value) -> bool {
        matches!(value, Value::Number(_))
    }
    
    pub fn is_string(&self, value: &Value) -> bool {
        matches!(value, Value::String(_))
    }
    
    pub fn is_undefined(&self, value: &Value) -> bool {
        matches!(value, Value::Undefined)
    }
    
    pub fn is_object(&self, value: &Value) -> bool {
        matches!(value, Value::Object(_))
    }
    
    pub fn is_function(&self, value: &Value) -> bool {
        // This would need access to the GC to check if object is a function
        matches!(value, Value::Object(_))
    }
}

#[derive(Debug, Clone)]
pub struct InspectOptions {
    pub colors: bool,
    pub depth: usize,
    pub show_hidden: bool,
}

impl Default for InspectOptions {
    fn default() -> Self {
        Self {
            colors: false,
            depth: 2,
            show_hidden: false,
        }
    }
}

impl Module for UtilModule {
    fn name(&self) -> &str {
        "util"
    }
    
    fn initialize(&mut self, _runtime: &mut Runtime) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    fn get_exports(&self) -> HashMap<String, Value> {
        self.exports.clone()
    }
}