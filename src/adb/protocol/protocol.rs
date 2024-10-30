use std::error::Error;
use std::path::Path;
use crate::adb::client::Client;
use crate::constants::{ADB_DEVICES_COMMAND, FAIL, OKAY, S_IFDIR, STAT_COMMAND, STAT_DATA_SIZE, USER_TRANSPORT_COMMAND};
use crate::enums::device_transport::DeviceTransport;
use crate::models::remote_metadata::RemoteMetadata;
use crate::models::stat_data::StatData;
use crate::utils::strip_adb_prefix;


impl Client {
    pub async fn send_transport(&mut self, device_transport: DeviceTransport) -> Result<(), Box<dyn Error>> {
        let transport_command = device_transport.get_device_transport();
        self.send_adb_command_and_check_if_fail(&transport_command, USER_TRANSPORT_COMMAND).await
    }

    pub async fn adb_devices(&mut self) -> Result<String, Box<dyn Error>> {
        self.send_adb_command(ADB_DEVICES_COMMAND).await?;
        let response = self.read_first_four_bytes_response().await?;
        if response == OKAY {
            let device_list_str = self.read_adb_full_response().await?;
            Ok(format!("List of devices attached\n{}", device_list_str))
        } else {
            let error_msg_str = self.read_adb_full_response().await?;
            println!("Received: {:?} Error message: {}", response, error_msg_str);
            Err("Failed to get devices list".into())
        }
    }

    pub async fn send_adb_command(&mut self, command: &str) -> Result<(), Box<dyn Error>> {
        let msg_len = format!("{:04x}", command.len());
        let adb_message = format!("{}{}", msg_len, command);
        self.send_command(adb_message.as_bytes()).await?;
        Ok(())
    }

    pub(crate) async fn get_remote_stat(&mut self, remote_path: &Path) -> Result<StatData, Box<dyn Error>> {
        let remote_path_str = remote_path.to_string_lossy();
        self.send_command(STAT_COMMAND.as_ref()).await?;
        self.send_command(&(remote_path_str.len() as u32).to_le_bytes()).await?;
        self.send_command(remote_path_str.as_bytes()).await?;

        let response = self.read_first_four_bytes_response().await?;

        if response == STAT_COMMAND {
            let stat_data = self.get_exact_bytes(STAT_DATA_SIZE).await?;
            let stat = StatData::from_bytes(&stat_data)?;
            Ok(stat)
        } else {
            Err(format!("ADB STAT error: Unexpected response: {}", response).into())
        }
    }

    pub async fn read_adb_full_response(&mut self) -> Result<String, Box<dyn Error>> {
        let prefix = self.read_exact_string(4).await?;
        let response = if let Ok(length) = usize::from_str_radix(&prefix, 16) {
            self.read_exact_string(length).await?
        } else {
            self.read_variable_length_response(prefix).await?
        };

        Ok(strip_adb_prefix(response))
    }

    pub async fn check_remote_path_is_directory(&mut self, remote_path: &str) -> Result<bool, Box<dyn Error>> {
        self.send_command(STAT_COMMAND.as_bytes()).await?;
        self.send_command(&((remote_path.len() + 1) as u32).to_le_bytes()).await?; // +1 for null terminator
        let remote_path_with_null = format!("{}\0", remote_path);
        self.send_command(remote_path_with_null.as_bytes()).await?;

        let response = self.read_first_four_bytes_response().await?;

        if response == STAT_COMMAND {
            let stat_data = self.get_exact_bytes(STAT_DATA_SIZE).await?;
            let stat = StatData::from_bytes(&stat_data)?;
            Ok((stat.mode() & S_IFDIR) != 0)
        } else {
            Ok(false)
        }
    }
    pub async fn get_remote_metadata(&mut self, remote_path: &str) -> Result<RemoteMetadata, Box<dyn Error>> {
        self.send_command(STAT_COMMAND.as_bytes()).await?;

        let path_with_null = format!("{}\0", remote_path);
        let path_len = (path_with_null.len()) as u32;
        self.send_command(&path_len.to_le_bytes()).await?;

        self.send_command(path_with_null.as_bytes()).await?;

        let response = self.get_exact_bytes(4).await?;
        let response_str = std::str::from_utf8(&response)?;

        if response_str != STAT_COMMAND {
            return Err(format!("Unexpected response for STAT_COMMAND: {}", response_str).into());
        }

        let stat_data_bytes = self.get_exact_bytes(12).await?;
        let stat = StatData::from_bytes(&stat_data_bytes)?;

        Ok(RemoteMetadata {
            mode: stat.mode(),
            mtime: stat.mtime(),
        })
    }
    pub async fn send_adb_command_and_check_if_fail(&mut self, command: &str, debug_command: &str) -> Result<(), Box<dyn Error>> {
        self.send_adb_command(&command).await?;
        let response = self.read_first_four_bytes_response().await?;
        match response.as_str() {
            OKAY => {
                Ok(())
            }
            FAIL => {
                let error_msg_str = self.read_adb_full_response().await?;
                return Err(format!("{}", error_msg_str).into());
            }
            _ => {
                let error_msg_str = self.read_adb_full_response().await?;
                return Err(format!("Failed send {} command : {}", debug_command, error_msg_str).into());
            }
        }
    }
    pub async fn send_adb_and_return_response(&mut self, command: &str, debug_command: &str) -> Result<String, Box<dyn Error>> {
        self.send_adb_command(&command).await?;

        let response = self.read_first_four_bytes_response().await?;
        if response == OKAY {
            let response = self.read_adb_full_response().await?;
            Ok(response)
        } else if response == FAIL {
            let error_msg_str = self.read_adb_full_response().await?;
            Err(format!("Failed to send {} command: {}", debug_command, error_msg_str).into())
        } else {
            Err(format!("Unexpected response: {}", response).into())
        }
    }

    pub async fn send_adb_command_with_extended_response(&mut self, command: &str, debug_command: &str) -> Result<String, Box<dyn Error>> {
        self.send_adb_command(&command).await?;
        let response = self.read_first_four_bytes_response().await?;
        if response == OKAY {
            if self.has_more_data().await? {
                let next_response = self.read_first_four_bytes_response().await?;
                match next_response.as_str() {
                    OKAY => {
                        let mut response_data = String::new();
                        if self.has_more_data().await? {
                            response_data = self.read_adb_full_response().await?;
                        }
                        Ok(response_data)
                    }
                    FAIL => {
                        let error_msg_str = self.read_adb_full_response().await?;
                        Err(format!("{}", error_msg_str).into())
                    }
                    _ => {
                        let error_msg_str = self.read_adb_full_response().await?;
                        Err(format!("Failed to send {} command: {}", debug_command, error_msg_str).into())
                    }
                }
            } else {
                Ok(String::new())
            }
        } else if response == FAIL {
            let error_msg_str = self.read_adb_full_response().await?;
            Err(format!("{}", error_msg_str).into())
        } else {
            let error_msg_str = self.read_adb_full_response().await?;
            Err(format!("Failed to send {} command: {}", debug_command, error_msg_str).into())
        }
    }
}
