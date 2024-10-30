use std::fs::{File, create_dir_all};
use std::error::Error;
use std::path::PathBuf;
use crate::adb::client::Client;
use crate::constants::OKAY;
use crate::enums::device_transport::DeviceTransport;
use std::io::Write;
use openssl::rsa::Rsa;
use openssl::pkey::PKey;
use dirs::home_dir;

pub const ADB_DISABLE_VERITY_COMMAND: &str = "disable-verity:";
pub const ADB_ENABLE_VERITY_COMMAND: &str = "enable-verity:";
pub const ADB_FOLDER_NAME: &str = ".android";
pub const ADB_KEY_FILENAME: &str = "adbkey";

impl Client {
    pub async fn adb_disable_verity(&mut self, device_transport: DeviceTransport) -> Result<String, Box<dyn Error>> {
        self.send_transport(device_transport.clone()).await?;
        self.send_adb_command(ADB_DISABLE_VERITY_COMMAND).await?;
        let response = self.read_first_four_bytes_response().await?;
        if response != OKAY {
            let error_msg_str = self.read_adb_full_response().await?;
            return Err(format!("Failed to disable verity: {}", error_msg_str).into());
        }

        let disable_verity_response = self.read_all_data().await?;
        Ok(disable_verity_response)
    }
    pub async fn adb_enable_verity(&mut self, device_transport: DeviceTransport) -> Result<String, Box<dyn Error>> {
        self.send_transport(device_transport.clone()).await?;
        self.send_adb_command(ADB_ENABLE_VERITY_COMMAND).await?;
        let response = self.read_first_four_bytes_response().await?;
        if response != OKAY {
            let error_msg_str = self.read_adb_full_response().await?;
            return Err(format!("Failed to enable verity: {}", error_msg_str).into());
        }

        let enable_verity_response = self.read_all_data().await?;
        Ok(enable_verity_response)
    }

    pub fn adb_keygen(&self, file_path: Option<&str>) -> Result<String, Box<dyn Error>> {
        let file_path = if let Some(path) = file_path {
            let path = PathBuf::from(path);
            if path.is_dir() {
                path.join(ADB_KEY_FILENAME)
            } else {
                path
            }
        } else {
            let mut path = home_dir().ok_or("Unable to determine home directory")?;
            path.push(ADB_FOLDER_NAME);
            path.push(ADB_KEY_FILENAME);
            path
        };

        if let Some(parent) = file_path.parent() {
            create_dir_all(parent)?;
        }

        let rsa = Rsa::generate(2048)?;
        let pkey = PKey::from_rsa(rsa)?;

        let private_key = pkey.private_key_to_pem_pkcs8()?;
        let public_key = pkey.public_key_to_pem()?;

        let mut private_key_file = File::create(&file_path)?;
        private_key_file.write_all(&private_key)?;

        let public_key_path = file_path.with_extension("pub");
        let mut public_key_file = File::create(&public_key_path)?;
        public_key_file.write_all(&public_key)?;

        Ok(format!("ADB key pair generated successfully.\nPrivate key saved to: {}\nPublic key saved to: {}",
                   file_path.display(), public_key_path.display()))
    }
}