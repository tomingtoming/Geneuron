#!/bin/bash
set -e

# Configuration
APP_NAME="Geneuron"
BUNDLE_ID="app.toming.geneuron"
VERSION=$(grep version Cargo.toml | head -n 1 | cut -d '"' -f 2)
BINARY_PATH="./target/release/geneuron"
BUNDLE_PATH="./target/$APP_NAME.app"

# Build release binary
cargo build --release

# Create app bundle structure
mkdir -p "$BUNDLE_PATH/Contents/"{MacOS,Resources}

# Copy binary
cp "$BINARY_PATH" "$BUNDLE_PATH/Contents/MacOS/geneuron-rs"

# Copy icon if exists
if [ -f "icon.icns" ]; then
    cp "icon.icns" "$BUNDLE_PATH/Contents/Resources/"
fi

# Create Info.plist
cat > "$BUNDLE_PATH/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundleDisplayName</key>
    <string>$APP_NAME</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleExecutable</key>
    <string>geneuron-rs</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

echo "Created app bundle at $BUNDLE_PATH"