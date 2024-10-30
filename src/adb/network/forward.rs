use std::error::Error;
use crate::adb::client::Client;
use crate::constants::{FAIL, HOST_FORWARD_COMMAND, HOST_FORWARD_KILL_ALL_COMMAND, HOST_FORWARD_KILL_COMMAND, HOST_FORWARD_LIST_COMMAND, NO_REBIND_OPTION, OKAY};
use crate::enums::device_transport::DeviceTransport;


impl Client {
    pub async fn send_forward_command_set(&mut self, device_transport: DeviceTransport, local: &str, remote: &str, no_rebind: bool) -> Result<(), Box<dyn Error>> {
        self.send_transport(device_transport).await?;

        let forward_message = if no_rebind {
            format!("{}:{}:{};{}", HOST_FORWARD_COMMAND, NO_REBIND_OPTION, local, remote)
        } else {
            format!("{}:{};{}", HOST_FORWARD_COMMAND, local, remote)
        };
        self.send_adb_command_and_check_if_fail(&forward_message, &forward_message).await
    }

    pub async fn send_forward_command_remove(&mut self, device_transport: DeviceTransport, local: &str) -> Result<(), Box<dyn Error>> {
        self.send_transport(device_transport).await?;

        let forward_message = format!("{}:{}", HOST_FORWARD_KILL_COMMAND, local);
        self.send_adb_command_and_check_if_fail(&forward_message, &forward_message).await
    }

    pub async fn send_forward_command_remove_all(&mut self, device_transport: DeviceTransport) -> Result<(), Box<dyn Error>> {
        self.send_transport(device_transport).await?;

        let forward_message = format!("{}", HOST_FORWARD_KILL_ALL_COMMAND);
        self.send_adb_command_and_check_if_fail(&forward_message, &forward_message).await
    }


    pub async fn send_forward_command_list(&mut self, device_transport: DeviceTransport) -> Result<String, Box<dyn Error>> {
        self.send_transport(device_transport).await?;

        let forward_message = format!("{}", HOST_FORWARD_LIST_COMMAND);
        self.send_adb_command(&forward_message).await?;

        let response = self.read_first_four_bytes_response().await?;
        if response == OKAY {
            let list = self.read_adb_full_response().await?;
            Ok(list)
        } else if response == FAIL {
            let error_msg_str = self.read_adb_full_response().await?;
            Err(format!("Failed to list forward connections: {}", error_msg_str).into())
        } else {
            Err(format!("Unexpected response: {}", response).into())
        }
    }
}