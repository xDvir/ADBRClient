use std::error::Error;
use crate::adb::client::Client;
use crate::enums::device_transport::DeviceTransport;
use crate::constants::{OKAY};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};
use chrono::Local;
use indicatif::{ProgressBar, ProgressStyle};

const LOGCAT_COMMAND_FORMAT: &str = "export ANDROID_LOG_TAGS=\"''\"; exec logcat {}";
const BUGREPORT_COMMAND: &str = "shell:bugreportz -p";
const DEFAULT_BUGREPORT_FILENAME: &str = "bugreport.zip";
const BUGREPORT_PREFIX: &str = "bugreport";
const PROGRESS_PREFIX: &str = "PROGRESS:";
const OK_PREFIX: &str = "OK:";
const XOK_PREFIX: &str = "XOK:";
const INFO_PREFIX: &str = "INFO:";
const DATE_FORMAT: &str = "%Y-%m-%d-%H-%M-%S";
const PROGRESS_BAR_LENGTH: u64 = 5000;

impl Client {
    pub async fn adb_bugreport(&mut self, device_transport: DeviceTransport, path: Option<&str>) -> Result<(), Box<dyn Error>> {
        self.send_transport(device_transport.clone()).await?;
        self.send_adb_command(BUGREPORT_COMMAND.as_ref()).await?;

        let response = self.read_first_four_bytes_response().await?;
        if response != OKAY {
            let error_msg = self.read_adb_full_response().await?;
            return Err(format!("Failed to initiate bugreport: {}", error_msg).into());
        }

        let output_path = path.unwrap_or(DEFAULT_BUGREPORT_FILENAME);
        let path = Path::new(output_path);

        let final_path = if path.is_dir() {
            let filename = format!("{}-{}.zip", BUGREPORT_PREFIX, Local::now().format(DATE_FORMAT));
            path.join(filename)
        } else {
            path.to_path_buf()
        };
        self.save_bugreportz_to_file(device_transport.clone(), &final_path).await
    }

    async fn save_bugreportz_to_file(&mut self, device: DeviceTransport, path: &Path) -> Result<(), Box<dyn Error>> {
        println!("Generating bugreport. This may take a while...");

        let pb = self.create_progress_bar();

        let mut reader = BufReader::new(&mut self.adb_stream);
        let mut line = String::new();
        let mut zip_file = String::new();

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await?;

            if bytes_read == 0 {
                break;
            }

            let trimmed_line = line.trim();

            if let Some(progress_str) = Self::extract_progress(trimmed_line) {
                if let Ok(progress) = progress_str.parse::<u64>() {
                    pb.set_position(progress);
                }
            } else if let Some(path_str) = Self::extract_zip_path(trimmed_line) {
                zip_file = path_str.to_string();
                break;
            } else if !trimmed_line.is_empty() && !trimmed_line.starts_with(INFO_PREFIX) {
                pb.suspend(|| println!("Info: {}", trimmed_line));
            }
        }

        pb.finish_with_message("Bugreport generated");

        if zip_file.is_empty() {
            return Err("Failed to generate bugreport: No zip file path received".into());
        }

        println!("\nPulling bugreport file...");

        let mut pull_client = Client::new(None, None).await?;
        pull_client.send_transport(device.clone()).await?;

        pull_client.adb_pull(device.clone(), &[zip_file], path.to_str().unwrap(), false).await?;

        println!("Bugreport saved to: {}", path.display());
        Ok(())
    }

    fn create_progress_bar(&self) -> ProgressBar {
        let pb = ProgressBar::new(PROGRESS_BAR_LENGTH);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("#>-")
        );
        pb
    }

    fn extract_progress(line: &str) -> Option<&str> {
        if let Some(idx) = line.find(PROGRESS_PREFIX) {
            let progress_part = &line[idx + PROGRESS_PREFIX.len()..];
            progress_part.split('/').next()
        } else {
            None
        }
    }

    fn extract_zip_path(line: &str) -> Option<&str> {
        if let Some(idx) = line.find(OK_PREFIX) {
            Some(line[idx + OK_PREFIX.len()..].trim())
        } else if let Some(idx) = line.find(XOK_PREFIX) {
            Some(line[idx + XOK_PREFIX.len()..].trim())
        } else {
            None
        }
    }

    pub async fn adb_logcat(&mut self, device: DeviceTransport, args: &str) -> Result<String, Box<dyn Error>> {
        let logcat_command = format!("{} {}", LOGCAT_COMMAND_FORMAT, args);
        self.adb_shell(device, &logcat_command).await
    }
}