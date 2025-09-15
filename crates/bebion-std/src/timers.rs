//! Timers module for setTimeout, setInterval, etc.

use crate::{Module, Value};
use bebion_runtime::Runtime;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::{sleep, interval};

pub struct TimersModule {
    exports: HashMap<String, Value>,
    timers: Arc<Mutex<HashMap<u64, TimerHandle>>>,
    next_id: Arc<Mutex<u64>>,
}

struct TimerHandle {
    id: u64,
    cancel_tx: tokio::sync::oneshot::Sender<()>,
}

impl TimersModule {
    pub fn new() -> Self {
        let mut exports = HashMap::new();
        
        exports.insert("setTimeout".to_string(), Value::Undefined);
        exports.insert("clearTimeout".to_string(), Value::Undefined);
        exports.insert("setInterval".to_string(), Value::Undefined);
        exports.insert("clearInterval".to_string(), Value::Undefined);
        exports.insert("setImmediate".to_string(), Value::Undefined);
        exports.insert("clearImmediate".to_string(), Value::Undefined);
        
        Self {
            exports,
            timers: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }
    
    pub fn set_timeout<F>(&self, callback: F, delay: u64) -> u64
    where
        F: FnOnce() + Send + 'static,
    {
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            let current_id = *next_id;
            *next_id += 1;
            current_id
        };
        
        let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();
        
        let timer_handle = TimerHandle {
            id,
            cancel_tx,
        };
        
        {
            let mut timers = self.timers.lock().unwrap();
            timers.insert(id, timer_handle);
        }
        
        let timers = Arc::clone(&self.timers);
        
        tokio::spawn(async move {
            tokio::select! {
                _ = sleep(Duration::from_millis(delay)) => {
                    // Timer expired, execute callback
                    callback();
                    
                    // Remove from active timers
                    let mut timers = timers.lock().unwrap();
                    timers.remove(&id);
                }
                _ = cancel_rx => {
                    // Timer was cancelled
                    let mut timers = timers.lock().unwrap();
                    timers.remove(&id);
                }
            }
        });
        
        id
    }
    
    pub fn clear_timeout(&self, id: u64) -> bool {
        let mut timers = self.timers.lock().unwrap();
        
        if let Some(timer) = timers.remove(&id) {
            let _ = timer.cancel_tx.send(());
            true
        } else {
            false
        }
    }
    
    pub fn set_interval<F>(&self, callback: F, delay: u64) -> u64
    where
        F: Fn() + Send + Sync + 'static,
    {
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            let current_id = *next_id;
            *next_id += 1;
            current_id
        };
        
        let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel();
        
        let timer_handle = TimerHandle {
            id,
            cancel_tx,
        };
        
        {
            let mut timers = self.timers.lock().unwrap();
            timers.insert(id, timer_handle);
        }
        
        let timers = Arc::clone(&self.timers);
        let callback = Arc::new(callback);
        
        tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_millis(delay));
            interval_timer.tick().await; // Skip first tick
            
            loop {
                tokio::select! {
                    _ = interval_timer.tick() => {
                        // Interval expired, execute callback
                        callback();
                    }
                    _ = &mut cancel_rx => {
                        // Interval was cancelled
                        let mut timers = timers.lock().unwrap();
                        timers.remove(&id);
                        break;
                    }
                }
            }
        });
        
        id
    }
    
    pub fn clear_interval(&self, id: u64) -> bool {
        self.clear_timeout(id)
    }
    
    pub fn set_immediate<F>(&self, callback: F) -> u64
    where
        F: FnOnce() + Send + 'static,
    {
        // setImmediate executes on next tick of event loop
        self.set_timeout(callback, 0)
    }
    
    pub fn clear_immediate(&self, id: u64) -> bool {
        self.clear_timeout(id)
    }
    
    pub fn active_timers(&self) -> usize {
        let timers = self.timers.lock().unwrap();
        timers.len()
    }
}

impl Module for TimersModule {
    fn name(&self) -> &str {
        "timers"
    }
    
    fn initialize(&mut self, runtime: &mut Runtime) -> Result<(), Box<dyn std::error::Error>> {
        // Set global timer functions
        runtime.set_global("setTimeout", Value::Undefined);
        runtime.set_global("clearTimeout", Value::Undefined);
        runtime.set_global("setInterval", Value::Undefined);
        runtime.set_global("clearInterval", Value::Undefined);
        runtime.set_global("setImmediate", Value::Undefined);
        runtime.set_global("clearImmediate", Value::Undefined);
        
        Ok(())
    }
    
    fn get_exports(&self) -> HashMap<String, Value> {
        self.exports.clone()
    }
}
