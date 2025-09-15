//! Bebion Core Engine
//! 
//! The main engine that orchestrates all components of the runtime.

use bebion_compiler::Compiler;
use bebion_gc::{GarbageCollector, GcHandle};
use bebion_parser::Parser;
use bebion_runtime::{EventLoop, Runtime};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info};

pub struct BebionEngine {
    parser: Parser,
    compiler: Compiler,
    runtime: Runtime,
    event_loop: EventLoop,
    gc: Arc<Mutex<GarbageCollector>>,
    modules: HashMap<String, ModuleInfo>,
}

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub id: String,
    pub path: String,
    pub exports: HashMap<String, GcHandle>,
}

#[derive(Debug)]
pub enum BebionError {
    ParseError(String),
    CompileError(String),
    RuntimeError(String),
    ModuleError(String),
}

impl std::fmt::Display for BebionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BebionError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            BebionError::CompileError(msg) => write!(f, "Compile Error: {}", msg),
            BebionError::RuntimeError(msg) => write!(f, "Runtime Error: {}", msg),
            BebionError::ModuleError(msg) => write!(f, "Module Error: {}", msg),
        }
    }
}

impl std::error::Error for BebionError {}

impl BebionEngine {
    pub fn new() -> Result<Self, BebionError> {
        info!("Initializing Bebion Engine");
        
        let gc = Arc::new(Mutex::new(GarbageCollector::new()));
        let parser = Parser::new();
        let compiler = Compiler::new();
        let runtime = Runtime::new(Arc::clone(&gc));
        let event_loop = EventLoop::new();
        
        Ok(Self {
            parser,
            compiler,
            runtime,
            event_loop,
            gc,
            modules: HashMap::new(),
        })
    }

    pub fn execute_script(&mut self, source: &str) -> Result<GcHandle, BebionError> {
        debug!("Executing script: {} chars", source.len());
        
        // Parse the source code
        let ast = self.parser.parse(source)
            .map_err(|e| BebionError::ParseError(e.to_string()))?;
        
        debug!("Parsed AST with {} nodes", ast.node_count());
        
        // Compile to bytecode
        let bytecode = self.compiler.compile(&ast)
            .map_err(|e| BebionError::CompileError(e.to_string()))?;
        
        debug!("Generated {} bytes of bytecode", bytecode.len());
        
        // Execute in runtime
        let result = self.runtime.execute(&bytecode)
            .map_err(|e| BebionError::RuntimeError(e.to_string()))?;
        
        // Process event loop
        self.event_loop.process_pending();
        
        Ok(result)
    }

    pub fn load_module(&mut self, path: &str) -> Result<ModuleInfo, BebionError> {
        info!("Loading module: {}", path);
        
        if let Some(cached) = self.modules.get(path) {
            debug!("Using cached module: {}", path);
            return Ok(cached.clone());
        }
        
        // Read file content
        let source = std::fs::read_to_string(path)
            .map_err(|e| BebionError::ModuleError(format!("Failed to read {}: {}", path, e)))?;
        
        // Execute module
        let result = self.execute_script(&source)?;
        
        // Create module info
        let module_info = ModuleInfo {
            id: path.to_string(),
            path: path.to_string(),
            exports: HashMap::new(),
        };
        
        self.modules.insert(path.to_string(), module_info.clone());
        
        Ok(module_info)
    }

    pub fn gc_collect(&mut self) -> usize {
        let mut gc = self.gc.lock().unwrap();
        let collected = gc.collect();
        debug!("GC collected {} objects", collected);
        collected
    }

    pub fn shutdown(&mut self) {
        info!("Shutting down Bebion Engine");
        self.event_loop.stop();
        self.gc_collect();
    }
}
