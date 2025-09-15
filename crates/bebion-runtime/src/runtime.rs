//! High-level runtime interface

use crate::{RuntimeError, RuntimeResult, Value, VirtualMachine};
use bebion_compiler::bytecode::Bytecode;
use bebion_gc::{GarbageCollector, GcHandle};
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

pub struct Runtime {
    vm: VirtualMachine,
    gc: Arc<Mutex<GarbageCollector>>,
}

impl Runtime {
    pub fn new(gc: Arc<Mutex<GarbageCollector>>) -> Self {
        let vm = VirtualMachine::new(Arc::clone(&gc));
        
        Self { vm, gc }
    }

    pub fn execute(&mut self, bytecode: &Bytecode) -> RuntimeResult<GcHandle> {
        debug!("Runtime executing bytecode");
        
        let value = self.vm.execute(bytecode)?;
        
        // Convert value to GC handle
        let handle = self.value_to_gc_handle(value)?;
        
        Ok(handle)
    }

    pub fn set_global(&mut self, name: &str, value: Value) {
        self.vm.set_global(name.to_string(), value);
    }

    pub fn get_global(&self, name: &str) -> Option<&Value> {
        self.vm.get_global(name)
    }

    fn value_to_gc_handle(&mut self, value: Value) -> RuntimeResult<GcHandle> {
        match value {
            Value::Object(handle) => Ok(handle),
            Value::Number(n) => {
                let mut gc = self.gc.lock().unwrap();
                Ok(gc.allocate_number(n))
            }
            Value::String(s) => {
                let mut gc = self.gc.lock().unwrap();
                Ok(gc.allocate_string(s))
            }
            Value::Boolean(b) => {
                let mut gc = self.gc.lock().unwrap();
                Ok(gc.allocate_boolean(b))
            }
            Value::Null => {
                let mut gc = self.gc.lock().unwrap();
                Ok(gc.allocate_null())
            }
            Value::Undefined => {
                let mut gc = self.gc.lock().unwrap();
                Ok(gc.allocate_undefined())
            }
        }
    }

    pub fn gc_collect(&mut self) -> usize {
        let mut gc = self.gc.lock().unwrap();
        gc.collect()
    }

    pub fn gc_stats(&self) -> bebion_gc::GcStats {
        let gc = self.gc.lock().unwrap();
        gc.stats()
    }
}