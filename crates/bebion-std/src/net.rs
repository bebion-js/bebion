//! Network module for TCP and UDP

use crate::{Module, Value};
use bebion_runtime::Runtime;
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};

pub struct NetworkModule {
    exports: HashMap<String, Value>,
}

impl NetworkModule {
    pub fn new() -> Self {
        let mut exports = HashMap::new();
        
        exports.insert("createTcpServer".to_string(), Value::Undefined);
        exports.insert("connectTcp".to_string(), Value::Undefined);
        exports.insert("createUdpSocket".to_string(), Value::Undefined);
        
        Self { exports }
    }
    
    pub async fn create_tcp_server<F>(&self, port: u16, handler: F) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn(TcpConnection) + Send + Sync + Clone + 'static,
    {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        println!("TCP server listening on port {}", port);
        
        loop {
            let (stream, addr) = listener.accept().await?;
            let handler = handler.clone();
            
            tokio::spawn(async move {
                let connection = TcpConnection::new(stream, addr.to_string());
                handler(connection);
            });
        }
    }
    
    pub async fn connect_tcp(&self, address: &str) -> Result<TcpConnection, Box<dyn std::error::Error + Send + Sync>> {
        let stream = TcpStream::connect(address).await?;
        Ok(TcpConnection::new(stream, address.to_string()))
    }
    
    pub async fn create_udp_socket(&self, address: &str) -> Result<UdpConnection, Box<dyn std::error::Error + Send + Sync>> {
        let socket = UdpSocket::bind(address).await?;
        Ok(UdpConnection::new(socket))
    }
}

pub struct TcpConnection {
    stream: TcpStream,
    remote_address: String,
}

impl TcpConnection {
    pub fn new(stream: TcpStream, remote_address: String) -> Self {
        Self {
            stream,
            remote_address,
        }
    }
    
    pub fn remote_address(&self) -> &str {
        &self.remote_address
    }
    
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.stream.read(buffer).await?)
    }
    
    pub async fn write(&mut self, data: &[u8]) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.stream.write(data).await?)
    }
    
    pub async fn flush(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.stream.flush().await?;
        Ok(())
    }
    
    pub async fn read_line(&mut self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut buffer = Vec::new();
        let mut byte = [0u8; 1];
        
        loop {
            self.stream.read_exact(&mut byte).await?;
            
            if byte[0] == b'\n' {
                break;
            }
            
            if byte[0] != b'\r' {
                buffer.push(byte[0]);
            }
        }
        
        Ok(String::from_utf8(buffer)?)
    }
    
    pub async fn write_line(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.stream.write_all(line.as_bytes()).await?;
        self.stream.write_all(b"\r\n").await?;
        self.stream.flush().await?;
        Ok(())
    }
}

pub struct UdpConnection {
    socket: UdpSocket,
}

impl UdpConnection {
    pub fn new(socket: UdpSocket) -> Self {
        Self { socket }
    }
    
    pub async fn send_to(&self, data: &[u8], address: &str) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.socket.send_to(data, address).await?)
    }
    
    pub async fn recv_from(&self, buffer: &mut [u8]) -> Result<(usize, String), Box<dyn std::error::Error + Send + Sync>> {
        let (size, addr) = self.socket.recv_from(buffer).await?;
        Ok((size, addr.to_string()))
    }
    
    pub fn local_address(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.socket.local_addr()?.to_string())
    }
}

impl Module for NetworkModule {
    fn name(&self) -> &str {
        "net"
    }
    
    fn initialize(&mut self, _runtime: &mut Runtime) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    fn get_exports(&self) -> HashMap<String, Value> {
        self.exports.clone()
    }
}