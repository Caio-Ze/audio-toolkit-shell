# Audio Normalizer - Release Binaries

High-performance audio normalizer with EBU R128 loudness measurement and format preservation.

## 📦 Available Binaries

This release includes several specialized binaries for different use cases:

- **`audio-normalizer-23`** - LUFS -23 preset (broadcast standard)
- **`audio-normalizer-20`** - LUFS -20 preset (streaming standard)  
- **`audio-normalizer-maximize`** - Peak-only normalization (maximize volume)
- **`audio-normalizer-configurable`** - Fully configurable targets
- **`audio-normalizer-interactive`** - Interactive mode with GUI-like interface
- **`audio-normalizer`** - Main binary with command-line interface

## 🚀 Quick Start

### Simple Usage (Recommended)
```bash
# Normalize to -23 LUFS (broadcast standard)
./audio-normalizer-23 /path/to/your/audio/folder

# Normalize to -20 LUFS (streaming standard)  
./audio-normalizer-20 /path/to/your/audio/folder

# Maximize volume (peak normalization)
./audio-normalizer-maximize /path/to/your/audio/folder
```

### Custom Settings
```bash
# Custom LUFS, peak, and suffix
./audio-normalizer-configurable /path/to/folder -23 -1 "_normalized"

# No suffix (overwrites original files)
./audio-normalizer-configurable /path/to/folder -23 -1 ""
```

## 📖 Detailed Usage Guide

### 1. Preset Binaries (Easiest)

#### `audio-normalizer-23` - Broadcast Standard
```bash
./audio-normalizer-23 <input_file_or_directory>
```
- **Target:** -23 LUFS, -1 dBFS peak
- **Suffix:** `_normalized`
- **Use case:** Broadcast, TV, radio content

**Examples:**
```bash
# Process entire directory
./audio-normalizer-23 /Users/john/Music/Album

# Process single file
./audio-normalizer-23 /Users/john/Music/song.wav
```

#### `audio-normalizer-20` - Streaming Standard
```bash
./audio-normalizer-20 <input_file_or_directory>
```
- **Target:** -20 LUFS, -1 dBFS peak
- **Suffix:** `_normalized`
- **Use case:** Spotify, Apple Music, streaming platforms

#### `audio-normalizer-maximize` - Peak Normalization
```bash
./audio-normalizer-maximize <input_file_or_directory>
```
- **Target:** Peak-only normalization (no LUFS target)
- **Peak:** -1 dBFS
- **Suffix:** `_maximized`
- **Use case:** Maximize volume without loudness standards

### 2. Configurable Binary (Most Flexible)

```bash
./audio-normalizer-configurable <input_file_or_directory> [target_lufs] [target_peak] [output_suffix]
```

**Parameters:**
- `target_lufs`: Target LUFS level (default: -23.0)
- `target_peak`: Target peak level in dBFS (default: -1.0)  
- `output_suffix`: Output filename suffix (default: "_normalized")

**Examples:**
```bash
# Custom settings
./audio-normalizer-configurable /path/to/folder -16 -0.5 "_loud"

# No suffix (overwrites originals) - BE CAREFUL!
./audio-normalizer-configurable /path/to/folder -23 -1 ""

# Quiet normalization
./audio-normalizer-configurable /path/to/folder -30 -3 "_quiet"
```

### 3. Interactive Binary (User-Friendly)

```bash
./audio-normalizer-interactive
```
- Guided interface with menus
- File/directory selection
- Output configuration
- Progress tracking
- Best for beginners

### 4. Main Binary (Advanced)

```bash
./audio-normalizer [OPTIONS] <FOLDER_PATH>
```

**Options:**
- `-l, --target-lufs=<LUFS>` - Target LUFS level
- `-p, --true-peak=<dBFS>` - Maximum true peak level  
- `-s, --output-suffix=<SUFFIX>` - Output filename suffix
- `-v, --verbose` - Enable verbose output
- `-m, --maximize` - Enable maximize mode

**Examples:**
```bash
# Basic usage
./audio-normalizer /path/to/folder

# Custom settings
./audio-normalizer --target-lufs=-23 --true-peak=-1 --output-suffix="_broadcast" /path/to/folder

# Maximize mode
./audio-normalizer --maximize /path/to/folder
```

## 📁 Input/Output Behavior

### Supported Input
- **Single files:** `.wav`, `.flac`, `.mp3`, `.m4a`, `.ogg`
- **Directories:** Automatically discovers all supported audio files
- **Recursive:** Processes subdirectories

### Output Format
- **Always WAV:** All output files are converted to WAV format
- **Format preservation:** Original bit-depth and sample rate maintained
- **No file size inflation:** Efficient format handling

### File Naming
```
Original: song.mp3
Output:   song_normalized.wav  (with default suffix)
Output:   song.wav             (with empty suffix "")
```

## 🎯 LUFS Standards Reference

| Standard | LUFS | Use Case |
|----------|------|----------|
| **-23** | Broadcast | TV, Radio, EBU R128 |
| **-20** | Streaming | Spotify, Apple Music |
| **-16** | Loud | Club, DJ sets |
| **-14** | Very Loud | Mastering reference |

## ⚡ Performance Features

- **Streaming processing:** Constant memory usage regardless of file size
- **Batch processing:** Handles hundreds of files efficiently
- **Error recovery:** Continues processing even if individual files fail
- **Format preservation:** Maintains original audio quality
- **EBU R128 compliant:** Professional-grade loudness measurement

## 📊 Processing Summary

All binaries provide processing summaries:
```
Processing Summary:
  Total files: 16
  Successfully processed: 13
  Skipped: 3
  Failed: 0
```

**Skip reasons:**
- "No gain adjustment needed" - File already at target loudness
- "Unsupported format" - File format not supported
- "File too short" - Audio shorter than minimum duration

## ⚠️ Important Notes

### File Safety
- **With suffix:** Original files are preserved, new files created
- **Without suffix (`""`):** Original files are **OVERWRITTEN** - make backups!

### Performance Tips
- Process entire directories for better efficiency
- Use preset binaries for fastest processing
- Enable verbose mode (`-v`) for detailed progress

### Troubleshooting
- **"Unsupported format":** Check file extension and format
- **"Permission denied":** Ensure write access to output directory
- **"File too short":** Audio must be at least 400ms for accurate measurement

## 🔧 Technical Specifications

- **Loudness standard:** EBU R128 / ITU-R BS.1770-4
- **True peak detection:** ITU-R BS.1770-4 compliant
- **Supported sample rates:** 8kHz to 192kHz
- **Supported bit depths:** 8, 16, 24, 32-bit
- **Channel support:** Mono to 7.1 surround
- **Memory usage:** ~256KB constant (streaming processing)
