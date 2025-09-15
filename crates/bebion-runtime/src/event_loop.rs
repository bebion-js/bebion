//! Event loop implementation for async/await and Promises

use crate::{RuntimeError, RuntimeResult};
use futures::future::{BoxFuture, Future};
use std::collections::{HashMap, VecDeque};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Handle;
use tracing::{debug, trace};

pub struct EventLoop {
    tasks: VecDeque<Task>,
    microtasks: VecDeque<Microtask>,
    timers: HashMap<u64, Timer>,
    next_timer_id: u64,
    running: bool,
    handle: Option<Handle>,
}

#[derive(Debug)]
struct Task {
    id: u64,
    future: BoxFuture<'static, ()>,
    created_at: Instant,
}

#[derive(Debug)]
struct Microtask {
    id: u64,
    callback: Box<dyn FnOnce() + Send>,
    created_at: Instant,
}

#[derive(Debug)]
struct Timer {
    id: u64,
    callback: Box<dyn FnOnce() + Send>,
    fire_at: Instant,
    interval: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct Promise {
    id: u64,
    state: Arc<Mutex<PromiseState>>,
}

#[derive(Debug)]
enum PromiseState {
    Pending {
        then_callbacks: Vec<Box<dyn FnOnce(PromiseResult) + Send>>,
    },
    Fulfilled(PromiseValue),
    Rejected(PromiseValue),
}

#[derive(Debug, Clone)]
pub enum PromiseValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
}

#[derive(Debug, Clone)]
pub enum PromiseResult {
    Ok(PromiseValue),
    Err(PromiseValue),
}

impl EventLoop {
    pub fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
            microtasks: VecDeque::new(),
            timers: HashMap::new(),
            next_timer_id: 1,
            running: false,
            handle: None,
        }
    }

    pub fn start(&mut self) {
        if self.running {
            return;
        }

        debug!("Starting event loop");
        self.running = true;

        // Try to get current tokio handle, or create a new runtime
        self.handle = Handle::try_current().ok();
    }

    pub fn stop(&mut self) {
        debug!("Stopping event loop");
        self.running = false;
        self.tasks.clear();
        self.microtasks.clear();
        self.timers.clear();
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn process_pending(&mut self) {
        if !self.running {
            return;
        }

        trace!("Processing pending tasks and microtasks");

        // Process all pending microtasks first
        while let Some(microtask) = self.microtasks.pop_front() {
            trace!("Executing microtask {}", microtask.id);
            (microtask.callback)();
        }

        // Process timers
        let now = Instant::now();
        let mut expired_timers = Vec::new();
        
        for (&timer_id, timer) in &self.timers {
            if now >= timer.fire_at {
                expired_timers.push(timer_id);
            }
        }

        for timer_id in expired_timers {
            if let Some(timer) = self.timers.remove(&timer_id) {
                trace!("Executing timer {}", timer.id);
                (timer.callback)();
                
                // Reschedule if it's an interval
                if let Some(interval) = timer.interval {
                    self.set_timer(interval, timer.callback);
                }
            }
        }

        // Process one task from the task queue
        if let Some(task) = self.tasks.pop_front() {
            trace!("Processing task {}", task.id);
            
            if let Some(handle) = &self.handle {
                handle.spawn(task.future);
            } else {
                // If no tokio runtime available, we can't execute async tasks
                debug!("No tokio runtime available for task execution");
            }
        }
    }

    pub fn queue_microtask<F>(&mut self, callback: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let microtask = Microtask {
            id: self.next_timer_id,
            callback: Box::new(callback),
            created_at: Instant::now(),
        };
        
        self.next_timer_id += 1;
        self.microtasks.push_back(microtask);
        
        trace!("Queued microtask {}", microtask.id);
    }

    pub fn set_timeout<F>(&mut self, callback: F, delay: Duration) -> u64
    where
        F: FnOnce() + Send + 'static,
    {
        let timer_id = self.next_timer_id;
        self.next_timer_id += 1;
        
        let timer = Timer {
            id: timer_id,
            callback: Box::new(callback),
            fire_at: Instant::now() + delay,
            interval: None,
        };
        
        self.timers.insert(timer_id, timer);
        trace!("Set timeout {} for {:?}", timer_id, delay);
        
        timer_id
    }

    pub fn set_interval<F>(&mut self, callback: F, interval: Duration) -> u64
    where
        F: FnOnce() + Send + 'static,
    {
        let timer_id = self.next_timer_id;
        self.next_timer_id += 1;
        
        let timer = Timer {
            id: timer_id,
            callback: Box::new(callback),
            fire_at: Instant::now() + interval,
            interval: Some(interval),
        };
        
        self.timers.insert(timer_id, timer);
        trace!("Set interval {} for {:?}", timer_id, interval);
        
        timer_id
    }

    pub fn set_timer<F>(&mut self, delay: Duration, callback: F) -> u64
    where
        F: FnOnce() + Send + 'static,
    {
        self.set_timeout(callback, delay)
    }

    pub fn clear_timeout(&mut self, timer_id: u64) -> bool {
        if let Some(timer) = self.timers.remove(&timer_id) {
            trace!("Cleared timeout {}", timer.id);
            true
        } else {
            false
        }
    }

    pub fn clear_interval(&mut self, timer_id: u64) -> bool {
        self.clear_timeout(timer_id)
    }

    pub fn spawn_task<F>(&mut self, future: F) -> u64
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task_id = self.next_timer_id;
        self.next_timer_id += 1;
        
        let task = Task {
            id: task_id,
            future: Box::pin(future),
            created_at: Instant::now(),
        };
        
        self.tasks.push_back(task);
        trace!("Spawned task {}", task_id);
        
        task_id
    }

    pub fn create_promise<F, T>(&mut self, executor: F) -> Promise
    where
        F: FnOnce(
            Box<dyn FnOnce(T) + Send>,
            Box<dyn FnOnce(T) + Send>,
        ) + Send + 'static,
        T: Into<PromiseValue> + Send + 'static,
    {
        let promise_id = self.next_timer_id;
        self.next_timer_id += 1;
        
        let state = Arc::new(Mutex::new(PromiseState::Pending {
            then_callbacks: Vec::new(),
        }));
        
        let promise = Promise {
            id: promise_id,
            state: Arc::clone(&state),
        };
        
        // Create resolve and reject callbacks
        let state_resolve = Arc::clone(&state);
        let state_reject = Arc::clone(&state);
        
        let resolve: Box<dyn FnOnce(T) + Send> = Box::new(move |value| {
            let mut state = state_resolve.lock().unwrap();
            if let PromiseState::Pending { then_callbacks } = 
                std::mem::replace(&mut *state, PromiseState::Fulfilled(value.into()))
            {
                for callback in then_callbacks {
                    callback(PromiseResult::Ok(value.into()));
                }
            }
        });
        
        let reject: Box<dyn FnOnce(T) + Send> = Box::new(move |value| {
            let mut state = state_reject.lock().unwrap();
            if let PromiseState::Pending { then_callbacks } = 
                std::mem::replace(&mut *state, PromiseState::Rejected(value.into()))
            {
                for callback in then_callbacks {
                    callback(PromiseResult::Err(value.into()));
                }
            }
        });
        
        // Execute the executor asynchronously
        self.queue_microtask(move || {
            executor(resolve, reject);
        });
        
        trace!("Created promise {}", promise_id);
        promise
    }

    pub fn stats(&self) -> EventLoopStats {
        EventLoopStats {
            pending_tasks: self.tasks.len(),
            pending_microtasks: self.microtasks.len(),
            active_timers: self.timers.len(),
            is_running: self.running,
        }
    }
}

impl Promise {
    pub fn then<F>(&self, callback: F)
    where
        F: FnOnce(PromiseResult) + Send + 'static,
    {
        let mut state = self.state.lock().unwrap();
        
        match &mut *state {
            PromiseState::Pending { then_callbacks } => {
                then_callbacks.push(Box::new(callback));
            }
            PromiseState::Fulfilled(value) => {
                callback(PromiseResult::Ok(value.clone()));
            }
            PromiseState::Rejected(value) => {
                callback(PromiseResult::Err(value.clone()));
            }
        }
    }

    pub fn is_resolved(&self) -> bool {
        let state = self.state.lock().unwrap();
        !matches!(*state, PromiseState::Pending { .. })
    }

    pub fn get_value(&self) -> Option<PromiseResult> {
        let state = self.state.lock().unwrap();
        match &*state {
            PromiseState::Fulfilled(value) => Some(PromiseResult::Ok(value.clone())),
            PromiseState::Rejected(value) => Some(PromiseResult::Err(value.clone())),
            PromiseState::Pending { .. } => None,
        }
    }
}

impl From<f64> for PromiseValue {
    fn from(n: f64) -> Self {
        PromiseValue::Number(n)
    }
}

impl From<String> for PromiseValue {
    fn from(s: String) -> Self {
        PromiseValue::String(s)
    }
}

impl From<&str> for PromiseValue {
    fn from(s: &str) -> Self {
        PromiseValue::String(s.to_string())
    }
}

impl From<bool> for PromiseValue {
    fn from(b: bool) -> Self {
        PromiseValue::Boolean(b)
    }
}

#[derive(Debug, Clone)]
pub struct EventLoopStats {
    pub pending_tasks: usize,
    pub pending_microtasks: usize,
    pub active_timers: usize,
    pub is_running: bool,
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}