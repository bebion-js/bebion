//! Bebion Garbage Collector
//! 
//! Incremental, generational garbage collector with mark-and-sweep.

use std::collections::{HashMap, HashSet};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{debug, trace};

/// Handle to a garbage-collected object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GcHandle(usize);

/// Generation of a garbage-collected object
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Generation {
    Young,
    Old,
}

/// Type of garbage-collected object
#[derive(Debug, Clone)]
pub enum GcObjectType {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
    Object(HashMap<String, GcHandle>),
    Array(Vec<GcHandle>),
    Function {
        name: Option<String>,
        bytecode: Vec<u8>,
        closure: HashMap<String, GcHandle>,
    },
    Promise {
        state: PromiseState,
        value: Option<GcHandle>,
        callbacks: Vec<GcHandle>,
    },
}

#[derive(Debug, Clone)]
pub enum PromiseState {
    Pending,
    Fulfilled,
    Rejected,
}

/// Garbage-collected object
#[derive(Debug)]
struct GcObject {
    object_type: GcObjectType,
    generation: Generation,
    marked: bool,
    size: usize,
    references: HashSet<GcHandle>,
}

/// Garbage collector state
pub struct GarbageCollector {
    objects: HashMap<GcHandle, GcObject>,
    next_handle: AtomicUsize,
    root_set: HashSet<GcHandle>,
    young_objects: HashSet<GcHandle>,
    old_objects: HashSet<GcHandle>,
    
    // Statistics
    total_allocations: usize,
    total_collections: usize,
    bytes_allocated: usize,
    bytes_freed: usize,
    
    // Collection thresholds
    young_threshold: usize,
    old_threshold: usize,
    collection_frequency: usize,
}

impl GcHandle {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
    
    pub fn id(&self) -> usize {
        self.0
    }
}

impl GarbageCollector {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            next_handle: AtomicUsize::new(1),
            root_set: HashSet::new(),
            young_objects: HashSet::new(),
            old_objects: HashSet::new(),
            
            total_allocations: 0,
            total_collections: 0,
            bytes_allocated: 0,
            bytes_freed: 0,
            
            young_threshold: 1024 * 1024,      // 1MB
            old_threshold: 10 * 1024 * 1024,   // 10MB
            collection_frequency: 100,
        }
    }

    /// Allocate a new object and return its handle
    pub fn allocate(&mut self, object_type: GcObjectType) -> GcHandle {
        let handle = GcHandle(self.next_handle.fetch_add(1, Ordering::SeqCst));
        let size = self.calculate_object_size(&object_type);
        
        let references = self.extract_references(&object_type);
        
        let object = GcObject {
            object_type,
            generation: Generation::Young,
            marked: false,
            size,
            references,
        };
        
        self.objects.insert(handle, object);
        self.young_objects.insert(handle);
        
        self.total_allocations += 1;
        self.bytes_allocated += size;
        
        trace!("Allocated object {} with size {} bytes", handle.0, size);
        
        // Trigger collection if threshold reached
        if self.should_collect() {
            self.collect();
        }
        
        handle
    }

    /// Add a handle to the root set
    pub fn add_root(&mut self, handle: GcHandle) {
        self.root_set.insert(handle);
    }

    /// Remove a handle from the root set
    pub fn remove_root(&mut self, handle: GcHandle) {
        self.root_set.remove(&handle);
    }

    /// Get the type of an object
    pub fn get_object_type(&self, handle: GcHandle) -> Option<&GcObjectType> {
        self.objects.get(&handle).map(|obj| &obj.object_type)
    }

    /// Update an object's type (for mutation)
    pub fn update_object(&mut self, handle: GcHandle, new_type: GcObjectType) -> bool {
        if let Some(object) = self.objects.get_mut(&handle) {
            let old_size = object.size;
            let new_size = self.calculate_object_size(&new_type);
            let new_references = self.extract_references(&new_type);
            
            object.object_type = new_type;
            object.size = new_size;
            object.references = new_references;
            
            self.bytes_allocated = self.bytes_allocated.saturating_sub(old_size) + new_size;
            
            true
        } else {
            false
        }
    }

    /// Perform garbage collection
    pub fn collect(&mut self) -> usize {
        debug!("Starting garbage collection cycle {}", self.total_collections + 1);
        
        let initial_count = self.objects.len();
        let initial_bytes = self.bytes_allocated;
        
        // Decide whether to collect young generation only or full collection
        let full_collection = self.total_collections % 10 == 0;
        
        if full_collection {
            self.full_collect()
        } else {
            self.minor_collect()
        }
        
        let final_count = self.objects.len();
        let final_bytes = self.bytes_allocated;
        
        let collected_objects = initial_count - final_count;
        let collected_bytes = initial_bytes - final_bytes;
        
        self.total_collections += 1;
        self.bytes_freed += collected_bytes;
        
        debug!(
            "Completed GC cycle: collected {} objects ({} bytes), {} objects remaining",
            collected_objects, collected_bytes, final_count
        );
        
        collected_objects
    }

    /// Minor collection (young generation only)
    fn minor_collect(&mut self) -> usize {
        debug!("Performing minor collection (young generation)");
        
        // Mark phase - start from roots
        self.clear_marks();
        self.mark_from_roots();
        
        // Promote surviving young objects to old generation
        let mut promoted = Vec::new();
        for &handle in &self.young_objects {
            if let Some(object) = self.objects.get_mut(&handle) {
                if object.marked {
                    object.generation = Generation::Old;
                    promoted.push(handle);
                }
            }
        }
        
        // Move promoted objects to old generation set
        for handle in promoted {
            self.young_objects.remove(&handle);
            self.old_objects.insert(handle);
        }
        
        // Sweep phase - collect unmarked young objects
        let mut to_remove = Vec::new();
        for &handle in &self.young_objects {
            if let Some(object) = self.objects.get(&handle) {
                if !object.marked {
                    to_remove.push(handle);
                }
            }
        }
        
        self.remove_objects(&to_remove)
    }

    /// Full collection (all generations)
    fn full_collect(&mut self) -> usize {
        debug!("Performing full collection (all generations)");
        
        // Mark phase - start from roots
        self.clear_marks();
        self.mark_from_roots();
        
        // Sweep phase - collect all unmarked objects
        let mut to_remove = Vec::new();
        for (&handle, object) in &self.objects {
            if !object.marked {
                to_remove.push(handle);
            }
        }
        
        self.remove_objects(&to_remove)
    }

    /// Clear all mark flags
    fn clear_marks(&mut self) {
        for object in self.objects.values_mut() {
            object.marked = false;
        }
    }

    /// Mark objects reachable from roots
    fn mark_from_roots(&mut self) {
        let roots: Vec<_> = self.root_set.iter().cloned().collect();
        for root in roots {
            self.mark_object(root);
        }
    }

    /// Mark an object and all objects it references
    fn mark_object(&mut self, handle: GcHandle) {
        if let Some(object) = self.objects.get_mut(&handle) {
            if object.marked {
                return; // Already marked
            }
            
            object.marked = true;
            let references: Vec<_> = object.references.iter().cloned().collect();
            
            // Mark all referenced objects
            for referenced_handle in references {
                self.mark_object(referenced_handle);
            }
        }
    }

    /// Remove a list of objects from the collector
    fn remove_objects(&mut self, handles: &[GcHandle]) -> usize {
        let mut freed_bytes = 0;
        
        for &handle in handles {
            if let Some(object) = self.objects.remove(&handle) {
                freed_bytes += object.size;
                self.young_objects.remove(&handle);
                self.old_objects.remove(&handle);
                self.root_set.remove(&handle);
            }
        }
        
        self.bytes_allocated = self.bytes_allocated.saturating_sub(freed_bytes);
        handles.len()
    }

    /// Check if collection should be triggered
    fn should_collect(&self) -> bool {
        let young_bytes: usize = self.young_objects.iter()
            .filter_map(|&h| self.objects.get(&h))
            .map(|obj| obj.size)
            .sum();
        
        young_bytes > self.young_threshold ||
        self.total_allocations % self.collection_frequency == 0
    }

    /// Calculate the size of an object in bytes
    fn calculate_object_size(&self, object_type: &GcObjectType) -> usize {
        match object_type {
            GcObjectType::Number(_) => 8,
            GcObjectType::Boolean(_) => 1,
            GcObjectType::Null | GcObjectType::Undefined => 0,
            GcObjectType::String(s) => s.len(),
            GcObjectType::Object(map) => map.len() * 16, // Rough estimate
            GcObjectType::Array(arr) => arr.len() * 8,
            GcObjectType::Function { bytecode, closure, .. } => {
                bytecode.len() + closure.len() * 16
            }
            GcObjectType::Promise { .. } => 64, // Rough estimate
        }
    }

    /// Extract references from an object
    fn extract_references(&self, object_type: &GcObjectType) -> HashSet<GcHandle> {
        let mut references = HashSet::new();
        
        match object_type {
            GcObjectType::Object(map) => {
                for &handle in map.values() {
                    references.insert(handle);
                }
            }
            GcObjectType::Array(arr) => {
                for &handle in arr {
                    references.insert(handle);
                }
            }
            GcObjectType::Function { closure, .. } => {
                for &handle in closure.values() {
                    references.insert(handle);
                }
            }
            GcObjectType::Promise { value, callbacks, .. } => {
                if let Some(handle) = value {
                    references.insert(*handle);
                }
                for &handle in callbacks {
                    references.insert(handle);
                }
            }
            _ => {}
        }
        
        references
    }

    /// Get garbage collection statistics
    pub fn stats(&self) -> GcStats {
        GcStats {
            total_objects: self.objects.len(),
            young_objects: self.young_objects.len(),
            old_objects: self.old_objects.len(),
            root_objects: self.root_set.len(),
            total_allocations: self.total_allocations,
            total_collections: self.total_collections,
            bytes_allocated: self.bytes_allocated,
            bytes_freed: self.bytes_freed,
        }
    }

    /// Force a full garbage collection
    pub fn force_collect(&mut self) -> usize {
        self.full_collect()
    }

    /// Set collection thresholds
    pub fn set_thresholds(&mut self, young_threshold: usize, old_threshold: usize) {
        self.young_threshold = young_threshold;
        self.old_threshold = old_threshold;
    }
}

/// Garbage collection statistics
#[derive(Debug, Clone)]
pub struct GcStats {
    pub total_objects: usize,
    pub young_objects: usize,
    pub old_objects: usize,
    pub root_objects: usize,
    pub total_allocations: usize,
    pub total_collections: usize,
    pub bytes_allocated: usize,
    pub bytes_freed: usize,
}

impl Default for GarbageCollector {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions for creating common object types
impl GarbageCollector {
    pub fn allocate_number(&mut self, value: f64) -> GcHandle {
        self.allocate(GcObjectType::Number(value))
    }
    
    pub fn allocate_string(&mut self, value: String) -> GcHandle {
        self.allocate(GcObjectType::String(value))
    }
    
    pub fn allocate_boolean(&mut self, value: bool) -> GcHandle {
        self.allocate(GcObjectType::Boolean(value))
    }
    
    pub fn allocate_null(&mut self) -> GcHandle {
        self.allocate(GcObjectType::Null)
    }
    
    pub fn allocate_undefined(&mut self) -> GcHandle {
        self.allocate(GcObjectType::Undefined)
    }
    
    pub fn allocate_object(&mut self, properties: HashMap<String, GcHandle>) -> GcHandle {
        self.allocate(GcObjectType::Object(properties))
    }
    
    pub fn allocate_array(&mut self, elements: Vec<GcHandle>) -> GcHandle {
        self.allocate(GcObjectType::Array(elements))
    }
    
    pub fn allocate_function(
        &mut self,
        name: Option<String>,
        bytecode: Vec<u8>,
        closure: HashMap<String, GcHandle>
    ) -> GcHandle {
        self.allocate(GcObjectType::Function { name, bytecode, closure })
    }
}
