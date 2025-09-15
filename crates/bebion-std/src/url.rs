//! URL parsing and manipulation module

use crate::{Module, Value};
use bebion_runtime::Runtime;
use std::collections::HashMap;

pub struct UrlModule {
    exports: HashMap<String, Value>,
}

impl UrlModule {
    pub fn new() -> Self {
        let mut exports = HashMap::new();
        
        exports.insert("parse".to_string(), Value::Undefined);
        exports.insert("format".to_string(), Value::Undefined);
        exports.insert("resolve".to_string(), Value::Undefined);
        
        Self { exports }
    }
    
    pub fn parse(&self, url_str: &str) -> Result<ParsedUrl, Box<dyn std::error::Error>> {
        // Simple URL parsing implementation
        let url = url_str;
        
        // Extract protocol
        let (protocol, remaining) = if let Some(pos) = url.find("://") {
            let protocol = &url[..pos];
            let remaining = &url[pos + 3..];
            (Some(protocol.to_string()), remaining)
        } else {
            (None, url)
        };
        
        // Extract hostname and path
        let (hostname, path) = if let Some(pos) = remaining.find('/') {
            let hostname = &remaining[..pos];
            let path = &remaining[pos..];
            (hostname, path)
        } else {
            (remaining, "/")
        };
        
        // Extract port from hostname
        let (hostname, port) = if let Some(pos) = hostname.find(':') {
            let host = &hostname[..pos];
            let port_str = &hostname[pos + 1..];
            let port = port_str.parse::<u16>().ok();
            (host.to_string(), port)
        } else {
            (hostname.to_string(), None)
        };
        
        // Extract query and hash from path
        let (pathname, query, hash) = {
            let mut current_path = path;
            
            // Extract hash
            let (path_without_hash, hash) = if let Some(pos) = current_path.find('#') {
                let path = &current_path[..pos];
                let hash = &current_path[pos + 1..];
                (path, Some(hash.to_string()))
            } else {
                (current_path, None)
            };
            
            // Extract query
            let (pathname, query) = if let Some(pos) = path_without_hash.find('?') {
                let path = &path_without_hash[..pos];
                let query = &path_without_hash[pos + 1..];
                (path.to_string(), Some(query.to_string()))
            } else {
                (path_without_hash.to_string(), None)
            };
            
            (pathname, query, hash)
        };
        
        Ok(ParsedUrl {
            protocol,
            hostname,
            port,
            pathname,
            query,
            hash,
            href: url_str.to_string(),
        })
    }
    
    pub fn format(&self, url: &ParsedUrl) -> String {
        let mut result = String::new();
        
        if let Some(protocol) = &url.protocol {
            result.push_str(protocol);
            result.push_str("://");
        }
        
        result.push_str(&url.hostname);
        
        if let Some(port) = url.port {
            result.push(':');
            result.push_str(&port.to_string());
        }
        
        result.push_str(&url.pathname);
        
        if let Some(query) = &url.query {
            result.push('?');
            result.push_str(query);
        }
        
        if let Some(hash) = &url.hash {
            result.push('#');
            result.push_str(hash);
        }
        
        result
    }
    
    pub fn resolve(&self, base: &str, relative: &str) -> Result<String, Box<dyn std::error::Error>> {
        let base_url = self.parse(base)?;
        
        // Simple resolution - in a full implementation this would be more complex
        if relative.starts_with("http://") || relative.starts_with("https://") {
            Ok(relative.to_string())
        } else if relative.starts_with('/') {
            let mut result = String::new();
            if let Some(protocol) = &base_url.protocol {
                result.push_str(protocol);
                result.push_str("://");
            }
            result.push_str(&base_url.hostname);
            if let Some(port) = base_url.port {
                result.push(':');
                result.push_str(&port.to_string());
            }
            result.push_str(relative);
            Ok(result)
        } else {
            // Relative to current path
            let base_path = if base_url.pathname.ends_with('/') {
                &base_url.pathname
            } else {
                // Remove filename from path
                if let Some(pos) = base_url.pathname.rfind('/') {
                    &base_url.pathname[..=pos]
                } else {
                    "/"
                }
            };
            
            let mut result = String::new();
            if let Some(protocol) = &base_url.protocol {
                result.push_str(protocol);
                result.push_str("://");
            }
            result.push_str(&base_url.hostname);
            if let Some(port) = base_url.port {
                result.push(':');
                result.push_str(&port.to_string());
            }
            result.push_str(base_path);
            result.push_str(relative);
            Ok(result)
        }
    }
    
    pub fn parse_query(&self, query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        
        for pair in query.split('&') {
            if let Some(pos) = pair.find('=') {
                let key = &pair[..pos];
                let value = &pair[pos + 1..];
                params.insert(
                    urlencoding::decode(key).unwrap_or_else(|_| key.into()).into(),
                    urlencoding::decode(value).unwrap_or_else(|_| value.into()).into(),
                );
            } else {
                params.insert(
                    urlencoding::decode(pair).unwrap_or_else(|_| pair.into()).into(),
                    String::new(),
                );
            }
        }
        
        params
    }
}

#[derive(Debug, Clone)]
pub struct ParsedUrl {
    pub protocol: Option<String>,
    pub hostname: String,
    pub port: Option<u16>,
    pub pathname: String,
    pub query: Option<String>,
    pub hash: Option<String>,
    pub href: String,
}

impl Module for UrlModule {
    fn name(&self) -> &str {
        "url"
    }
    
    fn initialize(&mut self, _runtime: &mut Runtime) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    fn get_exports(&self) -> HashMap<String, Value> {
        self.exports.clone()
    }
}

// Simple URL encoding implementation
mod urlencoding {
    use std::borrow::Cow;
    
    pub fn decode(input: &str) -> Result<Cow<str>, Box<dyn std::error::Error>> {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '%' {
                // Decode percent-encoded character
                let hex1 = chars.next().ok_or("Invalid percent encoding")?;
                let hex2 = chars.next().ok_or("Invalid percent encoding")?;
                
                let hex_str = format!("{}{}", hex1, hex2);
                let byte = u8::from_str_radix(&hex_str, 16)
                    .map_err(|_| "Invalid hex in percent encoding")?;
                
                result.push(byte as char);
            } else if ch == '+' {
                result.push(' ');
            } else {
                result.push(ch);
            }
        }
        
        if result == input {
            Ok(Cow::Borrowed(input))
        } else {
            Ok(Cow::Owned(result))
        }
    }
}