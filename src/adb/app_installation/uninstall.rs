use std::error::Error;
use crate::adb::client::Client;
use crate::constants::PM_UNINSTALL;
use crate::enums::device_transport::DeviceTransport;



impl Client {
    pub async fn adb_uninstall(&mut self, device_transport: DeviceTransport, package_name: &str, flags: &[String], ) -> Result<String, Box<dyn Error>> {
        if package_name.is_empty() {
            return Err("Package name is required".into());
        }

        let mut pm_command = String::from(PM_UNINSTALL);
        for flag in flags {
            pm_command.push_str(&format!(" {}", flag));
        }

        pm_command.push_str(&format!(" {}", package_name));

        self.adb_shell(device_transport, &pm_command).await
    }
}