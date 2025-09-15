//! Process module for system information and control

use crate::{Module, Value};
use bebion_runtime::Runtime;
use std::collections::HashMap;
use std::env;
use std::process;

pub struct ProcessModule {
    exports: HashMap<String, Value>,
    exit_handlers: Vec<Box<dyn FnOnce() + Send>>,
}

impl ProcessModule {
    pub fn new() -> Self {
        let mut exports = HashMap::new();
        
        exports.insert("exit".to_string(), Value::Undefined);
        exports.insert("argv".to_string(), Value::Undefined);
        exports.insert("env".to_string(), Value::Undefined);
        exports.insert("cwd".to_string(), Value::Undefined);
        exports.insert("pid".to_string(), Value::Undefined);
        exports.insert("platform".to_string(), Value::Undefined);
        exports.insert("arch".to_string(), Value::Undefined);
        exports.insert("version".to_string(), Value::Undefined);
        
        Self {
            exports,
            exit_handlers: Vec::new(),
        }
    }
    
    pub fn exit(&self, code: i32) -> ! {
        // Execute exit handlers
        process::exit(code);
    }
    
    pub fn argv(&self) -> Vec<String> {
        env::args().collect()
    }
    
    pub fn env_vars(&self) -> HashMap<String, String> {
        env::vars().collect()
    }
    
    pub fn cwd(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(env::current_dir()?.to_string_lossy().to_string())
    }
    
    pub fn pid(&self) -> u32 {
        process::id()
    }
    
    pub fn platform(&self) -> &'static str {
        if cfg!(target_os = "windows") {
            "win32"
        } else if cfg!(target_os = "macos") {
            "darwin"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "freebsd") {
            "freebsd"
        } else if cfg!(target_os = "openbsd") {
            "openbsd"
        } else if cfg!(target_os = "netbsd") {
            "netbsd"
        } else {
            "unknown"
        }
    }
    
    pub fn arch(&self) -> &'static str {
        if cfg!(target_arch = "x86_64") {
            "x64"
        } else if cfg!(target_arch = "x86") {
            "ia32"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else if cfg!(target_arch = "arm") {
            "arm"
        } else {
            "unknown"
        }
    }
    
    pub fn version(&self) -> String {
        format!("bebion-{}", env!("CARGO_PKG_VERSION"))
    }
    
    pub fn uptime(&self) -> f64 {
        0.0
    }
    
    pub fn memory_usage(&self) -> ProcessMemoryUsage {
        ProcessMemoryUsage {
            rss: 0,
            heap_total: 0,
            heap_used: 0,
            external: 0,
        }
    }
    
    pub fn hrtime(&self) -> (u64, u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        
        (now.as_secs(), now.subsec_nanos() as u64)
    }
    
    pub fn on_exit<F>(&mut self, handler: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.exit_handlers.push(Box::new(handler));
    }
}

#[derive(Debug, Clone)]
pub struct ProcessMemoryUsage {
    pub rss: usize,       // Resident Set Size
    pub heap_total: usize, // Total heap size
    pub heap_used: usize,  // Used heap size
    pub external: usize,   // External memory usage
}

impl Module for ProcessModule {
    fn name(&self) -> &str {
        "process"
    }
    
    fn initialize(&mut self, runtime: &mut Runtime) -> Result<(), Box<dyn std::error::Error>> {
        // Set global process object
        runtime.set_global("process", Value::Object(
            bebion_gc::GcHandle::new(0) // Placeholder
        ));
        
        Ok(())
    }
    
    fn get_exports(&self) -> HashMap<String, Value> {
        self.exports.clone()
    }
}
