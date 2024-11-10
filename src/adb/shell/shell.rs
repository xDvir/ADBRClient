use std::error::Error;
use std::io;

use tokio::time::{Duration};
use std::io::{Write};
use std::os::fd::BorrowedFd;
use std::os::unix::io::{AsRawFd};
use termios::{Termios, TCSAFLUSH, ICANON, ECHO};
use nix::sys::select::{select, FdSet};
use nix::sys::time::TimeVal;
use nix::unistd::read;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::adb::client::Client;
use crate::constants::{ADB_SHELL_COMMAND, FAIL, OKAY, SELECT_TIMEOUT_USEC, USER_EXIT_COMMAND};
use crate::enums::device_transport::DeviceTransport;

const WAIT_FOR_FIRST_CONNECTION_DURATION_MS: u64 = 300;

impl Client {
    pub async fn adb_shell(&mut self, device_transport: DeviceTransport, shell_command: &str) -> Result<String, Box<dyn Error>> {
        self.send_transport(device_transport.clone()).await?;
        let adb_shell_command = format!("{}{}", ADB_SHELL_COMMAND, shell_command);
        self.send_adb_command(&adb_shell_command).await?;

        tokio::time::sleep(Duration::from_millis(WAIT_FOR_FIRST_CONNECTION_DURATION_MS)).await;

        match self.read_first_four_bytes_response().await?.as_str() {
            OKAY => {
                if shell_command.is_empty() {
                    self.interactive_shell().await?;
                    Ok(String::new())
                } else {
                    let output = self.read_print_and_collect_output().await?;
                    io::stdout().flush()?;
                    Ok(output)
                }
            }
            FAIL => {
                let fail_response = self.read_adb_full_response().await?;
                Ok(fail_response)
            }
            _ => {
                let error_message = self.read_adb_full_response().await?;
                Err(format!(
                    "Failed to send shell command : {}",
                    error_message
                )
                    .into())
            }
        }
    }

    pub async fn interactive_shell(&mut self) -> Result<(), Box<dyn Error>> {
        let stdin = io::stdin();
        let stdin_fd = stdin.as_raw_fd();
        let mut oldtty_attrs = None;

        if atty::is(atty::Stream::Stdin) {
            oldtty_attrs = Some(Termios::from_fd(stdin_fd)?);
            let mut new_termios = oldtty_attrs.unwrap();
            new_termios.c_lflag &= !(ICANON | ECHO);
            termios::tcsetattr(stdin_fd, TCSAFLUSH, &new_termios)?;
        }

        let mut is_alive = true;
        let mut input_buffer = String::new();

        while is_alive {
            let mut read_fds = FdSet::new();
            read_fds.insert(unsafe { BorrowedFd::borrow_raw(self.adb_stream.as_raw_fd()) });
            read_fds.insert(unsafe { BorrowedFd::borrow_raw(stdin_fd) });

            match select(None, &mut read_fds, None, None, Some(&mut TimeVal::new(0, SELECT_TIMEOUT_USEC))) {
                Ok(ready) if ready > 0 => {
                    if read_fds.contains(unsafe { BorrowedFd::borrow_raw(self.adb_stream.as_raw_fd()) }) {
                        let mut buf = [0; 8000];
                        match self.adb_stream.read(&mut buf).await {
                            Ok(0) => is_alive = false,
                            Ok(n) => {
                                let decoded_out = String::from_utf8_lossy(&buf[..n]);
                                print!("{}", decoded_out);
                                io::stdout().flush()?;
                            }
                            Err(_) => is_alive = false,
                        }
                    }

                    if read_fds.contains(unsafe { BorrowedFd::borrow_raw(stdin_fd) }) && is_alive {
                        let mut char = [0; 1];
                        match read(stdin_fd, &mut char) {
                            Ok(0) => is_alive = false,
                            Ok(_) => {
                                self.adb_stream.write_all(&char).await?;
                                input_buffer.push(char[0] as char);

                                if input_buffer.ends_with(USER_EXIT_COMMAND) {
                                    print!("\n");
                                    is_alive = false;
                                }
                            }
                            Err(_) => is_alive = false,
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => return Err(Box::new(e)),
            }
        }

        if let Some(attrs) = oldtty_attrs {
            termios::tcsetattr(stdin_fd, TCSAFLUSH, &attrs)?;
        }

        Ok(())
    }
}