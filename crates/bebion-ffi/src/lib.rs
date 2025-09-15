//! Bebion Foreign Function Interface
//! 
//! Provides integration with native modules and WASI.

pub mod native;
pub mod wasi;

use bebion_runtime::{Runtime, Value};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub enum FfiError {
    LibraryNotFound(String),
    SymbolNotFound(String),
    InvalidArguments(String),
    RuntimeError(String),
    WasmError(String),
}

impl fmt::Display for FfiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FfiError::LibraryNotFound(lib) => write!(f, "Library not found: {}", lib),
            FfiError::SymbolNotFound(symbol) => write!(f, "Symbol not found: {}", symbol),
            FfiError::InvalidArguments(msg) => write!(f, "Invalid arguments: {}", msg),
            FfiError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            FfiError::WasmError(msg) => write!(f, "WASM error: {}", msg),
        }
    }
}

impl std::error::Error for FfiError {}

pub type FfiResult<T> = Result<T, FfiError>;

/// FFI manager for handling native libraries and WASI modules
pub struct FfiManager {
    native_libraries: HashMap<String, native::NativeLibrary>,
    wasi_modules: HashMap<String, wasi::WasiModule>,
}

impl FfiManager {
    pub fn new() -> Self {
        Self {
            native_libraries: HashMap::new(),
            wasi_modules: HashMap::new(),
        }
    }

    /// Load a native library
    pub fn load_native_library(&mut self, name: &str, path: &str) -> FfiResult<()> {
        let library = native::NativeLibrary::load(path)?;
        self.native_libraries.insert(name.to_string(), library);
        Ok(())
    }

    /// Load a WASI module
    pub fn load_wasi_module(&mut self, name: &str, path: &str) -> FfiResult<()> {
        let module = wasi::WasiModule::load(path)?;
        self.wasi_modules.insert(name.to_string(), module);
        Ok(())
    }

    /// Call a native function
    pub fn call_native_function(
        &mut self,
        library: &str,
        function: &str,
        args: Vec<Value>,
    ) -> FfiResult<Value> {
        let lib = self.native_libraries.get_mut(library)
            .ok_or_else(|| FfiError::LibraryNotFound(library.to_string()))?;
        
        lib.call_function(function, args)
    }

    /// Call a WASI function
    pub fn call_wasi_function(
        &mut self,
        module: &str,
        function: &str,
        args: Vec<Value>,
    ) -> FfiResult<Value> {
        let wasi_module = self.wasi_modules.get_mut(module)
            .ok_or_else(|| FfiError::LibraryNotFound(module.to_string()))?;
        
        wasi_module.call_function(function, args)
    }

    /// Initialize FFI runtime globals
    pub fn initialize_runtime(&self, runtime: &mut Runtime) -> FfiResult<()> {
        // Set up global FFI functions
        runtime.set_global("loadNativeLibrary", Value::Undefined); // Would need proper function
        runtime.set_global("loadWasiModule", Value::Undefined);
        runtime.set_global("callNative", Value::Undefined);
        runtime.set_global("callWasi", Value::Undefined);
        
        Ok(())
    }

    /// Get list of loaded libraries
    pub fn get_loaded_libraries(&self) -> Vec<String> {
        self.native_libraries.keys().cloned().collect()
    }

    /// Get list of loaded WASI modules
    pub fn get_loaded_modules(&self) -> Vec<String> {
        self.wasi_modules.keys().cloned().collect()
    }
}

impl Default for FfiManager {
    fn default() -> Self {
        Self::new()
    }
}