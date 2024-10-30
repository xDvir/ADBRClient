use std::env::args;
use std::io;
use std::io::Write;
use std::time::Duration;

use adbr::Client;
use adbr::DeviceTransport;
use adbr::constants::{FLAG_SERVER_ADDRESS, FLAG_VERSION, FLAG_EMULATOR, FLAG_SERVER_PORT, UNINSTALL_FLAG_KEEP_DATA, FLAG_SERIAL, FLAG_USB, OPTION_LIST, OPTION_NO_REBIND, OPTION_REMOVE, OPTION_REMOVE_ALL, USER_CONNECT_COMMAND, USER_DEVICES_COMMAND, USER_SHELL_COMMAND, USER_FORWARD_COMMAND, USER_REBOOT_COMMAND, USER_PUSH_COMMAND, USER_PULL_COMMAND, INSTALL_FLAG_SDCARD, INSTALL_FLAG_INTERNAL, USER_INSTALL_COMMAND, INSTALL_FLAG_DOWNGRADE, INSTALL_FLAG_REPLACE, INSTALL_FLAG_GRANT_PERMISSIONS, INSTALL_FLAG_TEST, INSTALL_FLAG_FORWARD_LOCK, USER_SERIALNO_COMMAND, USER_GET_DEVPATH_COMMAND, USER_ROOT_COMMAND, USER_UNROOT_COMMAND, USER_REMOUNT_COMMAND, FLAG_HELP_SHORT, VERSION, PROGRAM_NAME, FLAG_HELP_LONG, REFRESH_INTERVAL_SECS, FLAG_WATCH_DEVICES, USER_DISABLE_VERITY_COMMAND, USER_LOGCAT_COMMAND, USER_BUGREPORT_COMMAND, USER_TCPIP_COMMAND, USER_USB_COMMAND, USER_ENABLE_VERITY_COMMAND, USER_KEYGEN_COMMAND, USER_REVERSE_COMMAND, USER_GET_STATE_COMMAND, DEFAULT_WAIT_STATE, USER_WAIT_FOR_COMMAND, FLAG_TIMEOUT, USER_UNINSTALL_COMMAND};
use adbr::PushResult;
use adbr::PullResult;

#[tokio::main]
async fn main() {
    let args: Vec<String> = args().collect();

    if args.len() > 1 && (args[1] == FLAG_HELP_SHORT || args[1] == FLAG_HELP_LONG) {
        print_usage();
        return;
    } else if args.len() > 1 && args[1] == FLAG_VERSION {
        println!("{} version {}", PROGRAM_NAME, VERSION);
        return;
    } else if args.len() < 2 {
        print_usage();
        return;
    }

    handle_commands(args).await;
}

fn print_usage() {
    println!("Usage: {} [options] <command> [command args]", PROGRAM_NAME);
    println!();
    println!("global options:");
    println!("  -s <serial>     Use device with given serial number");
    println!("  -d              Use USB device (error if multiple devices connected)");
    println!("  -e              Use TCP/IP device (error if multiple TCP/IP devices available)");
    println!("  -H <host>       Name of adb server host [default=localhost]");
    println!("  -P <port>       Port of adb server [default=5037]");
    println!();
    println!("general commands:");
    println!("  devices [-w]    List connected devices");
    println!("                  -w: continuously monitors devices, refreshing every {} seconds", REFRESH_INTERVAL_SECS);
    println!("  --version   Print the version of the adbr client");

    println!();
    println!("networking:");
    println!("  forward         Forward socket connections");
    println!("    forward --list");
    println!("      List all forward socket connections");
    println!("    forward [--no-rebind] <local> <remote>");
    println!("      Forward socket connection");
    println!("      --no-rebind: Fail if local specification is already used");
    println!("    forward --remove <local>");
    println!("      Remove specific forward socket connection");
    println!("    forward --remove-all");
    println!("      Remove all forward socket connections");
    println!("  reverse         Reverse socket connections");
    println!("    reverse --list");
    println!("      List all reverse socket connections");
    println!("    reverse [--no-rebind] <remote> <local>");
    println!("      Reverse socket connection");
    println!("      --no-rebind: Fail if remote specification is already used");
    println!("    reverse --remove <remote>");
    println!("      Remove specific reverse socket connection");
    println!("    reverse --remove-all");
    println!("      Remove all reverse socket connections");
    println!("");
    println!("file transfer:");
    println!("  push [--sync] LOCAL... REMOTE");
    println!("     Copy local files/directories to device");
    println!("     --sync: only push files that are newer on the host than the device");
    println!("  pull [-a] REMOTE... LOCAL");
    println!("     Copy remote files/directories to host");
    println!("     -a: preserve file timestamp and mode");
    println!();
    println!("shell:");
    println!("  shell [<cmd>]   Run remote shell command (interactive shell if no command given)");
    println!();
    println!("app installation:");
    println!("  install [<flags>] <file>");
    println!("    Install package from the given file");
    println!("    flags:");
    println!("      -r: Replace existing application");
    println!("      -d: Allow version code downgrade");
    println!("      -g: Grant all runtime permissions");
    println!("      -t: Allow test packages");
    println!("      -s: Install package on the shared mass storage (SD card)");
    println!("      -f: Install package on the internal system memory");
    println!("      -l: Forward lock application");
    println!("  uninstall [-k] PACKAGE");
    println!("      remove this app package from the device");
    println!("       '-k': keep the data and cache directories");
    println!();
    println!("    Note: -s and -f flags are mutually exclusive");
    println!("          -d and -r flags may not work together on some Android versions");
    println!();
    println!("debugging:");
    println!("  logcat [<options>] [<filterspecs>]");
    println!("    View device log");
    println!("    options:");
    println!("      -c            Clear (flush) the entire log and exit");
    println!("      -f <filename> Log to file instead of stdout");
    println!("      -v <format>   Sets the output format for log messages");
    println!("                    Formats: brief, process, tag, thread, raw, time, threadtime, long");
    println!("      -b <buffer>   Request alternate ring buffer");
    println!("                    Buffers: main, system, radio, events, crash, default");
    println!("      -d            Dump the log and then exit (don't block)");
    println!("      -t <count>    Print only the most recent <count> lines (implies -d)");
    println!("      -T <time>     Print most recent lines since specified time (implies -d)");
    println!("    filterspecs:");
    println!("      <tag>[:priority]");
    println!("  bugreport [PATH]");
    println!("    Generate a bug report and save to PATH (default: bugreport.zip)");
    println!("    The report includes system logs, stack traces, and other diagnostic information");
    println!("");
    println!("security:");
    println!("  disable-verity    Disable dm-verity checking on userdebug builds");
    println!("  enable-verity     Re-enable dm-verity checking on userdebug builds");
    println!("  keygen [FILE]     Generate adb public/private key pair");
    println!("                    If FILE is not specified, generates key in ~/.android/adbkey");
    println!();
    println!("scripting:");
    println!("  wait-for[-TRANSPORT]-STATE [-t TIMEOUT]");
    println!("    Wait for device to be in the given state with optional timeout");
    println!("    TRANSPORT: usb | local | any (default: any)");
    println!("    STATE: device | recovery | rescue | sideload | bootloader | disconnect");
    println!("    -t TIMEOUT: Maximum time in seconds to wait for the device state");
    println!("  get-state");
    println!("    Prints the current state of the connected device");
    println!("  reboot [bootloader|recovery|sideload|sideload-auto-reboot]");
    println!("    Reboot the device; defaults to booting system image but");
    println!("    supports bootloader and recovery too. sideload reboots");
    println!("    into recovery and automatically starts sideload mode,");
    println!("    sideload-auto-reboot is the same but reboots after sideloading.");
    println!("  remount           Remount /system, /vendor, and /oem partitions read-write");
    println!("  root              Restart adbd with root permissions");
    println!("  unroot            Restart adbd without root permissions");
    println!("  usb               Restart adb server listening on USB");
    println!("  tcpip PORT        Restart adb server listening on TCP on PORT");
    println!();
    println!("environment variables:");
    println!("  ADB_ADDRESS       IP:PORT of ADB server (default: 127.0.0.1:5037)");
    println!();
    println!("Examples:");
    println!("  {} devices", PROGRAM_NAME);
    println!("  {} devices -w", PROGRAM_NAME);
    println!("  {} disable-verity", PROGRAM_NAME);
    println!("  {} enable-verity", PROGRAM_NAME);
    println!("  {} keygen", PROGRAM_NAME);
    println!("  {} keygen ~/.android/my_custom_adbkey", PROGRAM_NAME);
    println!("  {} bugreport", PROGRAM_NAME);
    println!("  {} bugreport /path/to/save/bugreport.zip", PROGRAM_NAME);
    println!("  {} logcat", PROGRAM_NAME);
    println!("  {} logcat *:E", PROGRAM_NAME);
    println!("  {} logcat -c", PROGRAM_NAME);
    println!("  {} logcat -v time ActivityManager:I *:S", PROGRAM_NAME);
    println!("  {} reboot", PROGRAM_NAME);
    println!("  {} remount", PROGRAM_NAME);
    println!("  {} root", PROGRAM_NAME);
    println!("  {} unroot", PROGRAM_NAME);
    println!("  {} usb", PROGRAM_NAME);
    println!("  {} tcpip 5555", PROGRAM_NAME);
    println!("  {} wait-for-device", PROGRAM_NAME);
    println!("  {} wait-for-usb-recovery -t 30", PROGRAM_NAME);
    println!("  {} get-state", PROGRAM_NAME);
    println!("  {} forward tcp:8000 tcp:9000", PROGRAM_NAME);
    println!("  {} forward --list", PROGRAM_NAME);
    println!("  {} forward --remove tcp:8000", PROGRAM_NAME);
    println!("  {} forward --remove-all", PROGRAM_NAME);
    println!("  {} reverse tcp:8000 tcp:9000", PROGRAM_NAME);
    println!("  {} reverse --list", PROGRAM_NAME);
    println!("  {} reverse --remove tcp:8000", PROGRAM_NAME);
    println!("  {} reverse --remove-all", PROGRAM_NAME);
    println!("  {} push /path/to/local/file /sdcard/", PROGRAM_NAME);
    println!("  {} push --sync /path/to/local/dir /sdcard/remote_dir", PROGRAM_NAME);
    println!("  {} pull /sdcard/remote_file /path/to/local/", PROGRAM_NAME);
    println!("  {} pull /sdcard/remote_dir /path/to/local/dir", PROGRAM_NAME);
    println!();
}

async fn handle_commands(mut args: Vec<String>) {
    let mut device_type = DeviceTransport::default();
    let mut server_address = None;
    let mut server_port = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            FLAG_SERVER_ADDRESS => {
                if i + 1 >= args.len() {
                    eprintln!("Invalid command: missing IP address after {}", FLAG_SERVER_ADDRESS);
                    return;
                }
                server_address = Some(args[i + 1].clone());
                args.drain(i..=i + 1);
                i = 1;
                continue;
            }
            FLAG_SERVER_PORT => {
                if i + 1 >= args.len() {
                    eprintln!("Invalid command: missing port after {}", FLAG_SERVER_PORT);
                    return;
                }
                match args[i + 1].parse::<u16>() {
                    Ok(port) => server_port = Some(port),
                    Err(_) => {
                        eprintln!("Invalid port number: {}", args[i + 1]);
                        std::process::exit(1);
                    }
                }
                args.drain(i..=i + 1);
                i = 1;
                continue;
            }
            FLAG_SERIAL => {
                if i + 1 >= args.len() {
                    eprintln!("Invalid command: missing serial number after {}", FLAG_SERIAL);
                    return;
                }
                let serial = args[i + 1].clone();
                device_type = DeviceTransport::usb(serial);
                args.drain(i..=i + 1);
                i = 1;
                continue;
            }
            FLAG_USB => {
                device_type = DeviceTransport::default_usb();
                args.remove(i);
                i = 1;
                continue;
            }
            FLAG_EMULATOR => {
                device_type = DeviceTransport::default_emulator();
                args.remove(i);
                i = 1;
                continue;
            }
            _ => i += 1,
        }
    }

    let command = args[1].clone();
    let command_args = args[2..].to_vec();

    let mut client = match Client::new(server_address.clone(), server_port).await {
        Ok(client) => client,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };

    match command.as_str() {
        devices_cmd if devices_cmd.starts_with(USER_DEVICES_COMMAND) => {
            let mut watch_flag = false;
            let mut devices_args = command_args.clone();

            if !devices_args.is_empty() && devices_args[0] == FLAG_WATCH_DEVICES {
                watch_flag = true;
                devices_args.remove(0);
            }

            if watch_flag {
                loop {
                    let mut client = match Client::new(None, None).await {
                        Ok(client) => client,
                        Err(err) => {
                            eprintln!("Error connecting to server: {}", err);
                            break;
                        }
                    };

                    match client.adb_devices().await {
                        Ok(result) => {
                            print!("\x1B[2J\x1B[H");
                            println!("{}", result);
                            io::stdout().flush().unwrap();
                        }
                        Err(err) => {
                            eprintln!("{}", err);
                        }
                    }

                    client.close().await;

                    tokio::time::sleep(Duration::from_secs(REFRESH_INTERVAL_SECS)).await;
                }
            } else {
                match client.adb_devices().await {
                    Ok(result) => {
                        println!("{}", result);
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                    }
                }
            }
        }
        connect_cmd if connect_cmd.starts_with(USER_CONNECT_COMMAND) => {
            println!("Please set the ADB server address by exporting the ADB_ADDRESS environment variable before running the ADB client.");
        }
        shell_cmd if shell_cmd.starts_with(USER_SHELL_COMMAND) => {
            let shell_args = if !command_args.is_empty() {
                command_args.join(" ")
            } else {
                String::new()
            };
            match client.adb_shell(device_type, &shell_args).await {
                Ok(output) => {
                    print!("{}", output);
                },
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }
        forward_command if forward_command.starts_with(USER_FORWARD_COMMAND) => {
            let mut no_rebind = false;
            let mut remove = false;
            let mut remove_all = false;
            let mut list = false;
            let mut args = command_args.clone();

            while !args.is_empty() && args[0].starts_with("--") {
                match args[0].as_str() {
                    OPTION_NO_REBIND => {
                        if remove || remove_all || list {
                            eprintln!("Error: --no-rebind cannot be used with --remove, --remove-all, or --list");
                            return;
                        }
                        no_rebind = true;
                        args.remove(0);
                    }
                    OPTION_REMOVE => {
                        if no_rebind || remove_all || list {
                            eprintln!("Error: --remove cannot be used with --no-rebind, --remove-all, or --list");
                            return;
                        }
                        remove = true;
                        args.remove(0);
                    }
                    OPTION_REMOVE_ALL => {
                        if no_rebind || remove || list {
                            eprintln!("Error: --remove-all cannot be used with --no-rebind, --remove, or --list");
                            return;
                        }
                        remove_all = true;
                        args.remove(0);
                    }
                    OPTION_LIST => {
                        if no_rebind || remove || remove_all {
                            eprintln!("Error: --list cannot be used with other options");
                            return;
                        }
                        list = true;
                        args.remove(0);
                    }
                    _ => {
                        eprintln!("Unknown option: {}", args[0]);
                        return;
                    }
                }
            }

            if list {
                if !args.is_empty() {
                    eprintln!("Invalid forward list command. Usage: forward --list");
                    return;
                }
                match client.send_forward_command_list(device_type.clone()).await {
                    Ok(list) => {
                        print!("{}", list);
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                    }
                }
                return;
            }

            if remove_all {
                if !args.is_empty() {
                    eprintln!("Invalid forward remove-all command. Usage: forward --remove-all");
                    return;
                }
                if let Err(err) = client.send_forward_command_remove_all(device_type).await {
                    eprintln!("{}", err);
                    return;
                }
            } else if remove {
                if args.len() != 1 {
                    eprintln!("Invalid forward remove command. Usage: forward --remove <local>");
                    return;
                }
                if let Err(err) = client.send_forward_command_remove(device_type, &args[0]).await {
                    eprintln!("{}", err);
                    return;
                }
            } else {
                if args.len() != 2 {
                    eprintln!("Invalid forward command. Usage: forward [--no-rebind] <local> <remote>");
                    return;
                }
                if let Err(err) = client.send_forward_command_set(device_type, &args[0], &args[1], no_rebind).await {
                    eprintln!("{}", err);
                    return;
                }
            }
        }
        USER_REVERSE_COMMAND => {
            let mut no_rebind = false;
            let mut remove = false;
            let mut remove_all = false;
            let mut args = command_args.clone();

            while !args.is_empty() && args[0].starts_with("--") {
                match args[0].as_str() {
                    OPTION_NO_REBIND => {
                        if remove || remove_all {
                            eprintln!("Error: --no-rebind cannot be used with --remove or --remove-all");
                            return;
                        }
                        no_rebind = true;
                        args.remove(0);
                    }
                    OPTION_REMOVE => {
                        if no_rebind || remove_all {
                            eprintln!("Error: --remove cannot be used with --no-rebind or --remove-all");
                            return;
                        }
                        remove = true;
                        args.remove(0);
                    }
                    OPTION_REMOVE_ALL => {
                        if no_rebind || remove {
                            eprintln!("Error: --remove-all cannot be used with --no-rebind or --remove");
                            return;
                        }
                        remove_all = true;
                        args.remove(0);
                    }
                    OPTION_LIST => {
                        if no_rebind || remove || remove_all {
                            eprintln!("Error: --list cannot be used with other options");
                            return;
                        }
                        args.remove(0);

                        match client.send_reverse_command_list(device_type.clone()).await {
                            Ok(list) => {
                                print!("{}", list);
                            }
                            Err(err) => {
                                eprintln!("{}", err);
                            }
                        }
                        return;
                    }
                    _ => {
                        eprintln!("Unknown option: {}", args[0]);
                        return;
                    }
                }
            }
            if remove_all {
                if !args.is_empty() {
                    eprintln!("Invalid reverse remove-all command. Usage: reverse --remove-all");
                    return;
                }
                match client.send_reverse_command_remove_all(device_type.clone()).await {
                    Ok(response) => {
                        print!("{}", response);
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                    }
                }
            } else if remove {
                if args.len() != 1 {
                    eprintln!("Invalid reverse remove command. Usage: reverse --remove <remote>");
                    return;
                }
                match client.send_reverse_command_remove(device_type.clone(), &args[0]).await {
                    Ok(response) => {
                        print!("{}", response);
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                    }
                }
            } else {
                if args.len() != 2 {
                    eprintln!("Invalid reverse command. Usage: reverse [--no-rebind] <remote> <local>");
                    return;
                }
                match client.send_reverse_command_set(device_type.clone(), &args[0], &args[1], no_rebind).await {
                    Ok(response) => {
                        println!("{}", response);
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                    }
                }
            }
        }
        push_command if push_command.starts_with(USER_PUSH_COMMAND) => {
            let mut sync_flag = false;
            let mut push_args = command_args.clone();

            if !push_args.is_empty() && push_args[0] == "--sync" {
                sync_flag = true;
                push_args.remove(0);
            }

            if push_args.len() < 2 {
                eprintln!("Error: push command requires at least two arguments");
                eprintln!("Usage: push [--sync] LOCAL... REMOTE");
                return;
            }

            let remote_path = push_args.pop().unwrap();
            let local_paths: Vec<String> = push_args;

            match client.adb_push(device_type.clone(), &local_paths, &remote_path, sync_flag).await {
                Ok(results) => {
                    let mut total_files_pushed = 0;
                    let mut total_files_skipped = 0;
                    let mut total_files_failed = 0;
                    let mut total_bytes_transferred = 0;
                    let mut total_duration = std::time::Duration::new(0, 0);

                    for (local_path, result) in results {
                        match result {
                            Ok(push_result) => {
                                match push_result {
                                    PushResult::Success(_, bytes, duration, file_count) => {
                                        println!("{}: {} file{} pushed.", local_path, file_count, if file_count == 1 { "" } else { "s" });
                                        total_files_pushed += file_count;
                                        total_bytes_transferred += bytes;
                                        total_duration += duration;
                                    }
                                    PushResult::SuccessDirectory(_, bytes, duration, file_count) => {
                                        println!("{}: {} file{} pushed.", local_path, file_count, if file_count == 1 { "" } else { "s" });
                                        total_files_pushed += file_count;
                                        total_bytes_transferred += bytes;
                                        total_duration += duration;
                                    }
                                    PushResult::Skip => {
                                        println!("{}: 1 file skipped.", local_path);
                                        total_files_skipped += 1;
                                    }
                                    PushResult::FailedAllPush(msg) => {
                                        println!("{}", msg);
                                        total_files_failed += 1;
                                    }
                                }
                            }
                            Err(err) => {
                                println!("{}", err);
                                total_files_failed += 1;
                            }
                        }
                    }

                    let total_transfer_rate = if total_duration.as_secs_f64() > 0.0 {
                        total_bytes_transferred as f64 / total_duration.as_secs_f64() / 1_000_000.0
                    } else {
                        0.0
                    };

                    println!("{} file{} pushed. {} file{} skipped. {} file{} failed.",
                             total_files_pushed,
                             if total_files_pushed == 1 { "" } else { "s" },
                             total_files_skipped,
                             if total_files_skipped == 1 { "" } else { "s" },
                             total_files_failed,
                             if total_files_failed == 1 { "" } else { "s" });
                    println!("{:.1} MB/s ({} bytes in {:.3}s)",
                             total_transfer_rate,
                             total_bytes_transferred,
                             total_duration.as_secs_f64());
                }
                Err(err) => println!("{}", err),
            }
        }
        pull_command if pull_command.starts_with(USER_PULL_COMMAND) => {
            let mut preserve = false;
            let mut pull_args = command_args.clone();

            if !pull_args.is_empty() && pull_args[0] == "-a" {
                preserve = true;
                pull_args.remove(0);
            }

            if pull_args.len() < 2 {
                eprintln!("Error: pull command requires at least two arguments");
                eprintln!("Usage: pull [-a] REMOTE... LOCAL");
                return;
            }

            let local_path = pull_args.pop().unwrap();
            let remote_paths = pull_args;

            match client.adb_pull(device_type.clone(), &remote_paths, &local_path, preserve).await {
                Ok(results) => {
                    let mut total_files_pulled = 0;
                    let mut total_files_failed = 0;
                    let mut total_bytes_transferred = 0;
                    let mut total_duration = std::time::Duration::new(0, 0);

                    for (remote_path, result) in results {
                        match result {
                            Ok(pull_result) => {
                                match pull_result {
                                    PullResult::Success(_, bytes, duration, file_count) => {
                                        println!("{}: {} file{} pulled.", remote_path, file_count, if file_count == 1 { "" } else { "s" });
                                        total_files_pulled += file_count;
                                        total_bytes_transferred += bytes;
                                        total_duration += duration;
                                    }
                                    PullResult::SuccessDirectory(_, bytes, duration, file_count) => {
                                        println!("{}: {} file{} pulled.", remote_path, file_count, if file_count == 1 { "" } else { "s" });
                                        total_files_pulled += file_count;
                                        total_bytes_transferred += bytes;
                                        total_duration += duration;
                                    }
                                    PullResult::FailedAllPull(msg) => {
                                        println!("{}", msg);
                                        total_files_failed += 1;
                                    }
                                }
                            }
                            Err(err) => {
                                println!("{}", err);
                                total_files_failed += 1;
                            }
                        }
                    }

                    let total_transfer_rate = if total_duration.as_secs_f64() > 0.0 {
                        total_bytes_transferred as f64 / total_duration.as_secs_f64() / 1_000_000.0
                    } else {
                        0.0
                    };

                    println!("{} file{} pulled. {} file{} failed.",
                             total_files_pulled,
                             if total_files_pulled == 1 { "" } else { "s" },
                             total_files_failed,
                             if total_files_failed == 1 { "" } else { "s" });
                    println!("{:.1} MB/s ({} bytes in {:.3}s)",
                             total_transfer_rate,
                             total_bytes_transferred,
                             total_duration.as_secs_f64());
                }
                Err(err) => println!("{}", err),
            }
        }
        USER_DISABLE_VERITY_COMMAND => {
            if !command_args.is_empty() {
                eprintln!("Error: disable-verity command does not accept any arguments");
                return;
            }
            match client.adb_disable_verity(device_type).await {
                Ok(result) => {
                    print!("{}", result);
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }
        USER_ENABLE_VERITY_COMMAND => {
            if !command_args.is_empty() {
                eprintln!("Error: enable-verity command does not accept any arguments");
                return;
            }
            match client.adb_enable_verity(device_type).await {
                Ok(result) => {
                    print!("{}", result);
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }
        USER_KEYGEN_COMMAND => {
            let file_path = if command_args.is_empty() {
                None
            } else if command_args.len() == 1 {
                Some(command_args[0].as_str())
            } else {
                eprintln!("Error: keygen command accepts at most one argument (file path)");
                return;
            };

            match client.adb_keygen(file_path) {
                Ok(result) => {
                    println!("{}", result);
                }
                Err(err) => {
                    eprintln!("Error generating ADB key pair: {}", err);
                }
            }
        }
        bugreport_cmd if bugreport_cmd == USER_BUGREPORT_COMMAND => {
            let path = command_args.get(0).map(|s| s.as_str());
            if let Err(err) = client.adb_bugreport(device_type, path).await {
                eprintln!("{}", err);
            }
        }
        logcat_cmd if logcat_cmd == USER_LOGCAT_COMMAND => {
            let logcat_args = command_args.join(" ");
            match client.adb_logcat(device_type, &logcat_args).await {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }
        install_cmd if install_cmd.starts_with(USER_INSTALL_COMMAND) => {
            let mut install_flags = vec![];
            let mut apk_file = String::new();

            for arg in command_args {
                match arg.as_str() {
                    INSTALL_FLAG_REPLACE | INSTALL_FLAG_DOWNGRADE | INSTALL_FLAG_GRANT_PERMISSIONS |
                    INSTALL_FLAG_TEST | INSTALL_FLAG_FORWARD_LOCK | INSTALL_FLAG_SDCARD | INSTALL_FLAG_INTERNAL => {
                        install_flags.push(arg);
                    }
                    _ => {
                        if apk_file.is_empty() {
                            apk_file = arg;
                        } else {
                            eprintln!("Error: Multiple APK files specified");
                            return;
                        }
                    }
                }
            }

            if apk_file.is_empty() {
                eprintln!("Error: No APK file specified");
                return;
            }

            match client.adb_install(device_type, &apk_file, &install_flags).await {
                Ok(result) => {
                    print!("{}", result);
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }
        uninstall_cmd if uninstall_cmd.starts_with(USER_UNINSTALL_COMMAND) => {
            let mut args_iter = command_args.iter();
            let mut uninstall_flags = vec![];
            let mut package_name = String::new();

            while let Some(arg) = args_iter.next() {
                match arg.as_str() {
                    UNINSTALL_FLAG_KEEP_DATA => {
                        uninstall_flags.push(arg.clone());
                    }
                    _ => {
                        package_name = std::iter::once(arg.clone())
                            .chain(args_iter.cloned())
                            .collect::<Vec<String>>()
                            .join(" ");
                        break;
                    }
                }
            }

            if package_name.is_empty() {
                eprintln!("Error: No package name specified");
                return;
            }

            match client.adb_uninstall(device_type.clone(), &package_name, &uninstall_flags).await {
                Ok(result) => {
                    print!("{}", result);
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }
        reboot_command if reboot_command.starts_with(USER_REBOOT_COMMAND) => {
            let reboot_target = command_args.get(0).cloned();
            if command_args.len() > 1 {
                eprintln!("Error: reboot command accepts at most one argument");
                return;
            }
            if let Err(err) = client.adb_reboot(device_type, reboot_target).await {
                eprintln!("{}", err);
            }
        }
        get_devpath_cmd if get_devpath_cmd.starts_with(USER_GET_DEVPATH_COMMAND) => {
            match client.adb_get_devpath(device_type).await {
                Ok(result) => {
                    println!("{}", result);
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }
        serial_no_cmd if serial_no_cmd == USER_SERIALNO_COMMAND => {
            match client.adb_serialno(device_type).await {
                Ok(result) => {
                    println!("{}", result);
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }
        remount_cmd if remount_cmd == USER_REMOUNT_COMMAND => {
            if !command_args.is_empty() {
                eprintln!("Error: remount command does not accept any arguments");
                return;
            }
            match client.adb_remount(device_type).await {
                Ok(result) => {
                    print!("{}", result);
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }
        root_cmd if root_cmd == USER_ROOT_COMMAND => {
            match client.adb_root(device_type).await {
                Ok(result) => {
                    print!("{}", result);
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }
        unroot_cmd if unroot_cmd == USER_UNROOT_COMMAND => {
            match client.adb_unroot(device_type).await {
                Ok(result) => {
                    print!("{}", result);
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }
        USER_USB_COMMAND => {
            if !command_args.is_empty() {
                eprintln!("Error: usb command does not accept any arguments");
                return;
            }
            match client.adb_usb(device_type).await {
                Ok(result) => {
                    print!("{}", result);
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }
        USER_TCPIP_COMMAND => {
            if command_args.len() != 1 {
                eprintln!("Error: tcpip command requires exactly one argument (PORT)");
                return;
            }
            let port = match command_args[0].parse::<u16>() {
                Ok(p) => p,
                Err(_) => {
                    eprintln!("Error: Invalid port number");
                    return;
                }
            };
            match client.adb_tcpip(device_type, port).await {
                Ok(result) => {
                    print!("{}", result);
                }
                Err(err) => {
                    eprintln!("{}", err);
                }
            }
        }
        wait_for_cmd if wait_for_cmd.starts_with(USER_WAIT_FOR_COMMAND) => {
            let command_remainder = &command[USER_WAIT_FOR_COMMAND.len()..];
            let command_remainder = command_remainder.trim_start_matches('-');

            let parts: Vec<&str> = command_remainder.split('-').collect();
            let state = parts.last().unwrap_or(&DEFAULT_WAIT_STATE);

            let mut timeout_duration: Option<Duration> = None;

            let mut args_iter = command_args.iter();

            while let Some(arg) = args_iter.next() {
                if arg == FLAG_TIMEOUT {
                    if let Some(timeout_str) = args_iter.next() {
                        match timeout_str.parse::<u64>() {
                            Ok(seconds) => {
                                timeout_duration = Some(Duration::from_secs(seconds));
                            }
                            Err(_) => {
                                eprintln!("Invalid timeout value: {}", timeout_str);
                                return;
                            }
                        }
                    } else {
                        eprintln!("No timeout value provided after {}", FLAG_TIMEOUT);
                        return;
                    }
                } else {
                    eprintln!("Unknown option: {}", arg);
                    return;
                }
            }

            println!("Waiting for device to reach '{}' state...", state);

            match client.adb_wait_for(device_type, state, timeout_duration).await {
                Ok(_) => println!("Device is now in '{}' state", state),
                Err(err) => eprintln!("{}", err),
            }
        }

        USER_GET_STATE_COMMAND => {
            match client.adb_get_state(device_type).await {
                Ok(state) => println!("{}", state),
                Err(err) => eprintln!("{}", err),
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
    client.close().await;
}

