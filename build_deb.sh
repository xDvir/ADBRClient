#!/bin/bash

set -e

# Version configuration - change this in one place
VERSION="1.0.0-1"

# Ensure we're in the project root
cd "$(dirname "$0")"

# Create releases directory if it doesn't exist
mkdir -p releases

# Build the project
cargo build --release

# Define package name and temporary directory
PKG_NAME="adbr_${VERSION}"
TMP_DIR="${PKG_NAME}"

# Create package directories
mkdir -p ${TMP_DIR}/DEBIAN
mkdir -p ${TMP_DIR}/usr/local/bin

# Create control file
cat > ${TMP_DIR}/DEBIAN/control << EOL
Package: adbr
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: amd64
Depends: libc6 (>= 2.17)
Maintainer: Your Name <your.email@example.com>
Description: ADB Rust Implementation
 A Rust implementation of the Android Debug Bridge (ADB) client.
 This package provides the adbr command line tool.
EOL

# Create postinst script
cat > ${TMP_DIR}/DEBIAN/postinst << EOL
#!/bin/sh
set -e

# Set correct permissions
chmod 755 /usr/local/bin/adbr

# Add /usr/local/bin to PATH if not already there
if ! echo \$PATH | grep -q "/usr/local/bin"; then
    echo 'export PATH="/usr/local/bin:\$PATH"' >> /etc/profile
    echo "Added /usr/local/bin to PATH. Please restart your shell or run 'source /etc/profile' to apply changes."
fi

echo "adbr has been installed. You can now use it by running 'adbr' from anywhere in your system."
EOL

chmod 755 ${TMP_DIR}/DEBIAN/postinst

# Copy binary
cp target/release/adbr ${TMP_DIR}/usr/local/bin/

# Build the package
dpkg-deb --build ${TMP_DIR}

# Move the .deb file to releases directory
mv ${TMP_DIR}.deb releases/

# Clean up temporary directory
rm -rf ${TMP_DIR}

echo "Debian package created: releases/${PKG_NAME}.deb"

# Install the package
echo "Installing the package..."
if [ "$EUID" -ne 0 ]; then
    echo "This script needs root privileges to install the package."
    sudo dpkg -i releases/${PKG_NAME}.deb
    if [ $? -ne 0 ]; then
        echo "Installation failed. Attempting to resolve dependencies..."
        sudo apt-get install -f
        sudo dpkg -i releases/${PKG_NAME}.deb
    fi
else
    dpkg -i releases/${PKG_NAME}.deb
    if [ $? -ne 0 ]; then
        echo "Installation failed. Attempting to resolve dependencies..."
        apt-get install -f
        dpkg -i releases/${PKG_NAME}.deb
    fi
fi

if [ $? -eq 0 ]; then
    echo "Installation completed successfully!"
    echo "You may need to restart your shell or run 'source /etc/profile' to update your PATH."
else
    echo "Installation failed. Please check the error messages above."
fi