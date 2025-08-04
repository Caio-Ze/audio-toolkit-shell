#!/bin/bash

# ==============================================================================
# Audio Converter Script
#
# Description:
# This script converts all .wav and .mp3 files in a user-specified folder
# to 56kbps mono MP3 files. It creates a new subfolder named "converted_56kbps"
# to store the processed files, leaving the original files untouched.
#
# Designed for macOS.
#
# Dependencies:
# - FFmpeg: This script requires FFmpeg to be installed on your system.
#   If you don't have it, you can install it using Homebrew with the command:
#   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
#   brew install ffmpeg
#
# How to Use:
# 1. Save this file as "convert_audio.sh".
# 2. Open Terminal.
# 3. Make the script executable by running: chmod +x convert_audio.sh
# 4. Run the script by typing: ./convert_audio.sh
# 5. When prompted, drag and drop the folder you want to process into the
#    Terminal window and press Enter.
# ==============================================================================

# --- Check for FFmpeg Dependency ---
if ! command -v ffmpeg &> /dev/null
then
    echo "Error: ffmpeg is not installed."
    echo "Please install it to use this script."
    echo "You can install it with Homebrew: brew install ffmpeg"
    exit 1
fi

# --- User Prompt ---
# Ask the user for the directory path.
# The -r option prevents backslash escapes from being interpreted.
# The -p option displays a prompt string.
echo "Please drag the folder you want to process into this window and press Enter:"
read -r FOLDER_PATH

# --- Sanitize Input Path ---
# macOS Terminal automatically escapes spaces with backslashes and sometimes
# wraps the path in single quotes if dragged and dropped.
# This line removes leading/trailing single quotes and double quotes.
# This helps ensure the path is read correctly.
CLEANED_PATH=$(echo "$FOLDER_PATH" | sed "s/^'//;s/'$//;s/^\"//;s/\"$//")

# Check if the provided path is a valid directory
if [ ! -d "$CLEANED_PATH" ]; then
    echo "Error: The path provided is not a valid directory."
    echo "You entered: $CLEANED_PATH"
    exit 1
fi

# --- Create Output Directory ---
# Define the name of the output folder.
OUTPUT_DIR="$CLEANED_PATH/converted_56kbps"

# Create the output directory if it doesn't already exist.
# The -p flag ensures that mkdir creates parent directories if needed
# and doesn't throw an error if the directory already exists.
mkdir -p "$OUTPUT_DIR"
echo "Converted files will be saved in: $OUTPUT_DIR"
echo "--------------------------------------------------"

# --- Processing Loop ---
# Use find to locate all files ending with .mp3 or .wav (case-insensitive).
# The -o flag means "OR". The -print0 and `while read -d ''` loop structure
# is a robust way to handle filenames that contain spaces or special characters.
find "$CLEANED_PATH" -maxdepth 1 -type f \( -iname "*.mp3" -o -iname "*.wav" \) -print0 | while IFS= read -r -d '' file; do

    # Get the base name of the file (e.g., "my_song.mp3")
    filename=$(basename "$file")
    # Get the name of the file without the extension (e.g., "my_song")
    name_no_ext="${filename%.*}"

    # Define the full path for the output file
    output_file="$OUTPUT_DIR/${name_no_ext}_56kbps.mp3"

    echo "Processing: $filename"

    # --- FFmpeg Conversion Command ---
    # -nostdin: Disables interactive mode, preventing confusing prompts.
    # -i "$file": Specifies the input file.
    # -y: Overwrite output file if it exists without asking.
    # -b:a 56k: Sets the audio bitrate to 56 kilobits per second.
    # -ac 1: Sets the number of audio channels to 1 (mono).
    # -ar 44100: Sets the audio sample rate to 44100 Hz. Common for music.
    # -loglevel error: Only shows fatal errors, keeping the output clean.
    ffmpeg -nostdin -i "$file" -y -b:a 56k -ac 1 -ar 44100 "$output_file" -loglevel error

    # Check the exit status of the ffmpeg command
    if [ $? -eq 0 ]; then
        echo "  -> Success: Converted to $output_file"
    else
        echo "  -> Error: Failed to convert $filename"
    fi
done

echo "--------------------------------------------------"
echo "Conversion process finished!"
