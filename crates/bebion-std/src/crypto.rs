//! Cryptographic functions module

use crate::{Module, Value};
use bebion_runtime::Runtime;
use base64;
use rand::{thread_rng, Rng};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub struct CryptoModule {
    exports: HashMap<String, Value>,
}

impl CryptoModule {
    pub fn new() -> Self {
        let mut exports = HashMap::new();
        
        exports.insert("randomBytes".to_string(), Value::Undefined);
        exports.insert("randomUUID".to_string(), Value::Undefined);
        exports.insert("hash".to_string(), Value::Undefined);
        exports.insert("sha256".to_string(), Value::Undefined);
        exports.insert("base64Encode".to_string(), Value::Undefined);
        exports.insert("base64Decode".to_string(), Value::Undefined);
        
        Self { exports }
    }
    
    pub fn random_bytes(&self, size: usize) -> Vec<u8> {
        let mut rng = thread_rng();
        (0..size).map(|_| rng.gen::<u8>()).collect()
    }
    
    pub fn random_uuid(&self) -> String {
        let mut rng = thread_rng();
        
        // Generate 16 random bytes
        let mut bytes = [0u8; 16];
        rng.fill(&mut bytes);
        
        // Set version (4) and variant bits
        bytes[6] = (bytes[6] & 0x0f) | 0x40; // Version 4
        bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant 10
        
        // Format as UUID string
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5],
            bytes[6], bytes[7],
            bytes[8], bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        )
    }
    
    pub fn sha256(&self, data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }
    
    pub fn hash(&self, algorithm: &str, data: &str) -> Result<String, Box<dyn std::error::Error>> {
        match algorithm {
            "sha256" => Ok(self.sha256(data)),
            _ => Err(format!("Unsupported hash algorithm: {}", algorithm).into()),
        }
    }
    
    pub fn base64_encode(&self, data: &[u8]) -> String {
        base64::encode(data)
    }
    
    pub fn base64_decode(&self, data: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(base64::decode(data)?)
    }
    
    pub fn random_int(&self, min: i32, max: i32) -> i32 {
        let mut rng = thread_rng();
        rng.gen_range(min..=max)
    }
    
    pub fn random_float(&self) -> f64 {
        let mut rng = thread_rng();
        rng.gen::<f64>()
    }
}

impl Module for CryptoModule {
    fn name(&self) -> &str {
        "crypto"
    }
    
    fn initialize(&mut self, _runtime: &mut Runtime) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    fn get_exports(&self) -> HashMap<String, Value> {
        self.exports.clone()
    }
}