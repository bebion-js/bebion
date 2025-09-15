//! Bebion Standard Library
//! 
//! Built-in modules providing filesystem, networking, crypto, and other APIs.

pub mod console;
pub mod crypto;
pub mod fs;
pub mod http;
pub mod net;
pub mod process;
pub mod timers;
pub mod url;
pub mod util;

use bebion_runtime::{Runtime, Value};
use std::collections::HashMap;

pub struct StandardLibrary {
    modules: HashMap<String, Box<dyn Module>>,
}

pub trait Module: Send + Sync {
    fn name(&self) -> &str;
    fn initialize(&mut self, runtime: &mut Runtime) -> Result<(), Box<dyn std::error::Error>>;
    fn get_exports(&self) -> HashMap<String, Value>;
}

impl StandardLibrary {
    pub fn new() -> Self {
        let mut stdlib = Self {
            modules: HashMap::new(),
        };
        
        // Register built-in modules
        stdlib.register_module(Box::new(console::ConsoleModule::new()));
        stdlib.register_module(Box::new(crypto::CryptoModule::new()));
        stdlib.register_module(Box::new(fs::FileSystemModule::new()));
        stdlib.register_module(Box::new(http::HttpModule::new()));
        stdlib.register_module(Box::new(net::NetworkModule::new()));
        stdlib.register_module(Box::new(process::ProcessModule::new()));
        stdlib.register_module(Box::new(timers::TimersModule::new()));
        stdlib.register_module(Box::new(url::UrlModule::new()));
        stdlib.register_module(Box::new(util::UtilModule::new()));
        
        stdlib
    }
    
    pub fn register_module(&mut self, module: Box<dyn Module>) {
        let name = module.name().to_string();
        self.modules.insert(name, module);
    }
    
    pub fn initialize_all(&mut self, runtime: &mut Runtime) -> Result<(), Box<dyn std::error::Error>> {
        for module in self.modules.values_mut() {
            module.initialize(runtime)?;
        }
        Ok(())
    }
    
    pub fn get_module_exports(&self, name: &str) -> Option<HashMap<String, Value>> {
        self.modules.get(name).map(|module| module.get_exports())
    }
    
    pub fn get_all_exports(&self) -> HashMap<String, HashMap<String, Value>> {
        let mut all_exports = HashMap::new();
        
        for (name, module) in &self.modules {
            all_exports.insert(name.clone(), module.get_exports());
        }
        
        all_exports
    }
}

impl Default for StandardLibrary {
    fn default() -> Self {
        Self::new()
    }
}