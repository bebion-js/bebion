//! File system module

use crate::{Module, Value};
use bebion_runtime::Runtime;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tokio::fs as async_fs;

pub struct FileSystemModule {
    exports: HashMap<String, Value>,
}

impl FileSystemModule {
    pub fn new() -> Self {
        let mut exports = HashMap::new();
        
        exports.insert("readFile".to_string(), Value::Undefined);
        exports.insert("writeFile".to_string(), Value::Undefined);
        exports.insert("readFileSync".to_string(), Value::Undefined);
        exports.insert("writeFileSync".to_string(), Value::Undefined);
        exports.insert("exists".to_string(), Value::Undefined);
        exports.insert("existsSync".to_string(), Value::Undefined);
        exports.insert("mkdir".to_string(), Value::Undefined);
        exports.insert("mkdirSync".to_string(), Value::Undefined);
        exports.insert("readdir".to_string(), Value::Undefined);
        exports.insert("readdirSync".to_string(), Value::Undefined);
        exports.insert("stat".to_string(), Value::Undefined);
        exports.insert("statSync".to_string(), Value::Undefined);
        exports.insert("unlink".to_string(), Value::Undefined);
        exports.insert("unlinkSync".to_string(), Value::Undefined);
        
        Self { exports }
    }
    
    pub fn read_file_sync(&self, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Ok(content)
    }
    
    pub fn write_file_sync(&self, path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        fs::write(path, content)?;
        Ok(())
    }
    
    pub fn exists_sync(&self, path: &str) -> bool {
        Path::new(path).exists()
    }
    
    pub fn mkdir_sync(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(path)?;
        Ok(())
    }
    
    pub fn readdir_sync(&self, path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let entries = fs::read_dir(path)?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    e.file_name().to_str().map(|s| s.to_string())
                })
            })
            .collect();
        
        Ok(entries)
    }
    
    pub fn stat_sync(&self, path: &str) -> Result<FileStats, Box<dyn std::error::Error>> {
        let metadata = fs::metadata(path)?;
        
        Ok(FileStats {
            size: metadata.len(),
            is_file: metadata.is_file(),
            is_directory: metadata.is_dir(),
            modified_time: metadata.modified()?
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        })
    }
    
    pub fn unlink_sync(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        fs::remove_file(path)?;
        Ok(())
    }
    
    pub async fn read_file(&self, path: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let content = async_fs::read_to_string(path).await?;
        Ok(content)
    }
    
    pub async fn write_file(&self, path: &str, content: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        async_fs::write(path, content).await?;
        Ok(())
    }
    
    pub async fn mkdir(&self, path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        async_fs::create_dir_all(path).await?;
        Ok(())
    }
    
    pub async fn readdir(&self, path: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut entries = async_fs::read_dir(path).await?;
        let mut result = Vec::new();
        
        while let Some(entry) = entries.next_entry().await? {
            if let Some(name) = entry.file_name().to_str() {
                result.push(name.to_string());
            }
        }
        
        Ok(result)
    }
    
    pub async fn stat(&self, path: &str) -> Result<FileStats, Box<dyn std::error::Error + Send + Sync>> {
        let metadata = async_fs::metadata(path).await?;
        
        Ok(FileStats {
            size: metadata.len(),
            is_file: metadata.is_file(),
            is_directory: metadata.is_dir(),
            modified_time: metadata.modified()?
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        })
    }
    
    pub async fn unlink(&self, path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        async_fs::remove_file(path).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileStats {
    pub size: u64,
    pub is_file: bool,
    pub is_directory: bool,
    pub modified_time: u64,
}

impl Module for FileSystemModule {
    fn name(&self) -> &str {
        "fs"
    }
    
    fn initialize(&mut self, _runtime: &mut Runtime) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    fn get_exports(&self) -> HashMap<String, Value> {
        self.exports.clone()
    }
}