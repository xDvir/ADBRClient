use std::error::Error;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;
use crate::adb::client::Client;
use tokio::io::{AsyncReadExt};
use crate::enums::push_result::PushResult;
use tokio::fs::File;
use crate::constants::{DATA_COMMAND, DONE_COMMAND, FAIL, OKAY, SEND_COMMAND, SYNC_COMMAND, QUIT_COMMAND, DEFAULT_PUSH_MODE};
use crate::enums::device_transport::DeviceTransport;
use crate::models::stat_data::StatData;

impl Client {
    pub async fn adb_push(&mut self, device_transport: DeviceTransport, local_paths: &[String], remote_path: &str, sync: bool) -> Result<Vec<(String, Result<PushResult, Box<dyn Error>>)>, Box<dyn Error>> {
        self.send_transport(device_transport).await?;
        self.send_adb_command(SYNC_COMMAND).await?;
        let response = self.read_first_four_bytes_response().await?;

        if response != OKAY {
            println!("{}", self.read_adb_full_response().await?);
            return Err(format!("adbr push failed: Unexpected response: {}", response).into());
        }
        let mut full_remote_path = remote_path.to_string();
        let should_be_directory = local_paths.len() > 1 || full_remote_path.ends_with('/') || full_remote_path.ends_with('\\');
        if should_be_directory || !full_remote_path.ends_with('/') && !full_remote_path.ends_with('\\') {
            match self.check_remote_path_is_directory(&full_remote_path).await? {
                true => {
                    if !full_remote_path.ends_with('/') && !full_remote_path.ends_with('\\') {
                        full_remote_path.push('/');
                    }
                }
                false => {
                    if should_be_directory {
                        return Err(format!("adbr: error: target '{}' is not a directory", remote_path).into());
                    }
                }
            }
        }

        let mut results = Vec::new();
        for local_path in local_paths {
            let mut current_remote_path = full_remote_path.clone();
            if current_remote_path.ends_with('/') || current_remote_path.ends_with('\\') {
                if let Some(file_name) = Path::new(local_path).file_name() {
                    if let Some(file_name_str) = file_name.to_str() {
                        current_remote_path.push_str(file_name_str);
                    } else {
                        return Err("Invalid UTF-8 in filename".into());
                    }
                } else {
                    return Err("Invalid local filename".into());
                }
            }
            let result = self.push_single_file(local_path, &current_remote_path, sync).await;

            match &result {
                Ok(PushResult::FailedAllPush(_)) | Err(_) => {
                    results.push((local_path.clone(), result));
                    return Ok(results);
                }
                _ => results.push((local_path.clone(), result)),
            }
        }

        self.send_command(QUIT_COMMAND.as_ref()).await?;

        Ok(results)
    }

    async fn push_single_file(&mut self, local_path: &str, remote_path: &str, sync: bool) -> Result<PushResult, Box<dyn Error>> {
        let local_path = Path::new(local_path);
        if local_path.is_dir() {
            self.push_directory(local_path, remote_path, sync).await
        } else {
            self.push_file(local_path, remote_path, sync).await
        }
    }

    async fn push_directory(&mut self, local_dir: &Path, remote_dir: &str, sync: bool) -> Result<PushResult, Box<dyn Error>> {
        let mut total_files = 0;
        let mut total_bytes = 0;
        let start_time = Instant::now();

        for entry in WalkDir::new(local_dir) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let relative_path = entry.path().strip_prefix(local_dir)?;
                let remote_path = Path::new(remote_dir).join(relative_path);
                match self.push_file(entry.path(), &remote_path.to_string_lossy(), sync).await {
                    Ok(PushResult::Success(_, bytes, _, _)) => {
                        total_files += 1;
                        total_bytes += bytes;
                    }
                    Ok(PushResult::Skip) => {}
                    Ok(PushResult::FailedAllPush(err)) => return Ok(PushResult::FailedAllPush(err)),
                    Err(e) => return Err(e),
                    _ => {}
                }
            }
        }

        let duration = start_time.elapsed();
        let transfer_rate = total_bytes as f64 / duration.as_secs_f64() / 1_000_000.0;

        Ok(PushResult::SuccessDirectory(transfer_rate, total_bytes, duration, total_files))
    }

    async fn push_file(&mut self, local_path: &Path, remote_path: &str, sync: bool) -> Result<PushResult, Box<dyn Error>> {
        let push_start_time = Instant::now();

        if !local_path.exists() {
            return Err(format!("adb: error: cannot stat '{}': No such file or directory", local_path.display()).into());
        }

        let full_remote_path = PathBuf::from(remote_path);

        if sync {
            let should_push = match self.get_remote_stat(&full_remote_path).await {
                Ok(remote_stat) => self.should_push_file(local_path, &remote_stat).await?,
                Err(_) => true,
            };

            if !should_push {
                return Ok(PushResult::Skip);
            }
        }

        self.send_command(SEND_COMMAND.as_ref()).await?;

        let mode = if local_path.metadata()?.permissions().mode() & 0o111 != 0 {
            0o755  // rwxr-xr-x
        } else {
            DEFAULT_PUSH_MODE
        };

        let remote_path_with_mode = format!("{},{}", full_remote_path.to_string_lossy(), mode);

        self.send_command(&(remote_path_with_mode.len() as u32).to_le_bytes()).await?;
        self.send_command(remote_path_with_mode.as_bytes()).await?;

        let mut file = File::open(local_path).await?;
        let bytes_transferred = self.send_file_contents(&mut file).await?;

        self.send_last_modified_time(local_path).await?;

        let response = self.read_first_four_bytes_response().await?;
        if response == FAIL {
            let adb_full_response = self.read_adb_full_response().await?;
            return Ok(PushResult::FailedAllPush(format!("adbr: error: failed to copy '{}' to '{}': remote {}", local_path.display(), full_remote_path.display(), adb_full_response)));
        }

        let duration = push_start_time.elapsed();
        let transfer_rate = bytes_transferred as f64 / duration.as_secs_f64() / 1_000_000.0;

        Ok(PushResult::Success(transfer_rate, bytes_transferred, duration, 1))
    }


    async fn should_push_file(&mut self, local_path: &Path, remote_stat: &StatData) -> Result<bool, Box<dyn Error>> {
        let local_metadata = tokio::fs::metadata(local_path).await?;
        let local_mtime = local_metadata.modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;

        Ok(local_mtime > remote_stat.mtime())
    }

    async fn send_file_contents(&mut self, file: &mut File) -> Result<u64, Box<dyn Error>> {
        let mut buffer = vec![0u8; 64 * 1024];
        let mut total_sent = 0u64;
        loop {
            let bytes_read = file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            self.send_command(DATA_COMMAND.as_ref()).await?;
            self.send_command(&(bytes_read as u32).to_le_bytes()).await?;
            self.send_command(&buffer[..bytes_read]).await?;
            total_sent += bytes_read as u64;
        }
        Ok(total_sent)
    }

    async fn send_last_modified_time(&mut self, local_path: &Path) -> Result<(), Box<dyn Error>> {
        let metadata = tokio::fs::metadata(local_path).await?;
        let mtime = metadata.modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as u32;

        self.send_command(DONE_COMMAND.as_ref()).await?;
        self.send_command(&mtime.to_le_bytes()).await?;

        Ok(())
    }
}