#!/bin/bash

# A general-purpose script to compare two video files and determine if they are visually identical.
# This script has no logic related to slates or trimming.
# It works by comparing a unique hash of every frame in both videos.
# VERSION 5: Corrected goal - General Purpose Identity Check.

# --- Functions ---
# Function to check for dependencies
check_deps() {
  if ! command -v ffmpeg &> /dev/null || ! command -v ffplay &> /dev/null; then
    echo "Error: ffmpeg or ffplay is not installed or not in your PATH."
    echo "Please install it (e.g., 'brew install ffmpeg') and try again."
    exit 1
  fi
}

# --- Main Script ---
check_deps

# 1. Prompt user and read the folder path
echo "Please drag and drop the folder containing the two videos you want to compare, then press Enter:"
read -r FOLDER_PATH_RAW

# 2. Clean the path
FOLDER_PATH=$(echo "$FOLDER_PATH_RAW" | sed "s/^'//" | sed "s/'$//")

# 3. Validate the folder path
if [ ! -d "$FOLDER_PATH" ]; then
  echo -e "\nError: The path provided is not a valid folder."
  echo "Path received: '$FOLDER_PATH'"
  exit 1
fi
echo -e "\nScanning folder: $FOLDER_PATH"

# 4. Find exactly two video files in the folder
video_files=()
while IFS= read -r -d $'\0' file; do
    video_files+=("$file")
done < <(find "$FOLDER_PATH" -maxdepth 1 -type f \( -iname "*.mov" -o -iname "*.mp4" -o -iname "*.mxf" -o -iname "*.avi" \) -print0)

# 5. Validate the number of files found
if [ "${#video_files[@]}" -ne 2 ]; then
  echo -e "\nError: Expected to find exactly 2 video files, but found ${#video_files[@]}."
  echo "Please ensure the folder contains only the two videos you want to compare."
  exit 1
fi

FILE_A="${video_files[0]}"
FILE_B="${video_files[1]}"

echo -e "\nComparing files:"
echo "  -> File A: $(basename "$FILE_A")"
echo "  -> File B: $(basename "$FILE_B")"

# 6. Perform definitive check using a frame-by-frame hash (framemd5)
echo -e "\nStep 1: Performing definitive frame-by-frame comparison..."
echo "(This may take a moment on long videos)"

# Generate a list of MD5 hashes for every frame in each video.
# The '-map 0:v:0' ensures we only look at the video stream.
HASH_A=$(ffmpeg -hide_banner -v error -i "$FILE_A" -map 0:v:0 -f framemd5 -)
HASH_B=$(ffmpeg -hide_banner -v error -i "$FILE_B" -map 0:v:0 -f framemd5 -)

# Check if the commands succeeded and compare the results.
if [ -z "$HASH_A" ] || [ -z "$HASH_B" ]; then
    echo -e "\n❌ Error: Could not generate frame hashes for one or both videos."
    echo "The files may be corrupt or an unknown FFmpeg error occurred."
elif [ "$HASH_A" = "$HASH_B" ]; then
  echo -e "\n✅ RESULT: The files are visually identical."
else
  echo -e "\n❌ RESULT: The files are DIFFERENT."
fi

# 7. Offer the user a live visual inspection
echo -en "\nStep 2: Do you want to run a live visual inspection to see the differences? (y/n) "
read -r response

if [[ "$response" == "y" || "$response" == "Y" ]]; then
  echo -e "\n\nLaunching ffplay..."
  echo "In the player window, BLACK pixels mean the files are identical at that frame."
  echo "Any COLORED pixels highlight the exact visual difference."
  echo "Press 'q' or close the window to exit."

  ffplay -hide_banner -autoexit \
    -f lavfi "movie='$FILE_A',setpts=PTS-STARTPTS[a]; \
             movie='$FILE_B',setpts=PTS-STARTPTS[b]; \
             [a][b]blend=all_mode=difference"
  
  echo -e "\nVisual inspection complete."
else
  echo -e "\nSkipping visual inspection. Done."
fi