use std::fs;
use std::path::Path;
use std::error::Error;
use crate::adb::client::Client;
use crate::enums::device_transport::DeviceTransport;
use crate::constants::{
    PM_INSTALL,
    INSTALL_FLAG_SDCARD, INSTALL_FLAG_INTERNAL,
    INSTALL_FLAG_DOWNGRADE, INSTALL_FLAG_REPLACE,
    DEVICE_TEMP_DIRECTORY,
};

impl Client {
    pub async fn adb_install(&mut self, device_transport: DeviceTransport, local_apk_path: &str, flags: &[String]) -> Result<String, Box<dyn Error>> {
        if !fs::metadata(local_apk_path).is_ok() {
            return Err(format!("APK file not found at path: {}", local_apk_path).into());
        }

        if flags.contains(&INSTALL_FLAG_SDCARD.to_string()) && flags.contains(&INSTALL_FLAG_INTERNAL.to_string()) {
            return Err(format!("{} and {} flags are mutually exclusive", INSTALL_FLAG_SDCARD, INSTALL_FLAG_INTERNAL).into());
        }

        if flags.contains(&INSTALL_FLAG_DOWNGRADE.to_string()) && flags.contains(&INSTALL_FLAG_REPLACE.to_string()) {
            println!("Warning: {} and {} flags may not work together on some Android versions", INSTALL_FLAG_DOWNGRADE, INSTALL_FLAG_REPLACE);
        }

        let apk_filename = Path::new(local_apk_path)
            .file_name()
            .ok_or("Invalid APK path")?
            .to_str()
            .ok_or("APK filename is not valid UTF-8")?;

        let remote_path = format!("{}{}", DEVICE_TEMP_DIRECTORY, apk_filename);

        self.adb_push(device_transport.clone(), &[local_apk_path.to_string()], &remote_path, false).await?;

        self.reconnect().await?;

        let mut pm_command = String::from(PM_INSTALL);
        for flag in flags {
            pm_command.push_str(&format!(" {}", flag));
        }
        pm_command.push_str(&format!(" {}", remote_path));

        self.adb_shell(device_transport, &pm_command).await
    }
}