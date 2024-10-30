use std::error::Error;
use std::io::ErrorKind;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::adb::client::Client;
use std::io::{Write};

impl Client {
    pub async fn send_command(&mut self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        self.adb_stream.write_all(data).await?;
        Ok(())
    }

    pub(crate) async fn read_variable_length_response(&mut self, prefix: String) -> Result<String, Box<dyn Error>> {
        let mut data = prefix.into_bytes();
        let mut temp_buffer = [0u8; 1024];

        loop {
            match self.adb_stream.read(&mut temp_buffer).await {
                Ok(0) => break,
                Ok(n) => data.extend_from_slice(&temp_buffer[..n]),
                Err(err) if err.kind() == ErrorKind::Interrupted => continue,
                Err(err) => return Err(err.into()),
            }
        }

        Ok(String::from_utf8(data)?)
    }

    pub async fn read_first_four_bytes_response(&mut self) -> Result<String, Box<dyn Error>> {
        self.read_exact_string(4).await
    }


    pub(crate) async fn read_exact_string(&mut self, length: usize) -> Result<String, Box<dyn Error>> {
        if length == 0 {
            return Ok(String::new());
        }

        let mut data = vec![0u8; length];
        match self.adb_stream.read_exact(&mut data).await {
            Ok(_) => Ok(String::from_utf8(data)?),
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Ok(String::new()),
            Err(e) => Err(Box::new(e))
        }
    }

    pub async fn get_exact_bytes(&mut self, num_bytes: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut buffer = vec![0u8; num_bytes];
        self.adb_stream.read_exact(&mut buffer).await.map_err(|e| Box::new(e) as Box<dyn Error>)?;
        Ok(buffer)
    }

    pub async fn read_all_data(&mut self) -> Result<String, Box<dyn Error>> {
        let mut buffer = Vec::new();
        let mut temp_buffer = [0u8; 1024];

        loop {
            match self.adb_stream.read(&mut temp_buffer).await {
                Ok(0) => break,
                Ok(n) => buffer.extend_from_slice(&temp_buffer[..n]),
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            }
        }

        String::from_utf8(buffer).map_err(|e| e.into())
    }


    pub async fn read_and_print_data(&mut self) -> Result<(), Box<dyn Error>> {
        let mut temp_buffer = [0u8; 1024];

        loop {
            match self.adb_stream.read(&mut temp_buffer).await {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&temp_buffer[..n]);
                    print!("{}", chunk);
                    std::io::stdout().flush()?;
                },
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            }
        }

        Ok(())
    }

    pub async fn has_more_data(&mut self) -> Result<bool, Box<dyn Error>> {
        let mut buf = [0u8; 1];
        let bytes_read = self.adb_stream.peek(&mut buf).await?;
        Ok(bytes_read > 0)
    }

}
