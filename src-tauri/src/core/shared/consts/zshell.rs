/// Zshell flag, that needs to put at the top of a zshell scirpt.
#[cfg(target_os = "macos")]
pub const ZSH_FLAG: &str = r#"
#!/bin/zsh
    
"#;
#[cfg(target_os = "macos")]
pub const SYMLINK_DIRECTORIES_ZSH: &str = r#"
if [[ $UID -eq 0 ]]; then
    ln -s $src $dst
    echo "1"
else
    echo "0"
fi
"#;

/// Extracts a dmg file to from the `$dmp_path` to the `$destination_directory`.
#[cfg(target_os = "macos")]
pub const EXTRACT_DMG_FILE_ZSHELL_EXPRESSION: &str = r#"

echo $dmg_path
echo $destination_directory

# Check if the DMG file exists
if [ ! -f "$dmg_path" ]; then
    echo "Error: DMG file not found at $dmg_path"
    exit 1
fi

# Check if the destination directory exists
if [ ! -d "$destination_directory" ]; then
    echo "Error: Destination directory not found at $destination_directory"
    exit 1
fi

# Get the base name of the DMG file without extension
dmg_basename=$(basename -s .dmg "$dmg_path")

# Mount the DMG file
echo "Mounting DMG file..."
hdiutil attach "$dmg_path" -mountpoint "/Volumes/$dmg_basename"

# Find the .app file within the mounted DMG
app_path=$(find "/Volumes/$dmg_basename" -name "*.app" -maxdepth 1 -print -quit)

# Check if .app file is found
if [ -z "$app_path" ]; then
    echo "Error: No .app file found in the DMG"
    hdiutil detach "/Volumes/$dmg_basename"
    exit 1
fi

# Create the destination .app directory with the desired name
destination_app_directory="$destination_directory/$dmg_basename.app"
mkdir -p "$destination_app_directory"

# Copy the .app file contents to the destination directory
echo "Copying .app contents to $destination_app_directory"
cp -R "$app_path/." "$destination_app_directory/"

# Unmount the DMG file
echo "Unmounting DMG file..."
hdiutil detach "/Volumes/$dmg_basename"

echo "Done!"
"#;
