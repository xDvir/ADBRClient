use adbr::{Client, PullResult, PushResult};
use adbr::DeviceTransport;
use std::error::Error;
use std::process::Command;
use tokio;
use std::fs;
use std::path::Path;
use ctor::ctor;

#[ctor]
fn init() {
    std::env::set_var("RUST_TEST_THREADS", "1");
}


fn run_adb_command(args: &[&str]) -> Result<String, Box<dyn Error>> {
    let cmd_string = format!("adbr {}", args.join(" "));
    println!("Executing: {}", cmd_string);

    let output = Command::new("adbr")
        .args(args)
        .output()?;

    if output.status.success() {
        let result = String::from_utf8(output.stdout)?;
        Ok(result)
    } else {
        let error = String::from_utf8(output.stderr)?;
        Err(error.into())
    }
}

#[tokio::test]
async fn test_adb_devices() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(None, None).await?;
    let result = client.adb_devices().await?;
    println!("result {}", result);
    assert!(result.contains("List of devices attached"));

    client.close().await;

    Ok(())
}

#[tokio::test]
async fn test_adb_devices_with_device() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(None, None).await?;
    let result = client.adb_devices().await?;
    println!("result: {}", result);

    if !result.contains("List of devices attached") {
        return Err("ADB not responding correctly".into());
    }

    let lines: Vec<&str> = result.lines().collect();

    let devices: Vec<&str> = lines.iter()
        .skip(1)
        .map(|&line| line)
        .filter(|line| !line.trim().is_empty())
        .collect();

    if devices.is_empty() {
        return Err("No devices connected".into());
    }

    // Check if any device is in 'device' state
    let ready_devices: Vec<&str> = devices.iter()
        .filter(|&&line| line.contains("device"))
        .copied()
        .collect();

    if ready_devices.is_empty() {
        return Err("No authorized devices found".into());
    }

    println!("Found {} ready device(s)", ready_devices.len());
    for device in ready_devices {
        println!("Device: {}", device);
    }

    client.close().await;

    Ok(())
}

#[tokio::test]
async fn test_adb_shell() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(None, None).await?;
    let result = client.adb_shell(DeviceTransport::default(), "echo test").await?;


    assert_eq!(result.trim(), "test");

    client.close().await;

    Ok(())
}


#[tokio::test]
async fn test_apk_installation() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(None, None).await?;
    let package_name = "com.example.myapplication";

    let project_root = env!("CARGO_MANIFEST_DIR");
    let apk_path = std::path::Path::new(project_root)
        .join("tests")
        .join("resources")
        .join("test_app.apk")
        .to_str()
        .ok_or("Failed to convert APK path to string")?
        .to_string();

    let _ = client.adb_uninstall(DeviceTransport::default(), package_name, &[]).await;
    client.reconnect().await?;

    let install_options = vec![
        "-r".to_string(),
        "-d".to_string(),
        "-g".to_string(),
    ];

    let install_result = client
        .adb_install(DeviceTransport::default(), &apk_path, &install_options)
        .await?;

    if !install_result.contains("Success") {
        return Err(format!("Installation failed: {}", install_result).into());
    }

    client.reconnect().await?;

    let packages = client
        .adb_shell(DeviceTransport::default(), "pm list packages")
        .await?;
    assert!(
        packages.contains(package_name),
        "Package not found after installation"
    );

    client.reconnect().await?;

    let uninstall_result = client
        .adb_uninstall(DeviceTransport::default(), package_name, &[])
        .await?;

    if !uninstall_result.contains("Success") {
        return Err(format!("Uninstallation failed: {}", uninstall_result).into());
    }

    client.reconnect().await?;

    let packages_after = client
        .adb_shell(DeviceTransport::default(), "pm list packages")
        .await?;
    assert!(
        !packages_after.contains(package_name),
        "Package still exists after uninstallation"
    );

    client.close().await;
    Ok(())
}

#[tokio::test]
async fn test_adb_get_state() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(None, None).await?;
    let state = client.adb_get_state(DeviceTransport::default()).await?;
    assert!(!state.is_empty());

    client.close().await;

    Ok(())
}


#[tokio::test]
async fn test_adb_reverse() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(None, None).await?;

    client.send_reverse_command_set(DeviceTransport::default(), "tcp:8000", "tcp:9000", false).await?;

    client.reconnect().await?;

    let reverse_list = client.send_reverse_command_list(DeviceTransport::default()).await?;

    assert!(reverse_list.contains("tcp:8000"));

    client.reconnect().await?;

    client.send_reverse_command_remove(DeviceTransport::default(), "tcp:8000").await?;

    Ok(())
}


#[tokio::test]
async fn test_adb_screencap() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(None, None).await?;
    println!("Starting screencap test...");

    // Take screenshot
    client.adb_shell(DeviceTransport::default(), "screencap -p /data/local/tmp/screenshot.png").await?;
    client.reconnect().await?;


    client.adb_pull(
        DeviceTransport::default(),
        &["/data/local/tmp/screenshot.png".to_string()],
        "local_screenshot.png",
        false,
    ).await?;
    client.reconnect().await?;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let metadata = std::fs::metadata("local_screenshot.png")?;
    assert!(metadata.len() > 0, "Screenshot file is empty");

    // Cleanup
    std::fs::remove_file("local_screenshot.png")?;

    client.adb_shell(DeviceTransport::default(), "rm /data/local/tmp/screenshot.png").await?;
    client.reconnect().await?;

    client.close().await;

    Ok(())
}

#[tokio::test]
async fn test_adb_battery_status() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(None, None).await?;

    let battery_info = client.adb_shell(DeviceTransport::default(), "dumpsys battery").await?;
    client.reconnect().await?;

    // Verify basic battery information is present
    assert!(battery_info.contains("level:"));
    assert!(battery_info.contains("scale:"));
    assert!(battery_info.contains("status:"));

    client.close().await;

    Ok(())
}

#[tokio::test]
async fn test_adb_package_manager() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(None, None).await?;

    // List packages
    let packages = client.adb_shell(DeviceTransport::default(), "pm list packages -f").await?;
    client.reconnect().await?;

    // Check that we have a meaningful number of packages (typical Android devices have >50)
    let package_count = packages.lines().filter(|line| line.starts_with("package:")).count();
    assert!(
        package_count > 50,
        "Found only {} packages, expected more than 50 on a typical Android device",
        package_count
    );

    // Check for some essential packages that should exist on any Android device
    let essential_packages = [
        "android", // Framework
        "com.android.systemui", // System UI
        "com.android.settings", // Settings
    ];

    for package in essential_packages {
        assert!(
            packages.contains(&format!(".apk={}", package)),
            "Essential package '{}' not found",
            package
        );
    }

    client.close().await;
    Ok(())
}

#[tokio::test]
async fn test_adb_multiple_devices() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(None, None).await?;

    // Get list of devices
    let devices_output = client.adb_devices().await?;
    client.reconnect().await?;

    // Parse the devices from output
    let device_lines: Vec<&str> = devices_output
        .lines()
        .skip(1) // Skip "List of devices attached" header
        .filter(|line| !line.is_empty())
        .collect();

    // Skip test if only one device is connected
    if device_lines.len() <= 1 {
        println!("Skipping test: not enough devices connected");
        return Ok(());
    }

    // Get first device serial
    if let Some(device_line) = device_lines.first() {
        let device_serial = device_line.split_whitespace().next()
            .ok_or("Failed to parse device serial")?;

        let transport = DeviceTransport::usb(device_serial.to_string());

        // Try a simple command with the specific device
        let result = client
            .adb_shell(transport, "echo test")
            .await?;

        assert!(!result.is_empty(), "Command execution failed");
    }

    client.close().await;
    Ok(())
}

#[tokio::test]
async fn test_adb_error_handling() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(None, None).await?;

    let project_root = env!("CARGO_MANIFEST_DIR");
    let nonexistent_file = std::path::Path::new(project_root)
        .join("tests")
        .join("resources")
        .join("nonexistent_file")
        .to_str()
        .ok_or("Failed to convert path to string")?
        .to_string();

    let result = client
        .adb_shell(DeviceTransport::default(), "invalid_command")
        .await?;
    client.reconnect().await?;

    assert!(result.contains("inaccessible or not found"));

    // Test invalid device serial
    let result = client
        .adb_shell(DeviceTransport::usb("nonexistent".to_string()), "echo test")
        .await;
    client.reconnect().await?;

    assert!(result.is_err());

    let result = client
        .adb_pull(
            DeviceTransport::default(),
            &["/nonexistent/file".to_string()],
            &nonexistent_file,
            false,
        )
        .await;

    client.reconnect().await?;

    assert!(result.is_err());

    if std::path::Path::new(&nonexistent_file).exists() {
        std::fs::remove_file(&nonexistent_file)?;
    }

    client.close().await;

    Ok(())
}

#[tokio::test]
async fn test_adb_push_single_file() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(None, None).await?;

    // Create a test file
    let test_content = "test content";
    std::fs::write("test_file.txt", test_content)?;

    println!("Pushing single file to device...");
    let result = client.adb_push(
        DeviceTransport::default(),
        &["test_file.txt".to_string()],
        "/data/local/tmp/",
        false,
    ).await?;

    client.reconnect().await?;

    // Verify push result
    for (path, result) in result {
        match result {
            Ok(push_result) => {
                println!("Push result for {}: {:?}", path, push_result);
                assert!(matches!(push_result, PushResult::Success(_, _, _, 1)));
            }
            Err(e) => panic!("Push failed: {}", e),
        }
    }

    // Cleanup
    std::fs::remove_file("test_file.txt")?;
    client.adb_shell(DeviceTransport::default(), "rm /data/local/tmp/test_file.txt").await?;

    client.close().await;
    Ok(())
}

#[tokio::test]
async fn test_adb_logcat_clear_and_dump() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(None, None).await?;

    println!("Clearing logcat...");
    client.adb_logcat(DeviceTransport::default(), "-c").await?;
    client.reconnect().await?;

    // Sleep briefly to ensure logcat clear takes effect
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Get logcat content using shell command instead
    let result = client.adb_shell(DeviceTransport::default(), "logcat -d").await?;
    client.reconnect().await?;

    // Count the lines in the output
    let line_count = result.lines().count();

    // Should be relatively small after clear
    assert!(line_count < 1000, "Logcat should have fewer lines after clear, got {}", line_count);

    client.close().await;
    Ok(())
}

#[tokio::test]
async fn test_adb_forward_commands() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(None, None).await?;

    println!("Testing port forwarding...");

    // Setup forward
    client.send_forward_command_set(
        DeviceTransport::default(),
        "tcp:8000",
        "tcp:8001",
        false,
    ).await?;
    client.reconnect().await?;

    // List forwards
    let list = client.send_forward_command_list(DeviceTransport::default()).await?;
    assert!(list.contains("8000"), "Forward not found in list");
    println!("Forward list: {}", list);
    client.reconnect().await?;

    // Remove forward
    client.send_forward_command_remove(
        DeviceTransport::default(),
        "tcp:8000",
    ).await?;
    client.reconnect().await?;

    // Verify removal
    let list_after = client.send_forward_command_list(DeviceTransport::default()).await?;
    assert!(!list_after.contains("8000"), "Forward still exists after removal");

    client.close().await;
    Ok(())
}

#[tokio::test]
async fn test_adb_device_info() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(None, None).await?;

    println!("Getting device information...");

    // Get device serial
    let serial = client.adb_serialno(DeviceTransport::default()).await?;
    assert!(!serial.trim().is_empty(), "Serial number should not be empty");
    println!("Device serial: {}", serial);
    client.reconnect().await?;

    // Get device path
    let devpath = client.adb_get_devpath(DeviceTransport::default()).await?;
    assert!(!devpath.trim().is_empty(), "Device path should not be empty");
    println!("Device path: {}", devpath);
    client.reconnect().await?;

    // Get some basic device properties
    let props = client.adb_shell(
        DeviceTransport::default(),
        "getprop ro.product.model && getprop ro.build.version.release",
    ).await?;
    println!("Device properties: {}", props);
    assert!(!props.trim().is_empty(), "Device properties should not be empty");

    client.close().await;
    Ok(())
}

#[tokio::test]
async fn test_adb_push_pull_large_file() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(None, None).await?;

    let project_root = env!("CARGO_MANIFEST_DIR");
    let test_dir = std::path::Path::new(project_root)
        .join("tests")
        .join("resources");

    // Create a large file (1MB)
    let large_file_path = test_dir.join("large_test_file.dat");
    let large_data = vec![0x55; 1024 * 1024]; // 1MB of data
    std::fs::write(&large_file_path, &large_data)?;

    // Push large file
    let push_results = client.adb_push(
        DeviceTransport::default(),
        &[large_file_path.to_str().unwrap().to_string()],
        "/data/local/tmp/large_test_file.dat",
        false,
    ).await?;

    for (_, result) in &push_results {
        match result {
            Ok(push_result) => {
                assert!(
                    matches!(push_result, PushResult::Success(_, bytes, _, 1) if *bytes == 1024 * 1024),
                    "Push failed or wrong file size"
                );
            }
            Err(e) => panic!("Push failed: {}", e),
        }
    }

    client.reconnect().await?;

    // Pull and verify
    let pull_path = test_dir.join("pulled_large_file.dat");
    let pull_results = client.adb_pull(
        DeviceTransport::default(),
        &["/data/local/tmp/large_test_file.dat".to_string()],
        pull_path.to_str().unwrap(),
        false,
    ).await?;

    // Verify pull results
    for (_, result) in &pull_results {
        match result {
            Ok(pull_result) => {
                assert!(
                    matches!(pull_result, PullResult::Success(_, bytes, _, 1) if *bytes == 1024 * 1024),
                    "Pull failed or wrong file size"
                );
            }
            Err(e) => panic!("Pull failed: {}", e),
        }
    }

    client.reconnect().await?;

    // Cleanup
    std::fs::remove_file(large_file_path)?;
    std::fs::remove_file(pull_path)?;
    client.adb_shell(DeviceTransport::default(), "rm /data/local/tmp/large_test_file.dat").await?;

    client.close().await;
    Ok(())
}

#[tokio::test]
async fn test_adb_push_pull_multiple_files() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(None, None).await?;

    let project_root = env!("CARGO_MANIFEST_DIR");
    let test_dir = std::path::Path::new(project_root)
        .join("tests")
        .join("resources");

    // Create multiple test files
    let file_contents = [
        ("test1.txt", "content1\n"),
        ("test2.txt", "content2\n"),
        ("test3.txt", "content3\n"),
    ];

    let file_paths: Vec<String> = file_contents
        .iter()
        .map(|(name, content)| {
            let path = test_dir.join(name);
            std::fs::write(&path, content).unwrap();
            path.to_str().unwrap().to_string()
        })
        .collect();

    // Push all files at once
    let push_results = client.adb_push(
        DeviceTransport::default(),
        &file_paths,
        "/data/local/tmp/",
        false,
    ).await?;

    for (_, result) in &push_results {
        match result {
            Ok(push_result) => {
                assert!(
                    matches!(push_result, PushResult::Success(_, _, _, 1)),
                    "Push failed"
                );
            }
            Err(e) => panic!("Push failed: {}", e),
        }
    }

    // Create pulled files directory
    let pull_dir = test_dir.join("pulled_files");
    std::fs::create_dir_all(&pull_dir)?;

    // Pull all files at once
    let remote_paths: Vec<String> = file_contents
        .iter()
        .map(|(name, _)| format!("/data/local/tmp/{}", name))
        .collect();

    client.reconnect().await?;

    let pull_results = client.adb_pull(
        DeviceTransport::default(),
        &remote_paths,
        pull_dir.to_str().unwrap(),
        false,
    ).await?;

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Verify all pulls succeeded
    for (path, result) in &pull_results {
        match result {
            Ok(pull_result) => {
                assert!(
                    matches!(pull_result, PullResult::Success(_, _, _, 1)),
                    "Pull failed for {}", path
                );
            }
            Err(e) => panic!("Pull failed for {}: {}", path, e),
        }
    }

    // Verify content of all files
    for (name, content) in &file_contents {
        let pulled_content = std::fs::read_to_string(pull_dir.join(name))?;
        assert_eq!(
            pulled_content,
            *content,
            "Content mismatch for file: {}",
            name
        );
    }


    // Cleanup
    for (name, _) in &file_contents {
        client.reconnect().await?;
        std::fs::remove_file(test_dir.join(name))?;
        std::fs::remove_file(pull_dir.join(name))?;
        client.adb_shell(
            DeviceTransport::default(),
            &format!("rm /data/local/tmp/{}", name),
        ).await?;
    }
    std::fs::remove_dir(pull_dir)?;

    client.close().await;
    Ok(())
}

#[tokio::test]
async fn test_adb_push_pull_binary_file() -> Result<(), Box<dyn Error>> {
    let mut client = Client::new(None, None).await?;

    let project_root = env!("CARGO_MANIFEST_DIR");
    let test_dir = std::path::Path::new(project_root)
        .join("tests")
        .join("resources");

    let binary_data: Vec<u8> = (0..1024).map(|_| rand::random::<u8>()).collect();
    let binary_file = test_dir.join("test.bin");
    std::fs::write(&binary_file, &binary_data)?;

    let push_results = client.adb_push(
        DeviceTransport::default(),
        &[binary_file.to_str().unwrap().to_string()],
        "/data/local/tmp/test.bin",
        false,
    ).await?;

    for (_, result) in &push_results {
        match result {
            Ok(push_result) => {
                assert!(
                    matches!(push_result, PushResult::Success(_, bytes, _, 1) if *bytes == 1024),
                    "Push failed or wrong file size"
                );
            }
            Err(e) => panic!("Push failed: {}", e),
        }
    }

    client.reconnect().await?;


    let pulled_file = test_dir.join("pulled.bin");
    let pull_results = client.adb_pull(
        DeviceTransport::default(),
        &["/data/local/tmp/test.bin".to_string()],
        pulled_file.to_str().unwrap(),
        false,
    ).await?;

    for (_, result) in &pull_results {
        match result {
            Ok(pull_result) => {
                assert!(
                    matches!(pull_result, PullResult::Success(_, bytes, _, 1) if *bytes == 1024),
                    "Pull failed or wrong file size"
                );
            }
            Err(e) => panic!("Pull failed: {}", e),
        }
    }

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // Verify content matches
    let pulled_data = std::fs::read(&pulled_file)?;
    assert_eq!(pulled_data, binary_data, "Binary content mismatch");

    // Cleanup
    std::fs::remove_file(binary_file)?;
    std::fs::remove_file(pulled_file)?;

    client.reconnect().await?;

    client.adb_shell(DeviceTransport::default(), "rm /data/local/tmp/test.bin").await?;

    client.close().await;
    Ok(())
}

#[tokio::test]
async fn cli_test_list_devices() -> Result<(), Box<dyn Error>> {
    let result = run_adb_command(&["devices"])?;
    println!("result {}", result);
    assert!(result.contains("List of devices attached"));
    Ok(())
}

#[tokio::test]
async fn cli_test_device_status() -> Result<(), Box<dyn Error>> {
    let result = run_adb_command(&["devices"])?;
    println!("result: {}", result);

    if !result.contains("List of devices attached") {
        return Err("ADB not responding correctly".into());
    }

    let lines: Vec<&str> = result.lines().collect();
    let devices: Vec<&str> = lines.iter()
        .skip(1)
        .map(|&line| line)
        .filter(|line| !line.trim().is_empty())
        .collect();

    if devices.is_empty() {
        return Err("No devices connected".into());
    }

    let ready_devices: Vec<&str> = devices.iter()
        .filter(|&&line| line.contains("device"))
        .copied()
        .collect();

    if ready_devices.is_empty() {
        return Err("No authorized devices found".into());
    }

    println!("Found {} ready device(s)", ready_devices.len());
    for device in ready_devices {
        println!("Device: {}", device);
    }

    Ok(())
}

#[tokio::test]
async fn cli_test_shell_echo() -> Result<(), Box<dyn Error>> {
    let result = run_adb_command(&["shell", "echo", "test"])?;
    assert_eq!(result.trim(), "test");
    Ok(())
}

#[tokio::test]
async fn cli_test_file_transfer() -> Result<(), Box<dyn Error>> {
    println!("Starting push/pull test...");

    let project_root = env!("CARGO_MANIFEST_DIR");
    let test_dir = Path::new(project_root).join("tests").join("resources");
    fs::create_dir_all(&test_dir)?;

    // Create test file
    let test_content = "test_file\n";
    let local_path = test_dir.join("test_file.txt");
    fs::write(&local_path, test_content)?;

    let remote_path = "/data/local/tmp/test_file.txt";

    // Push file
    run_adb_command(&[
        "push",
        local_path.to_str().unwrap(),
        remote_path
    ])?;

    // Verify file exists and content
    let ls_output = run_adb_command(&[
        "shell",
        "ls",
        "-l",
        remote_path
    ])?;

    let cat_output = run_adb_command(&[
        "shell",
        "cat",
        remote_path
    ])?;

    assert!(!ls_output.contains("No such file"));
    assert_eq!(cat_output.trim(), test_content.trim());

    // Pull file
    let pull_path = test_dir.join("pulled_test_file.txt");
    run_adb_command(&[
        "pull",
        remote_path,
        pull_path.to_str().unwrap()
    ])?;

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let pulled_content = fs::read_to_string(&pull_path)?;
    assert_eq!(pulled_content, test_content);

    fs::remove_file(&local_path)?;
    fs::remove_file(&pull_path)?;
    run_adb_command(&["shell", "rm", remote_path])?;

    Ok(())
}

#[tokio::test]
async fn cli_test_apk_install() -> Result<(), Box<dyn Error>> {
    let package_name = "com.example.myapplication";
    let project_root = env!("CARGO_MANIFEST_DIR");
    let apk_path = Path::new(project_root)
        .join("tests")
        .join("resources")
        .join("test_app.apk");

    // Uninstall if exists
    let _ = run_adb_command(&["uninstall", package_name]);

    // Install APK
    let install_result = run_adb_command(&[
        "install",
        apk_path.to_str().unwrap()
    ])?;

    assert!(install_result.contains("Success"));

    // // Verify installation
    // let packages = run_adb_command(&["shell", "pm", "list", "packages"])?;
    // assert!(packages.contains(package_name));
    //
    // // Uninstall
    // let uninstall_result = run_adb_command(&["uninstall", package_name])?;
    // assert!(uninstall_result.contains("Success"));

    Ok(())
}

#[tokio::test]
async fn cli_test_screenshot() -> Result<(), Box<dyn Error>> {
    println!("Starting screencap test...");

    // Take screenshot
    run_adb_command(&[
        "shell",
        "screencap",
        "-p",
        "/data/local/tmp/screenshot.png"
    ])?;

    // Pull screenshot
    run_adb_command(&[
        "pull",
        "/data/local/tmp/screenshot.png",
        "local_screenshot.png"
    ])?;

    // Verify file exists and has content
    let metadata = fs::metadata("local_screenshot.png")?;
    assert!(metadata.len() > 0);

    // Cleanup
    fs::remove_file("local_screenshot.png")?;
    run_adb_command(&["shell", "rm", "/data/local/tmp/screenshot.png"])?;

    Ok(())
}


// #[tokio::test]
// async fn cli_test_logcat_operations() -> Result<(), Box<dyn Error>> {
//     // Clear logcat
//     run_adb_command(&["logcat", "-c"])?;
//
//     // Wait briefly
//     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
//
//     // Dump logcat
//     let result = run_adb_command(&["logcat", "-d"])?;
//     let line_count = result.lines().count();
//     assert!(line_count < 1000, "Logcat should have fewer lines after clear");
//
//     Ok(())
// }

#[tokio::test]
async fn cli_test_port_forwarding() -> Result<(), Box<dyn Error>> {
    run_adb_command(&["forward", "tcp:9000", "tcp:9001"])?;

    // List forwards
    let list = run_adb_command(&["forward", "--list"])?;
    assert!(list.contains("9000"));

    // Remove forward
    run_adb_command(&["forward", "--remove", "tcp:9000"])?;

    // Verify removal
    let list_after = run_adb_command(&["forward", "--list"])?;
    assert!(!list_after.contains("9000"));

    Ok(())
}

#[tokio::test]
async fn cli_test_device_properties() -> Result<(), Box<dyn Error>> {
    // Get device serial
    let serial = run_adb_command(&["get-serialno"])?;
    assert!(!serial.trim().is_empty());

    // Get device properties
    let props = run_adb_command(&[
        "shell",
        "getprop ro.product.model && getprop ro.build.version.release"
    ])?;
    assert!(!props.trim().is_empty());

    Ok(())
}