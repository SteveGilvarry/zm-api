#!/bin/bash
# ZM API Installation Script
# Creates user, directories, and installs systemd service

set -e

# Configuration
ZM_USER="zoneminder"
ZM_GROUP="zoneminder"
ZM_HOME="/var/lib/zoneminder"
ZM_EVENTS="/var/cache/zoneminder/events"
ZM_IMAGES="/var/cache/zoneminder/images"
ZM_TEMP="/var/cache/zoneminder/temp"
ZM_RUN="/run/zm"
CONFIG_DIR="/etc/zm_api"
STATIC_DIR="/usr/share/zm_api/static"
BIN_PATH="/usr/bin/zm_api"

echo "=== ZM API Installation ==="

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo "This script must be run as root"
   exit 1
fi

# Create zoneminder group if it doesn't exist
if ! getent group "$ZM_GROUP" > /dev/null 2>&1; then
    echo "Creating group: $ZM_GROUP"
    groupadd --system "$ZM_GROUP"
fi

# Create zoneminder user if it doesn't exist
if ! id "$ZM_USER" > /dev/null 2>&1; then
    echo "Creating user: $ZM_USER"
    useradd --system \
        --gid "$ZM_GROUP" \
        --groups video \
        --home-dir "$ZM_HOME" \
        --shell /usr/sbin/nologin \
        --comment "ZoneMinder Daemon" \
        "$ZM_USER"
fi

# Add user to video group (for camera access)
echo "Adding $ZM_USER to video group"
usermod -aG video "$ZM_USER"

# Create directories
echo "Creating directories..."
mkdir -p "$ZM_HOME"
mkdir -p "$ZM_EVENTS"
mkdir -p "$ZM_IMAGES"
mkdir -p "$ZM_TEMP"
mkdir -p "$CONFIG_DIR"
mkdir -p "$STATIC_DIR"

# Set ownership
echo "Setting directory ownership..."
chown -R "$ZM_USER:$ZM_GROUP" "$ZM_HOME"
chown -R "$ZM_USER:$ZM_GROUP" "$ZM_EVENTS"
chown -R "$ZM_USER:$ZM_GROUP" "$ZM_IMAGES"
chown -R "$ZM_USER:$ZM_GROUP" "$ZM_TEMP"
chown -R "$ZM_USER:$ZM_GROUP" "$CONFIG_DIR"
chown -R "$ZM_USER:$ZM_GROUP" "$STATIC_DIR"

# Set permissions
chmod 755 "$ZM_HOME"
chmod 755 "$ZM_EVENTS"
chmod 755 "$ZM_IMAGES"
chmod 755 "$ZM_TEMP"
chmod 750 "$CONFIG_DIR"

# Install binary (if provided as argument or in current directory)
if [[ -f "${1:-./target/release/zm_api}" ]]; then
    echo "Installing binary to $BIN_PATH"
    cp "${1:-./target/release/zm_api}" "$BIN_PATH"
    chmod 755 "$BIN_PATH"
else
    echo "Note: Binary not found. Copy zm_api to $BIN_PATH manually."
fi

# Install systemd service
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -f "$SCRIPT_DIR/systemd/zm_api.service" ]]; then
    echo "Installing systemd service..."
    cp "$SCRIPT_DIR/systemd/zm_api.service" /etc/systemd/system/
fi

# Install environment file (if not exists, to preserve local changes)
if [[ ! -f "$CONFIG_DIR/zm_api.env" ]] && [[ -f "$SCRIPT_DIR/systemd/zm_api.env" ]]; then
    echo "Installing environment file..."
    cp "$SCRIPT_DIR/systemd/zm_api.env" "$CONFIG_DIR/"
    chown "$ZM_USER:$ZM_GROUP" "$CONFIG_DIR/zm_api.env"
    chmod 640 "$CONFIG_DIR/zm_api.env"
else
    echo "Note: $CONFIG_DIR/zm_api.env exists, not overwriting."
fi

# Reload systemd
echo "Reloading systemd..."
systemctl daemon-reload

echo ""
echo "=== Installation Complete ==="
echo ""
echo "Next steps:"
echo "  1. Configure database connection in $CONFIG_DIR/zm_api.env"
echo "  2. Copy configuration files to $CONFIG_DIR/"
echo "  3. Enable and start the service:"
echo "     sudo systemctl enable zm_api"
echo "     sudo systemctl start zm_api"
echo ""
echo "  Check status with:"
echo "     sudo systemctl status zm_api"
echo "     sudo journalctl -u zm_api -f"
