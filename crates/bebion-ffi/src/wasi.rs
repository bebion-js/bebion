//! WASI (WebAssembly System Interface) integration

use crate::{FfiError, FfiResult};
use bebion_runtime::Value;
use std::collections::HashMap;
use tracing::{debug, error};

#[cfg(not(target_family = "wasm"))]
use wasmtime::{Engine, Instance, Linker, Module, Store, WasmParams, WasmResults};

/// WASI module wrapper
pub struct WasiModule {
    #[cfg(not(target_family = "wasm"))]
    instance: Instance,
    #[cfg(not(target_family = "wasm"))]
    store: Store<WasiState>,
    functions: HashMap<String, WasiFunctionInfo>,
}

/// WASI function information
#[derive(Debug, Clone)]
pub struct WasiFunctionInfo {
    pub name: String,
    pub parameter_count: usize,
    pub return_count: usize,
}

/// WASI state for the store
#[derive(Default)]
struct WasiState {
    // Add any state needed for WASI operations
}

impl WasiModule {
    /// Load a WASI module from file
    pub fn load(path: &str) -> FfiResult<Self> {
        debug!("Loading WASI module: {}", path);

        #[cfg(not(target_family = "wasm"))]
        {
            let engine = Engine::default();
            let module = Module::from_file(&engine, path)
                .map_err(|e| FfiError::WasmError(format!("Failed to load module: {}", e)))?;

            let mut linker = Linker::new(&engine);
            
            // Add WASI imports
            wasmtime_wasi::add_to_linker(&mut linker, |s| s)
                .map_err(|e| FfiError::WasmError(format!("Failed to add WASI to linker: {}", e)))?;

            let wasi = wasmtime_wasi::WasiCtxBuilder::new()
                .inherit_stdio()
                .inherit_args()
                .map_err(|e| FfiError::WasmError(format!("Failed to create WASI context: {}", e)))?
                .build();

            let mut store = Store::new(&engine, WasiState::default());
            store.data_mut().wasi = wasi;

            let instance = linker
                .instantiate(&mut store, &module)
                .map_err(|e| FfiError::WasmError(format!("Failed to instantiate module: {}", e)))?;

            // Discover exported functions
            let mut functions = HashMap::new();
            for export in module.exports() {
                if let Some(func_type) = export.ty().func() {
                    let info = WasiFunctionInfo {
                        name: export.name().to_string(),
                        parameter_count: func_type.params().len(),
                        return_count: func_type.results().len(),
                    };
                    functions.insert(export.name().to_string(), info);
                    debug!("Discovered WASI function: {}", export.name());
                }
            }

            Ok(Self {
                instance,
                store,
                functions,
            })
        }

        #[cfg(target_family = "wasm")]
        {
            // WASI not supported in WASM target
            Err(FfiError::WasmError("WASI not supported in WASM target".to_string()))
        }
    }

    /// Call a function in the WASI module
    pub fn call_function(&mut self, name: &str, args: Vec<Value>) -> FfiResult<Value> {
        debug!("Calling WASI function: {} with {} args", name, args.len());

        #[cfg(not(target_family = "wasm"))]
        {
            let func_info = self.functions.get(name)
                .ok_or_else(|| FfiError::SymbolNotFound(name.to_string()))?;

            // Validate argument count
            if args.len() != func_info.parameter_count {
                return Err(FfiError::InvalidArguments(format!(
                    "Expected {} arguments, got {}",
                    func_info.parameter_count,
                    args.len()
                )));
            }

            // Get the function from the instance
            let func = self.instance
                .get_func(&mut self.store, name)
                .ok_or_else(|| FfiError::SymbolNotFound(name.to_string()))?;

            // Convert JavaScript values to WASM values
            let wasm_args = self.convert_args_to_wasm(&args)?;

            // Prepare results buffer
            let mut results = vec![wasmtime::Val::I32(0); func_info.return_count];

            // Call the function
            func.call(&mut self.store, &wasm_args, &mut results)
                .map_err(|e| FfiError::RuntimeError(format!("WASM function call failed: {}", e)))?;

            // Convert results back to JavaScript values
            if results.is_empty() {
                Ok(Value::Undefined)
            } else {
                self.convert_wasm_to_value(&results[0])
            }
        }

        #[cfg(target_family = "wasm")]
        {
            Err(FfiError::WasmError("WASI not supported in WASM target".to_string()))
        }
    }

    #[cfg(not(target_family = "wasm"))]
    fn convert_args_to_wasm(&self, args: &[Value]) -> FfiResult<Vec<wasmtime::Val>> {
        let mut wasm_args = Vec::new();

        for arg in args {
            let wasm_val = match arg {
                Value::Number(n) => {
                    if n.fract() == 0.0 && *n >= i32::MIN as f64 && *n <= i32::MAX as f64 {
                        wasmtime::Val::I32(*n as i32)
                    } else {
                        wasmtime::Val::F64(*n)
                    }
                }
                Value::Boolean(b) => wasmtime::Val::I32(if *b { 1 } else { 0 }),
                _ => {
                    return Err(FfiError::InvalidArguments(format!(
                        "Cannot convert {:?} to WASM value",
                        arg
                    )));
                }
            };
            wasm_args.push(wasm_val);
        }

        Ok(wasm_args)
    }

    #[cfg(not(target_family = "wasm"))]
    fn convert_wasm_to_value(&self, wasm_val: &wasmtime::Val) -> FfiResult<Value> {
        match wasm_val {
            wasmtime::Val::I32(i) => Ok(Value::Number(*i as f64)),
            wasmtime::Val::I64(i) => Ok(Value::Number(*i as f64)),
            wasmtime::Val::F32(f) => Ok(Value::Number(*f as f64)),
            wasmtime::Val::F64(f) => Ok(Value::Number(*f)),
            _ => Err(FfiError::InvalidArguments(
                "Unsupported WASM return type".to_string()
            )),
        }
    }

    /// Get available function names
    pub fn get_function_names(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }

    /// Get function information
    pub fn get_function_info(&self, name: &str) -> Option<&WasiFunctionInfo> {
        self.functions.get(name)
    }

    /// Get memory from the WASI module (if available)
    #[cfg(not(target_family = "wasm"))]
    pub fn get_memory(&mut self) -> Option<wasmtime::Memory> {
        self.instance.get_memory(&mut self.store, "memory")
    }

    /// Read string from WASI memory
    #[cfg(not(target_family = "wasm"))]
    pub fn read_string(&mut self, ptr: u32, len: u32) -> FfiResult<String> {
        if let Some(memory) = self.get_memory() {
            let data = memory.data(&self.store);
            let start = ptr as usize;
            let end = start + len as usize;
            
            if end <= data.len() {
                let bytes = &data[start..end];
                String::from_utf8(bytes.to_vec())
                    .map_err(|e| FfiError::RuntimeError(format!("Invalid UTF-8: {}", e)))
            } else {
                Err(FfiError::RuntimeError("Memory access out of bounds".to_string()))
            }
        } else {
            Err(FfiError::RuntimeError("No memory export found".to_string()))
        }
    }

    /// Write string to WASI memory
    #[cfg(not(target_family = "wasm"))]
    pub fn write_string(&mut self, s: &str) -> FfiResult<u32> {
        if let Some(memory) = self.get_memory() {
            let bytes = s.as_bytes();
            let data = memory.data_mut(&mut self.store);
            
            // Simple allocation - in a real implementation, you'd need a proper allocator
            let ptr = data.len() as u32;
            
            // This is a simplified example - real WASI modules would have proper memory management
            Err(FfiError::RuntimeError("Memory allocation not implemented".to_string()))
        } else {
            Err(FfiError::RuntimeError("No memory export found".to_string()))
        }
    }
}

// Add the missing WasiState field
#[cfg(not(target_family = "wasm"))]
impl WasiState {
    fn new() -> Self {
        Self::default()
    }
}

#[cfg(not(target_family = "wasm"))]
impl WasiState {
    pub wasi: wasmtime_wasi::WasiCtx,
}

// Helper functions for common WASI patterns
impl WasiModule {
    /// Create a simple calculator WASI module interface
    pub fn create_calculator_interface() -> Vec<WasiFunctionInfo> {
        vec![
            WasiFunctionInfo {
                name: "add".to_string(),
                parameter_count: 2,
                return_count: 1,
            },
            WasiFunctionInfo {
                name: "subtract".to_string(),
                parameter_count: 2,
                return_count: 1,
            },
            WasiFunctionInfo {
                name: "multiply".to_string(),
                parameter_count: 2,
                return_count: 1,
            },
            WasiFunctionInfo {
                name: "divide".to_string(),
                parameter_count: 2,
                return_count: 1,
            },
        ]
    }
}