# ADBR

A modern implementation of the Android Debug Bridge (ADB) client written in Rust, focusing on code maintainability and better error handling. Currently optimized for Ubuntu systems.

## Why ADBR?

- 📝 Clean, modern Rust implementation compared to AOSP's C-based ADB
- ✨ Improved error handling and user feedback
- 🚀 Well-structured and maintainable codebase
- 🔍 Easy to understand and modify

Key improvements over traditional ADB:
- Clear separation of client/server communication
- Modern error handling patterns
- Well-structured command processing

Coming Soon: A full Rust implementation of the ADB server!

## Requirements

- ADB server (running on default port 5037 or custom)
- Android device with USB debugging enabled
- Ubuntu 20.04 or newer

## Installation

### Option 1: Install from DEB Package (Recommended)
```bash
# Download and install
wget https://raw.githubusercontent.com/xDvir/ADBRClient/main/releases/adbr_1.0.0-1.deb
sudo dpkg -i adbr_1.0.0-1.deb

# If needed, resolve dependencies
sudo apt-get install -f
```

### Option 2: Build from Source
```bash
# Install build dependencies
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev musl-tools

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add musl target
rustup target add x86_64-unknown-linux-musl

# Clone and build
git clone https://github.com/xDvir/ADBRClient.git
cd ADBRClient
./build_deb.sh
```

## Basic Usage

```bash
# List devices
adbr devices

# File operations
adbr push local_file.txt /sdcard/
adbr pull /sdcard/remote_file.txt ./

# Shell commands
adbr shell "ls -l"

# App management
adbr install app.apk
adbr uninstall package.name

# Network
adbr forward tcp:8000 tcp:8001
```

## Configuration

Set custom ADB server:
```bash
# Environment variable
export ADB_ADDRESS=192.168.1.100:5037

# Or command-line
adbr -H 192.168.1.100 -P 5037 devices
```

## Available Commands

### Device Management
```bash
adbr devices         # List connected devices
adbr devices -w      # Monitor devices continuously
adbr wait-for-device # Wait for device to connect
adbr get-state      # Get device state
```

### File Operations
```bash
adbr push SOURCE TARGET    # Copy to device
adbr pull SOURCE TARGET    # Copy from device
```

### App Management
```bash
adbr install APP.apk      # Install an app
adbr uninstall PACKAGE    # Remove an app
```

### Network
```bash
adbr forward LOCAL REMOTE  # Forward ports
adbr reverse REMOTE LOCAL  # Reverse forward ports
```

### System
```bash
adbr root                 # Restart ADB with root
adbr unroot              # Restart ADB without root
adbr reboot              # Reboot device
adbr shell               # Start shell session
```

## Notes

- Compatible with Ubuntu 20.04 and newer
- Works with all Android devices that support ADB
- Requires an ADB server to be running
- USB debugging must be enabled on Android devices
- Binary location: `/usr/local/bin/adbr`

## Contributing

Found a bug or want to contribute? Open an issue or submit a pull request!

## License

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

## Version History

1.0.0-1: Initial release
- Complete ADB command set implementation
- Ubuntu package support
- Command-line interface parity with ADB

## Related Publications

- [adbDocumentation](https://github.com/cstyan/adbDocumentation)
- [python-adb](https://github.com/google/python-adb)
- [paramiko-shell](https://github.com/sirosen/paramiko-shell/blob/master/interactive_shell.py)