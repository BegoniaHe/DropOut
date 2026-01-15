#!/bin/bash
# fix AppImage for Wayland compatibility
# This script modifies the AppImage bundle created by Tauri to ensure compatibility with Wayland
# It specifically targets the GTK backend settings to avoid forcing X11

set -e

echo "Fixing AppImage for Wayland compatibility..."

# Tauri sets the APPIMAGE_BUNDLE_PATH environment variable during the build process
APPDIR_PATH="${APPIMAGE_BUNDLE_PATH:-}"

if [ -z "$APPDIR_PATH" ]; then
    echo "No AppImage bundle path found, skipping fix"
    exit 0
fi

# Check for the presence of the GTK hook file
if [ -d "$APPDIR_PATH/apprun-hooks" ]; then
    HOOK_FILE="$APPDIR_PATH/apprun-hooks/linuxdeploy-plugin-gtk.sh"
    
    if [ -f "$HOOK_FILE" ]; then
        echo "Found GTK hook file, patching..."
        
        # Comment out the line that forces GDK_BACKEND to x11
        sed -i 's/^export GDK_BACKEND=x11.*$/# export GDK_BACKEND=x11  # Disabled for Wayland compatibility/' "$HOOK_FILE"
        
        echo "Successfully patched $HOOK_FILE"
    fi
fi

echo "AppImage Wayland fix completed!"
