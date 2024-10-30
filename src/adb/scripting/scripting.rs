use std::error::Error;
use crate::adb::client::Client;
use crate::constants::{HOST_GET_DEVPATH_COMMAND, REBOOT_BOOTLOADER, HOST_SERIALNO_COMMAND, REBOOT_RECOVERY, REBOOT_SIDELOAD, REBOOT_SIDELOAD_AUTO_REBOOT, OKAY, ADB_REBOOT_BOOTLOADER_COMMAND, ADB_REBOOT_RECOVERY_COMMAND, ADB_REBOOT_SIDELOAD_COMMAND, ADB_REBOOT_SIDELOAD_AUTO_REBOOT_COMMAND, ADB_REBOOT_COMMAND, ADB_ROOT_COMMAND, ADB_UNROOT_COMMAND, ADB_REMOUNT_COMMAND, ADB_USB_COMMAND, ADB_TCPIP_COMMAND, ADB_GET_STATE_COMMAND};
use crate::enums::device_transport::DeviceTransport;
use tokio::time::{Duration, Instant, sleep};

const WAIT_FOR_STATE_POLL_INTERVAL_SEC: u64 = 1;

impl Client {
    pub async fn adb_wait_for(&mut self, device_transport: DeviceTransport, desired_state: &str, timeout_duration: Option<Duration>) -> Result<(), Box<dyn Error>> {
        let start_time = Instant::now();

        loop {
            if let Some(timeout) = timeout_duration {
                if Instant::now().duration_since(start_time) >= timeout {
                    return Err(format!(
                        "Timeout while waiting for device to reach '{}' state",
                        desired_state
                    ).into());
                }
            }

            match self.adb_get_state(device_transport.clone()).await {
                Ok(current_state) => {
                    if current_state == desired_state {
                        return Ok(());
                    } else {
                        sleep(Duration::from_secs(WAIT_FOR_STATE_POLL_INTERVAL_SEC)).await;
                    }
                }
                Err(_) => {
                    sleep(Duration::from_secs(WAIT_FOR_STATE_POLL_INTERVAL_SEC)).await;
                }
            }
        }
    }


    pub async fn adb_get_state(&mut self, device_transport: DeviceTransport) -> Result<String, Box<dyn Error>> {
        self.send_transport(device_transport.clone()).await?;
        self.send_adb_command(ADB_GET_STATE_COMMAND).await?;
        let response = self.read_first_four_bytes_response().await?;

        if response != OKAY {
            let error_msg_str = self.read_adb_full_response().await?;
            return Err(format!("Failed to get device state: {}", error_msg_str).into());
        }

        self.read_adb_full_response().await
    }


    pub async fn adb_reboot(&mut self, device_transport: DeviceTransport, reboot_target: Option<String>) -> Result<(), Box<dyn Error>> {
        self.send_transport(device_transport.clone()).await?;
        let command = match reboot_target.as_deref() {
            Some(REBOOT_BOOTLOADER) => ADB_REBOOT_BOOTLOADER_COMMAND,
            Some(REBOOT_RECOVERY) => ADB_REBOOT_RECOVERY_COMMAND,
            Some(REBOOT_SIDELOAD) => ADB_REBOOT_SIDELOAD_COMMAND,
            Some(REBOOT_SIDELOAD_AUTO_REBOOT) => ADB_REBOOT_SIDELOAD_AUTO_REBOOT_COMMAND,
            Some(target) => {
                return Err(format!("Invalid reboot target: {}", target).into());
            }
            None => ADB_REBOOT_COMMAND,
        };
        self.send_adb_command(command).await?;
        let response = self.read_first_four_bytes_response().await?;

        if response != OKAY {
            let error_msg_str = self.read_adb_full_response().await?;
            return Err(format!("Failed to reboot the device: {}", error_msg_str).into());
        }
        Ok(())
    }
    pub async fn adb_serialno(&mut self, device_transport: DeviceTransport) -> Result<String, Box<dyn Error>> {
        self.send_transport(device_transport.clone()).await?;
        self.send_adb_command(HOST_SERIALNO_COMMAND).await?;

        let response = self.read_first_four_bytes_response().await?;

        if response != OKAY {
            let error_msg_str = self.read_adb_full_response().await?;
            return Err(format!("Failed to get device serial: {}", error_msg_str).into());
        }

        self.read_adb_full_response().await
    }

    pub async fn adb_remount(&mut self, device_transport: DeviceTransport) -> Result<String, Box<dyn Error>> {
        self.send_transport(device_transport.clone()).await?;
        self.send_adb_command(ADB_REMOUNT_COMMAND).await?;
        let response = self.read_first_four_bytes_response().await?;
        if response != OKAY {
            let error_msg_str = self.read_adb_full_response().await?;
            return Err(format!("Failed to remount the device: {}", error_msg_str).into());
        }
        self.read_all_data().await
    }

    pub async fn adb_root(&mut self, device_transport: DeviceTransport) -> Result<String, Box<dyn Error>> {
        self.send_transport(device_transport.clone()).await?;
        self.send_adb_command(ADB_ROOT_COMMAND).await?;
        let response = self.read_first_four_bytes_response().await?;

        if response != OKAY {
            let error_msg_str = self.read_adb_full_response().await?;
            return Err(format!("Failed to root the device: {}", error_msg_str).into());
        }

        self.read_all_data().await
    }

    pub async fn adb_unroot(&mut self, device_transport: DeviceTransport) -> Result<String, Box<dyn Error>> {
        self.send_transport(device_transport.clone()).await?;
        self.send_adb_command(ADB_UNROOT_COMMAND).await?;
        let response = self.read_first_four_bytes_response().await?;

        if response != OKAY {
            let error_msg_str = self.read_adb_full_response().await?;
            return Err(format!("Failed to unroot the device: {}", error_msg_str).into());
        }

        self.read_all_data().await
    }

    pub async fn adb_get_devpath(&mut self, device_transport: DeviceTransport) -> Result<String, Box<dyn Error>> {
        self.send_transport(device_transport.clone()).await?;
        self.send_adb_command(HOST_GET_DEVPATH_COMMAND).await?;

        let response = self.read_first_four_bytes_response().await?;

        if response != OKAY {
            let error_msg_str = self.read_adb_full_response().await?;
            return Err(format!("Failed to get device dev path: {}", error_msg_str).into());
        }

        self.read_adb_full_response().await
    }

    pub async fn adb_usb(&mut self, device_transport: DeviceTransport) -> Result<String, Box<dyn Error>> {
        self.send_transport(device_transport.clone()).await?;
        self.send_adb_command(ADB_USB_COMMAND).await?;
        let response = self.read_first_four_bytes_response().await?;

        if response != OKAY {
            let error_msg_str = self.read_adb_full_response().await?;
            return Err(format!("Failed to switch to USB mode: {}", error_msg_str).into());
        }

        self.read_adb_full_response().await
    }

    pub async fn adb_tcpip(&mut self, device_transport: DeviceTransport, port: u16) -> Result<String, Box<dyn Error>> {
        self.send_transport(device_transport.clone()).await?;
        let command = format!("{}{}", ADB_TCPIP_COMMAND, port);
        self.send_adb_command(&command).await?;
        let response = self.read_first_four_bytes_response().await?;

        if response != OKAY {
            let error_msg_str = self.read_adb_full_response().await?;
            return Err(format!("Failed to switch to TCP mode on port {}: {}", port, error_msg_str).into());
        }

        self.read_adb_full_response().await
    }
}