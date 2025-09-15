//! Console module for logging and debugging

use crate::{Module, Value};
use bebion_runtime::Runtime;
use std::collections::HashMap;
use std::io::{self, Write};

pub struct ConsoleModule {
    exports: HashMap<String, Value>,
}

impl ConsoleModule {
    pub fn new() -> Self {
        let mut exports = HashMap::new();
        
        // Add console functions
        exports.insert("log".to_string(), Value::Undefined); // Placeholder
        exports.insert("error".to_string(), Value::Undefined);
        exports.insert("warn".to_string(), Value::Undefined);
        exports.insert("info".to_string(), Value::Undefined);
        exports.insert("debug".to_string(), Value::Undefined);
        exports.insert("trace".to_string(), Value::Undefined);
        exports.insert("clear".to_string(), Value::Undefined);
        exports.insert("time".to_string(), Value::Undefined);
        exports.insert("timeEnd".to_string(), Value::Undefined);
        
        Self { exports }
    }
    
    pub fn log(&self, args: Vec<Value>) {
        let message = args.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        
        println!("{}", message);
        io::stdout().flush().unwrap_or(());
    }
    
    pub fn error(&self, args: Vec<Value>) {
        let message = args.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        
        eprintln!("{}", message);
        io::stderr().flush().unwrap_or(());
    }
    
    pub fn warn(&self, args: Vec<Value>) {
        let message = args.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        
        eprintln!("Warning: {}", message);
        io::stderr().flush().unwrap_or(());
    }
    
    pub fn info(&self, args: Vec<Value>) {
        let message = args.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        
        println!("Info: {}", message);
        io::stdout().flush().unwrap_or(());
    }
    
    pub fn debug(&self, args: Vec<Value>) {
        let message = args.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        
        println!("Debug: {}", message);
        io::stdout().flush().unwrap_or(());
    }
    
    pub fn clear(&self) {
        // Clear the console
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap_or(());
    }
}

impl Module for ConsoleModule {
    fn name(&self) -> &str {
        "console"
    }
    
    fn initialize(&mut self, runtime: &mut Runtime) -> Result<(), Box<dyn std::error::Error>> {
        // Set global console object
        runtime.set_global("console", Value::Object(
            // This would need proper object creation with methods
            bebion_gc::GcHandle::new(0) // Placeholder
        ));
        
        Ok(())
    }
    
    fn get_exports(&self) -> HashMap<String, Value> {
        self.exports.clone()
    }
}