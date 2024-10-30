use std::{env};
use std::error::Error;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use std::io::{ErrorKind};
use tokio::io::{AsyncWriteExt};
use crate::constants::{DEFAULT_ADB_SERVER_IP, DEFAULT_ADB_SERVER_PORT, ADB_ADDRESS_ENV, ADB_SERVER_CONNECT_TIMEOUT_SECONDS_DURATION};

pub struct Client {
    pub adb_stream: TcpStream,
    server_address: Option<String>,
    server_port: Option<u16>,
}

impl Client {
    pub async fn new(server_address: Option<String>, server_port: Option<u16>) -> Result<Self, Box<dyn Error>> {
        let (adb_stream, _adb_server_addr) = Self::connect(server_address.clone(), server_port).await?;

        Ok(Client {
            adb_stream,
            server_address,
            server_port,
        })
    }

    async fn connect(server_address: Option<String>, server_port: Option<u16>) -> Result<(TcpStream, String), Box<dyn Error>> {
        let adb_server_addr = match (server_address, server_port) {
            (Some(addr), Some(port)) => format!("{}:{}", addr, port),
            (Some(addr), None) => format!("{}:{}", addr, DEFAULT_ADB_SERVER_PORT),
            (None, Some(port)) => format!("{}:{}", DEFAULT_ADB_SERVER_IP, port),
            (None, None) => env::var(ADB_ADDRESS_ENV).unwrap_or_else(|_| {
                format!("{}:{}", DEFAULT_ADB_SERVER_IP, DEFAULT_ADB_SERVER_PORT)
            }),
        };

        let change_env_message = format!(
            "You can change the ADB server address by setting the {} environment variable (e.g., export {}=127.0.0.1:5037)",
            ADB_ADDRESS_ENV, ADB_ADDRESS_ENV
        );

        let adb_stream = match timeout(
            Duration::from_secs(ADB_SERVER_CONNECT_TIMEOUT_SECONDS_DURATION),
            TcpStream::connect(&adb_server_addr),
        ).await {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => return Err(format!(
                "Failed to connect to ADB server at address {}: {}. {}",
                adb_server_addr, e, change_env_message
            ).into()),
            Err(_) => return Err(format!(
                "Connection attempt to ADB server at address {} timed out. {}",
                adb_server_addr, change_env_message
            ).into()),
        };

        Ok((adb_stream, adb_server_addr))
    }

    pub async fn reconnect(&mut self) -> Result<(), Box<dyn Error>> {
        self.close().await;
        let (new_stream, _) = Self::connect(self.server_address.clone(), self.server_port).await?;
        self.adb_stream = new_stream;
        Ok(())
    }

    pub async fn close(&mut self) {
        match self.adb_stream.shutdown().await {
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::NotConnected => {}
            Err(e) => {
                eprintln!("Error shutting down ADB stream: {}", e);
            }
        }
    }
    pub async fn is_connected(&self) -> bool {
        let mut buf = [0; 1];
        match self.adb_stream.try_read(&mut buf) {
            Ok(0) => false,
            Ok(_) => true,
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => true,
            Err(_) => false,
        }
    }

}