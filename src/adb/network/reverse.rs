use std::error::Error;
use crate::adb::client::Client;
use crate::constants::{HOST_REVERSE_COMMAND, HOST_REVERSE_LIST_COMMAND, HOST_REVERSE_REMOVE_COMMAND, HOST_REVERSE_REMOVE_ALL_COMMAND, NO_REBIND_OPTION};
use crate::enums::device_transport::DeviceTransport;

impl Client {
    pub async fn send_reverse_command_set(&mut self, device_transport: DeviceTransport, remote: &str, local: &str, no_rebind: bool, ) -> Result<String, Box<dyn Error>> {
        self.send_transport(device_transport).await?;

        let reverse_message = if no_rebind {
            format!("{}:{}:{};{}", HOST_REVERSE_COMMAND, NO_REBIND_OPTION, remote, local)
        } else {
            format!("{}:{};{}", HOST_REVERSE_COMMAND, remote, local)
        };

        self.send_adb_command_with_extended_response(&reverse_message, &reverse_message).await
    }


    pub async fn send_reverse_command_remove(&mut self, device_transport: DeviceTransport, remote: &str, ) -> Result<String, Box<dyn Error>> {
        self.send_transport(device_transport).await?;

        let reverse_message = format!("{}:{}", HOST_REVERSE_REMOVE_COMMAND, remote);
        self.send_adb_command_with_extended_response(&reverse_message, &reverse_message).await
    }

    pub async fn send_reverse_command_remove_all(&mut self, device_transport: DeviceTransport, ) -> Result<String, Box<dyn Error>> {
        self.send_transport(device_transport).await?;

        let reverse_message = format!("{}", HOST_REVERSE_REMOVE_ALL_COMMAND);
        self.send_adb_command_with_extended_response(&reverse_message, &reverse_message).await
    }

    pub async fn send_reverse_command_list(&mut self, device_transport: DeviceTransport, ) -> Result<String, Box<dyn Error>> {
        self.send_transport(device_transport).await?;
        let reverse_message = format!("{}", HOST_REVERSE_LIST_COMMAND);
        self.send_adb_and_return_response(&reverse_message,&reverse_message).await
    }


}