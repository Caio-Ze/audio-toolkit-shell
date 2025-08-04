#!/bin/sh

# Interactive script to compare the AUDIO ONLY of two .mp4 files in a folder.
# VERSION 5: Uses the more reliable 'astats' filter for analysis and a robust
# ffmpeg-to-ffplay pipe for the auditory test.

# --- Functions ---
check_ffmpeg() {
  if ! command -v ffmpeg >/dev/null 2>&1 || ! command -v ffplay >/dev/null 2>&1; then
    echo "Error: ffmpeg/ffplay is not installed or not in your PATH."
    echo "Please install it (e.g., 'brew install ffmpeg') and try again."
    exit 1
  fi
}

# --- Main Script ---
check_ffmpeg

# 1. Prompt user
echo "Please drag and drop the folder containing the two videos you want to compare, then press Enter:"
read -r FOLDER_PATH_RAW

# 2. Validate the path
if [ ! -d "$FOLDER_PATH_RAW" ]; then
  echo "\nError: The path provided is not a valid folder."
  echo "Path received: '$FOLDER_PATH_RAW'"
  exit 1
fi
echo "\nScanning folder: $FOLDER_PATH_RAW"

# 3. Robustly find exactly two .mp4 files
FILE_LIST=$(find "$FOLDER_PATH_RAW" -maxdepth 1 -type f -iname "*.mp4")
NUM_FILES=$(echo "$FILE_LIST" | wc -l | tr -d ' ')

if [ "$NUM_FILES" -ne 2 ]; then
  echo "\n❌ Error: Found $NUM_FILES MP4 file(s) in this folder."
  echo "This script requires exactly two .mp4 files to compare."
  exit 1
fi

FILE_A=$(echo "$FILE_LIST" | sed -n '1p')
FILE_B=$(echo "$FILE_LIST" | sed -n '2p')

echo "\nComparing audio from:"
echo "  -> File A: $(basename "$FILE_A")"
echo "  -> File B: $(basename "$FILE_B")"

# 4. Run the FFmpeg quantitative analysis using the 'astats' filter
echo "\nPerforming audio analysis..."
# The 'astats' filter provides simple statistics. We check the 'Peak_level'.
# If the difference is silence, the peak will be -inf.
FFMPEG_OUTPUT=$(ffmpeg -hide_banner \
  -i "$FILE_A" \
  -i "$FILE_B" \
  -filter_complex "[0:a][1:a]amix=inputs=2:duration=first:weights=1 -1,astats=metadata=1" \
  -f null - 2>&1)

# 5. Analyze the output
IS_IDENTICAL=false
PEAK_LEVEL=$(echo "$FFMPEG_OUTPUT" | sed -n 's/.*Peak level: //p' | head -n 1)

if [ "$PEAK_LEVEL" = "-inf" ]; then
  echo "\n✅ RESULT: The audio streams are IDENTICAL."
  IS_IDENTICAL=true
else
  echo "\n❌ RESULT: The audio streams are DIFFERENT."
  echo "   (Peak level of difference: $PEAK_LEVEL dB)"
fi
echo "\nAnalysis complete."

# 6. Offer a live auditory comparison using a robust pipe
echo "\nDo you want to perform a live auditory test? (Listen to the difference) [y/n]"
printf "Your choice: "
read -r choice

case "$choice" in
  y|Y|s|S)
    if [ "$IS_IDENTICAL" = true ]; then
      echo "\nPlaying the difference... You should hear absolute silence."
    else
      echo "\nPlaying the difference... Listen for any residual sound."
    fi
    
    # This robust pipe sends the processed audio from ffmpeg directly to ffplay.
    ffmpeg -hide_banner \
      -i "$FILE_A" -i "$FILE_B" \
      -filter_complex "[0:a][1:a]amix=inputs=2:duration=first:weights=1 -1" \
      -f wav - | ffplay -hide_banner -nodisp -autoexit -
    
    echo "Auditory test finished."
    ;;
  *)
    echo "\nSkipping auditory test."
    ;;
esac