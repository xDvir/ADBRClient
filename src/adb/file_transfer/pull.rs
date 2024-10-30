use std::collections::VecDeque;
use std::error::Error;
use std::fs;
use std::fs::set_permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path};
use std::time::{Instant, UNIX_EPOCH};
use filetime::{FileTime, set_file_times};
use crate::adb::client::Client;
use crate::enums::pull_result::PullResult;
use crate::constants::{RECV_COMMAND, DATA_COMMAND, DONE_COMMAND, FAIL, OKAY, S_IFDIR, SYNC_COMMAND, QUIT_COMMAND, LIST_COMMAND, DENT_COMMAND};
use crate::enums::device_transport::DeviceTransport;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt};
use crate::models::remote_dir_entry::RemoteDirEntry;


impl Client {
    pub async fn adb_pull(&mut self, device_transport: DeviceTransport, remote_paths: &[String], local_path: &str, preserve: bool) -> Result<Vec<(String, Result<PullResult, Box<dyn Error>>)>, Box<dyn Error>> {
        self.send_transport(device_transport).await?;
        self.send_adb_command(SYNC_COMMAND).await?;
        let response = self.read_first_four_bytes_response().await?;


        if response != OKAY {
            return Err(format!("adb pull failed: Unexpected response: {}", response).into());
        }

        let local_path = Path::new(local_path);
        let should_be_directory = remote_paths.len() > 1 || local_path.is_dir();

        if should_be_directory && !local_path.exists() {
            tokio::fs::create_dir_all(local_path).await?;
        }

        let mut results = Vec::new();
        for remote_path in remote_paths {
            let result = self.pull_single_file(remote_path, local_path, should_be_directory, preserve).await;

            match &result {
                Ok(PullResult::FailedAllPull(err)) => {
                    return Err(err.clone().into());
                }
                Err(err) => {
                    return Err(err.to_string().into());
                }
                _ => results.push((remote_path.clone(), result)),
            }
        }

        self.send_command(QUIT_COMMAND.as_bytes()).await?;

        Ok(results)
    }

    async fn pull_single_file(&mut self, remote_path: &str, local_path: &Path, should_be_directory: bool, preserve: bool) -> Result<PullResult, Box<dyn Error>> {
        let is_directory = self.check_remote_path_is_directory(remote_path).await?;

        if is_directory {
            self.pull_directory(remote_path, local_path, preserve).await
        } else {
            let dest_path = if should_be_directory {
                if !local_path.is_dir() {
                    tokio::fs::create_dir_all(local_path).await?;
                }
                local_path.join(Path::new(remote_path).file_name().ok_or("Invalid remote filename")?)
            } else {
                local_path.to_path_buf()
            };
            self.pull_file(remote_path, &dest_path, preserve).await
        }
    }

    async fn pull_file(&mut self, remote_path: &str, local_path: &Path, preserve: bool) -> Result<PullResult, Box<dyn Error>> {
        let pull_start_time = Instant::now();

        self.send_command(RECV_COMMAND.as_bytes()).await?;
        self.send_command(&(remote_path.len() as u32).to_le_bytes()).await?;
        self.send_command(remote_path.as_bytes()).await?;

        let header = self.get_exact_bytes(8).await?;
        let cmd = std::str::from_utf8(&header[..4])?;
        let size = u32::from_le_bytes(header[4..8].try_into()?);

        if cmd == FAIL {
            let error_msg = self.get_exact_bytes(size as usize).await?;
            return Err(format!(
                "adb: error: failed to copy '{}' to '{}': {}",
                remote_path,
                local_path.display(),
                String::from_utf8_lossy(&error_msg)
            ).into());
        }

        if cmd != DATA_COMMAND {
            return Err(format!("adb: error: unexpected response: {}", cmd).into());
        }

        let mut file = File::create(local_path).await?;
        let mut total_bytes = 0u64;

        let data = self.get_exact_bytes(size as usize).await?;
        file.write_all(&data).await?;
        total_bytes += size as u64;

        loop {
            let header = self.get_exact_bytes(8).await?;
            let cmd = std::str::from_utf8(&header[..4])?;
            let size = u32::from_le_bytes(header[4..8].try_into()?);

            match cmd {
                DATA_COMMAND => {
                    let data = self.get_exact_bytes(size as usize).await?;
                    file.write_all(&data).await?;
                    total_bytes += size as u64;
                }
                DONE_COMMAND => {
                    break;
                }
                FAIL => {
                    let error_msg = self.get_exact_bytes(size as usize).await?;
                    return Err(format!(
                        "adb: error: failed to copy '{}' to '{}': {}",
                        remote_path,
                        local_path.display(),
                        String::from_utf8_lossy(&error_msg)
                    ).into());
                }
                _ => {
                    continue;
                }
            }
        }

        if preserve {
            match self.get_remote_metadata(remote_path).await {
                Ok(metadata) => {
                    set_permissions(local_path, fs::Permissions::from_mode(metadata.mode))?;
                    let mtime = UNIX_EPOCH + std::time::Duration::from_secs(metadata.mtime as u64);
                    let system_time = mtime;
                    set_file_times(local_path, FileTime::from_system_time(system_time), FileTime::from_system_time(system_time))?;
                }
                Err(e) => {
                    eprintln!("Warning: Failed to preserve metadata for '{}': {}", remote_path, e);
                }
            }
        }
        let duration = pull_start_time.elapsed();
        let transfer_rate = total_bytes as f64 / duration.as_secs_f64() / 1_000_000.0;

        Ok(PullResult::Success(transfer_rate, total_bytes, duration, 1))
    }

    async fn pull_directory(&mut self, remote_path: &str, local_path: &Path, preserve: bool) -> Result<PullResult, Box<dyn Error>> {
        let mut total_files = 0;
        let mut total_bytes = 0u64;
        let start_time = Instant::now();

        tokio::fs::create_dir_all(local_path).await?;

        let mut dirs_to_process = VecDeque::new();
        dirs_to_process.push_back((remote_path.to_string(), local_path.to_path_buf()));

        while let Some((current_remote_dir, current_local_dir)) = dirs_to_process.pop_front() {
            tokio::fs::create_dir_all(&current_local_dir).await?;

            let entries = self.list_remote_directory(&current_remote_dir).await?;

            for entry in entries {
                let remote_file_path = format!("{}/{}", current_remote_dir.trim_end_matches('/'), entry.name);
                let local_file_path = current_local_dir.join(&entry.name);

                if entry.mode & S_IFDIR != 0 {
                    dirs_to_process.push_back((remote_file_path, local_file_path));
                } else {
                    match self.pull_file(&remote_file_path, &local_file_path, preserve).await {
                        Ok(PullResult::Success(_, bytes, _, _)) => {
                            total_files += 1;
                            total_bytes += bytes;
                        }
                        Ok(PullResult::FailedAllPull(err)) => {
                            return Ok(PullResult::FailedAllPull(err));
                        }
                        Err(e) => {
                            return Err(e);
                        }
                        _ => {}
                    }
                }
            }
        }

        let duration = start_time.elapsed();
        let transfer_rate = if duration.as_secs_f64() > 0.0 {
            total_bytes as f64 / duration.as_secs_f64() / 1_000_000.0
        } else {
            0.0
        };

        Ok(PullResult::SuccessDirectory(transfer_rate, total_bytes, duration, total_files))
    }

    async fn list_remote_directory(&mut self, remote_path: &str) -> Result<Vec<RemoteDirEntry>, Box<dyn Error>> {
        self.send_command(LIST_COMMAND.as_bytes()).await?;
        self.send_command(&(remote_path.len() as u32).to_le_bytes()).await?;
        self.send_command(remote_path.as_bytes()).await?;

        let mut entries = Vec::new();

        loop {
            let header = self.get_exact_bytes(4).await?;
            let cmd = std::str::from_utf8(&header)?;


            if cmd == DONE_COMMAND {
                break;
            } else if cmd == FAIL {
                let size_bytes = self.get_exact_bytes(4).await?;
                let size = u32::from_le_bytes(size_bytes.as_slice().try_into()?);
                let error_msg = self.get_exact_bytes(size as usize).await?;
                return Err(format!(
                    "adb: error: failed to list directory '{}': {}",
                    remote_path,
                    String::from_utf8_lossy(&error_msg)
                ).into());
            } else if cmd == DENT_COMMAND {
                let data = self.get_exact_bytes(16).await?;
                let mode = u32::from_le_bytes(data[0..4].try_into()?);
                let size = u32::from_le_bytes(data[4..8].try_into()?);
                let mtime = u32::from_le_bytes(data[8..12].try_into()?);
                let namelen = u32::from_le_bytes(data[12..16].try_into()?);

                let name_bytes = self.get_exact_bytes(namelen as usize).await?;
                let name = String::from_utf8_lossy(&name_bytes).to_string();

                entries.push(RemoteDirEntry {
                    name,
                    mode,
                    size,
                    mtime,
                });
            } else {
                return Err(format!("adb: error: unexpected response during list: {}", cmd).into());
            }
        }

        Ok(entries)
    }
}