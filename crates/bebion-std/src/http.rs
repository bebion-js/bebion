//! HTTP client and server module

use crate::{Module, Value};
use bebion_runtime::Runtime;
use reqwest;
use serde_json;
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct HttpModule {
    exports: HashMap<String, Value>,
}

impl HttpModule {
    pub fn new() -> Self {
        let mut exports = HashMap::new();
        
        exports.insert("get".to_string(), Value::Undefined);
        exports.insert("post".to_string(), Value::Undefined);
        exports.insert("put".to_string(), Value::Undefined);
        exports.insert("delete".to_string(), Value::Undefined);
        exports.insert("request".to_string(), Value::Undefined);
        exports.insert("createServer".to_string(), Value::Undefined);
        
        Self { exports }
    }
    
    pub async fn get(&self, url: &str, headers: Option<HashMap<String, String>>) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let mut request = client.get(url);
        
        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(&key, &value);
            }
        }
        
        let response = request.send().await?;
        
        let status = response.status().as_u16();
        let headers: HashMap<String, String> = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        
        let body = response.text().await?;
        
        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }
    
    pub async fn post(&self, url: &str, data: Option<String>, headers: Option<HashMap<String, String>>) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let mut request = client.post(url);
        
        if let Some(data) = data {
            request = request.body(data);
        }
        
        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(&key, &value);
            }
        }
        
        let response = request.send().await?;
        
        let status = response.status().as_u16();
        let headers: HashMap<String, String> = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        
        let body = response.text().await?;
        
        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }
    
    pub async fn put(&self, url: &str, data: Option<String>, headers: Option<HashMap<String, String>>) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let mut request = client.put(url);
        
        if let Some(data) = data {
            request = request.body(data);
        }
        
        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(&key, &value);
            }
        }
        
        let response = request.send().await?;
        
        let status = response.status().as_u16();
        let headers: HashMap<String, String> = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        
        let body = response.text().await?;
        
        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }
    
    pub async fn delete(&self, url: &str, headers: Option<HashMap<String, String>>) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let mut request = client.delete(url);
        
        if let Some(headers) = headers {
            for (key, value) in headers {
                request = request.header(&key, &value);
            }
        }
        
        let response = request.send().await?;
        
        let status = response.status().as_u16();
        let headers: HashMap<String, String> = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        
        let body = response.text().await?;
        
        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }
    
    pub async fn create_server<F>(&self, port: u16, handler: F) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn(HttpRequest) -> HttpResponse + Send + Sync + 'static,
    {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        println!("HTTP server listening on port {}", port);
        
        loop {
            let (stream, _) = listener.accept().await?;
            let handler = &handler;
            
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, handler).await {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }
    }
    
    async fn handle_connection<F>(
        mut stream: TcpStream,
        handler: &F,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn(HttpRequest) -> HttpResponse,
    {
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).await?;
        
        let request_str = String::from_utf8_lossy(&buffer[..n]);
        let request = Self::parse_request(&request_str)?;
        
        let response = handler(request);
        
        let response_str = format!(
            "HTTP/1.1 {} OK\r\nContent-Length: {}\r\n\r\n{}",
            response.status,
            response.body.len(),
            response.body
        );
        
        stream.write_all(response_str.as_bytes()).await?;
        stream.flush().await?;
        
        Ok(())
    }
    
    fn parse_request(request_str: &str) -> Result<HttpRequest, Box<dyn std::error::Error + Send + Sync>> {
        let lines: Vec<&str> = request_str.split("\r\n").collect();
        
        if lines.is_empty() {
            return Err("Invalid request".into());
        }
        
        let request_line: Vec<&str> = lines[0].split_whitespace().collect();
        if request_line.len() < 3 {
            return Err("Invalid request line".into());
        }
        
        let method = request_line[0].to_string();
        let path = request_line[1].to_string();
        
        let mut headers = HashMap::new();
        let mut i = 1;
        
        while i < lines.len() && !lines[i].is_empty() {
            if let Some(colon_pos) = lines[i].find(':') {
                let key = lines[i][..colon_pos].trim().to_string();
                let value = lines[i][colon_pos + 1..].trim().to_string();
                headers.insert(key, value);
            }
            i += 1;
        }
        
        // Body would be after empty line
        let body = if i + 1 < lines.len() {
            lines[i + 1..].join("\r\n")
        } else {
            String::new()
        };
        
        Ok(HttpRequest {
            method,
            path,
            headers,
            body,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Module for HttpModule {
    fn name(&self) -> &str {
        "http"
    }
    
    fn initialize(&mut self, _runtime: &mut Runtime) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    fn get_exports(&self) -> HashMap<String, Value> {
        self.exports.clone()
    }
}