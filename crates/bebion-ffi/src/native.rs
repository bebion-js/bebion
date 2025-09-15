//! Native library integration

use crate::{FfiError, FfiResult};
use bebion_runtime::Value;
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int, c_void};
use tracing::{debug, error};

/// Represents a loaded native library
pub struct NativeLibrary {
    library: Library,
    functions: HashMap<String, FunctionSignature>,
}

/// Function signature for native functions
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub parameter_types: Vec<NativeType>,
    pub return_type: NativeType,
}

/// Native data types supported by the FFI
#[derive(Debug, Clone, PartialEq)]
pub enum NativeType {
    Void,
    Int32,
    Float64,
    String,
    Pointer,
}

/// C-compatible function pointer types
type VoidFn = unsafe extern "C" fn();
type IntFn = unsafe extern "C" fn() -> c_int;
type FloatFn = unsafe extern "C" fn() -> c_double;
type StringFn = unsafe extern "C" fn() -> *const c_char;
type IntIntFn = unsafe extern "C" fn(c_int) -> c_int;
type FloatFloatFn = unsafe extern "C" fn(c_double) -> c_double;
type StringStringFn = unsafe extern "C" fn(*const c_char) -> *const c_char;

impl NativeLibrary {
    /// Load a native library from the given path
    pub fn load(path: &str) -> FfiResult<Self> {
        debug!("Loading native library: {}", path);
        
        let library = unsafe {
            Library::new(path).map_err(|e| {
                FfiError::LibraryNotFound(format!("Failed to load {}: {}", path, e))
            })?
        };

        Ok(Self {
            library,
            functions: HashMap::new(),
        })
    }

    /// Register a function signature
    pub fn register_function(&mut self, signature: FunctionSignature) -> FfiResult<()> {
        // Verify that the symbol exists
        let symbol_name = CString::new(signature.name.as_bytes())
            .map_err(|_| FfiError::InvalidArguments("Invalid function name".to_string()))?;

        unsafe {
            let _symbol: Symbol<*mut c_void> = self.library
                .get(symbol_name.as_bytes())
                .map_err(|_| FfiError::SymbolNotFound(signature.name.clone()))?;
        }

        self.functions.insert(signature.name.clone(), signature);
        debug!("Registered function: {}", &signature.name);
        
        Ok(())
    }

    /// Call a function in the library
    pub fn call_function(&mut self, name: &str, args: Vec<Value>) -> FfiResult<Value> {
        let signature = self.functions.get(name)
            .ok_or_else(|| FfiError::SymbolNotFound(name.to_string()))?
            .clone();

        debug!("Calling native function: {} with {} args", name, args.len());

        // Validate argument count
        if args.len() != signature.parameter_types.len() {
            return Err(FfiError::InvalidArguments(format!(
                "Expected {} arguments, got {}",
                signature.parameter_types.len(),
                args.len()
            )));
        }

        // Convert arguments to native types
        let native_args = self.convert_args_to_native(&args, &signature.parameter_types)?;

        // Call the function based on signature
        let result = unsafe {
            self.call_native_function_unsafe(name, &signature, &native_args)?
        };

        Ok(result)
    }

    /// Convert JavaScript values to native arguments
    fn convert_args_to_native(&self, args: &[Value], types: &[NativeType]) -> FfiResult<Vec<NativeArg>> {
        let mut native_args = Vec::new();

        for (arg, arg_type) in args.iter().zip(types.iter()) {
            let native_arg = match (arg, arg_type) {
                (Value::Number(n), NativeType::Int32) => NativeArg::Int32(*n as i32),
                (Value::Number(n), NativeType::Float64) => NativeArg::Float64(*n),
                (Value::String(s), NativeType::String) => {
                    let c_string = CString::new(s.as_str())
                        .map_err(|_| FfiError::InvalidArguments("Invalid string argument".to_string()))?;
                    NativeArg::String(c_string)
                }
                _ => {
                    return Err(FfiError::InvalidArguments(format!(
                        "Cannot convert {:?} to {:?}",
                        arg, arg_type
                    )));
                }
            };
            native_args.push(native_arg);
        }

        Ok(native_args)
    }

    /// Unsafe function call dispatcher
    unsafe fn call_native_function_unsafe(
        &self,
        name: &str,
        signature: &FunctionSignature,
        args: &[NativeArg],
    ) -> FfiResult<Value> {
        let symbol_name = CString::new(name.as_bytes())
            .map_err(|_| FfiError::InvalidArguments("Invalid function name".to_string()))?;

        match (&signature.parameter_types[..], &signature.return_type) {
            // No parameters
            ([], NativeType::Void) => {
                let func: Symbol<VoidFn> = self.library
                    .get(symbol_name.as_bytes())
                    .map_err(|_| FfiError::SymbolNotFound(name.to_string()))?;
                func();
                Ok(Value::Undefined)
            }
            ([], NativeType::Int32) => {
                let func: Symbol<IntFn> = self.library
                    .get(symbol_name.as_bytes())
                    .map_err(|_| FfiError::SymbolNotFound(name.to_string()))?;
                let result = func();
                Ok(Value::Number(result as f64))
            }
            ([], NativeType::Float64) => {
                let func: Symbol<FloatFloatFn> = self.library
                    .get(symbol_name.as_bytes())
                    .map_err(|_| FfiError::SymbolNotFound(name.to_string()))?;
                let result = func();
                Ok(Value::Number(result))
            }
            ([], NativeType::String) => {
                let func: Symbol<StringFn> = self.library
                    .get(symbol_name.as_bytes())
                    .map_err(|_| FfiError::SymbolNotFound(name.to_string()))?;
                let result_ptr = func();
                if result_ptr.is_null() {
                    Ok(Value::Null)
                } else {
                    let c_str = CStr::from_ptr(result_ptr);
                    let rust_str = c_str.to_string_lossy().into_owned();
                    Ok(Value::String(rust_str))
                }
            }

            // One parameter functions
            ([NativeType::Int32], NativeType::Int32) => {
                let func: Symbol<IntIntFn> = self.library
                    .get(symbol_name.as_bytes())
                    .map_err(|_| FfiError::SymbolNotFound(name.to_string()))?;
                
                if let NativeArg::Int32(arg) = &args[0] {
                    let result = func(*arg);
                    Ok(Value::Number(result as f64))
                } else {
                    Err(FfiError::InvalidArguments("Expected int32 argument".to_string()))
                }
            }
            ([NativeType::Float64], NativeType::Float64) => {
                let func: Symbol<FloatFloatFn> = self.library
                    .get(symbol_name.as_bytes())
                    .map_err(|_| FfiError::SymbolNotFound(name.to_string()))?;
                
                if let NativeArg::Float64(arg) = &args[0] {
                    let result = func(*arg);
                    Ok(Value::Number(result))
                } else {
                    Err(FfiError::InvalidArguments("Expected float64 argument".to_string()))
                }
            }
            ([NativeType::String], NativeType::String) => {
                let func: Symbol<StringStringFn> = self.library
                    .get(symbol_name.as_bytes())
                    .map_err(|_| FfiError::SymbolNotFound(name.to_string()))?;
                
                if let NativeArg::String(arg) = &args[0] {
                    let result_ptr = func(arg.as_ptr());
                    if result_ptr.is_null() {
                        Ok(Value::Null)
                    } else {
                        let c_str = CStr::from_ptr(result_ptr);
                        let rust_str = c_str.to_string_lossy().into_owned();
                        Ok(Value::String(rust_str))
                    }
                } else {
                    Err(FfiError::InvalidArguments("Expected string argument".to_string()))
                }
            }

            _ => {
                error!("Unsupported function signature: {:?}", signature);
                Err(FfiError::InvalidArguments(format!(
                    "Unsupported function signature for {}",
                    name
                )))
            }
        }
    }

    /// Get available function names
    pub fn get_function_names(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }

    /// Get function signature
    pub fn get_function_signature(&self, name: &str) -> Option<&FunctionSignature> {
        self.functions.get(name)
    }
}

/// Native argument wrapper
#[derive(Debug)]
enum NativeArg {
    Int32(i32),
    Float64(f64),
    String(CString),
}

impl FunctionSignature {
    pub fn new(
        name: String,
        parameter_types: Vec<NativeType>,
        return_type: NativeType,
    ) -> Self {
        Self {
            name,
            parameter_types,
            return_type,
        }
    }
}

// Helper functions for common native library patterns
impl NativeLibrary {
    /// Create a simple math library interface
    pub fn create_math_library() -> FunctionSignature {
        FunctionSignature::new(
            "add".to_string(),
            vec![NativeType::Float64, NativeType::Float64],
            NativeType::Float64,
        )
    }

    /// Create a string processing library interface
    pub fn create_string_library() -> Vec<FunctionSignature> {
        vec![
            FunctionSignature::new(
                "strlen".to_string(),
                vec![NativeType::String],
                NativeType::Int32,
            ),
            FunctionSignature::new(
                "strdup".to_string(),
                vec![NativeType::String],
                NativeType::String,
            ),
        ]
    }
}