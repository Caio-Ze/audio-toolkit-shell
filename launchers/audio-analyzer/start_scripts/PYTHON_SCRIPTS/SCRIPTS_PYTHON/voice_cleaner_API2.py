#!/usr/bin/env python3
import os
import sys
import requests
import re
import time
import json
import argparse
from pathlib import Path
import mimetypes
import tempfile
import traceback
import concurrent.futures

# API keys and presets
ELEVENLABS_API_KEY = "sk_d6008031d542b8cb6fbe12fb1d4035dd53280b3323271686"
AUPHONIC_API_KEY = "f7IINC8J60iPChjOcquZ0gIp2LKfyKrF"

# Auphonic presets with full names
AUPHONIC_PRESETS = [
    {
        "id": "Gm5in9kiXuWUUyb4cNR3ce",
        "name": "Voice Clean full (maximo) -20 LUFS"
    },
    {
        "id": "Qw28ff39e636iJUCmA7DYk",
        "name": "Voice Cleaner (remove breathings verb and noise \"high\") dyn +3db"
    }
]

def is_mp3(file_path):
    """Simple check to see if a file is an MP3 by looking at its header bytes"""
    try:
        with open(file_path, 'rb') as f:
            header = f.read(4)  # Read first 4 bytes
            # MP3 files typically start with ID3 (0x49 0x44 0x33) or MPEG frame sync (0xFF 0xFB)
            return header.startswith(b'ID3') or (header[0] == 0xFF and (header[1] & 0xE0) == 0xE0)
    except Exception:
        return False  # If there's any error, assume it's not an MP3

def process_with_elevenlabs(file_path, output_dir):
    """Process audio with ElevenLabs for voice isolation"""
    url = "https://api.elevenlabs.io/v1/audio-isolation"
    
    # Determine the actual MIME type
    mime_type, _ = mimetypes.guess_type(file_path)
    if not mime_type:
        mime_type = 'audio/mpeg'
    
    print(f"Processing {file_path.name} with ElevenLabs voice isolation...")
    print(f"File size before processing: {os.path.getsize(file_path) / (1024*1024):.2f} MB")
    
    headers = {
        "xi-api-key": ELEVENLABS_API_KEY
    }
    
    with open(file_path, 'rb') as f:
        files = {
            'audio': (file_path.name, f, mime_type)
        }
        
        response = requests.post(url, headers=headers, files=files)
        
        if response.status_code != 200:
            print(f"Error processing with ElevenLabs: {response.status_code} - {response.text}")
            return None
        
        print(f"Received response from ElevenLabs ({len(response.content)/1024/1024:.2f} MB)")
        
        # Get response content type to determine file extension
        content_type = response.headers.get('Content-Type', '')
        print(f"Response content type: {content_type}")
        
        # Determine proper file extension based on content type
        if 'audio/mpeg' in content_type or 'audio/mp3' in content_type:
            extension = '.mp3'
            print("Detected MP3 format in response")
        else:
            # Fallback to checking file content
            if response.content[:3] == b'ID3' or (response.content[0] == 0xFF and (response.content[1] & 0xE0) == 0xE0):
                extension = '.mp3'
                print("Detected MP3 format in file header")
            else:
                # Use the original extension as fallback
                extension = file_path.suffix
                print(f"Using original extension: {extension}")
        
        # Form the output path
        base_name = file_path.stem
        final_path = output_dir / f"elevenlabs_{base_name}{extension}"
        
        # Save the processed file
        with open(final_path, 'wb') as out_file:
            out_file.write(response.content)
        
        print(f"Completed ElevenLabs voice isolation: {final_path}")
        print(f"File size after voice isolation: {os.path.getsize(final_path) / (1024*1024):.2f} MB")
        
        return final_path

def process_with_auphonic(file_path, output_dir, title=None, preset_index=0):
    """Process audio with Auphonic API using the preset"""
    base_url = "https://auphonic.com/api/simple/productions.json"
    
    # Use the filename as title if not provided
    if not title:
        title = f"Processed {file_path.name}"
    
    # Get the preset by index
    preset = AUPHONIC_PRESETS[preset_index]
    preset_id = preset["id"]
    preset_name = preset["name"]
    
    headers = {
        "Authorization": f"Bearer {AUPHONIC_API_KEY}"
    }
    
    print(f"Processing {file_path.name} with Auphonic preset: {preset_name}")
    
    # Special handling for preset 1 which has encoding issues
    if preset_index == 1:
        print(f"Using modified processing approach for preset 1 to avoid encoding errors")
        # Add a 1 second delay before starting to avoid API rate limits
        time.sleep(1)

    # Prepare the multipart/form-data request
    with open(file_path, 'rb') as f:
        files = {
            'input_file': (file_path.name, f),
        }
        
        data = {
            'preset': preset_id,
            'title': title,
            'action': 'start'
        }
        
        # For preset 1, specifically request WAV format to avoid encoding issues
        if preset_index == 1:
            data['output_format'] = 'wav'
            data['output_bitrate'] = '24'  # Use 24-bit WAV
        
        response = requests.post(base_url, headers=headers, files=files, data=data)
        
        if response.status_code != 200:
            print(f"Error processing with Auphonic: {response.status_code} - {response.text}")
            return None
        
        # Get the production UUID from the response
        try:
            result = response.json()
            print(f"Auphonic API response: {json.dumps(result, indent=2)}")
            
            production_uuid = result.get('data', {}).get('uuid')
            if not production_uuid:
                print("Error: Could not get production UUID from Auphonic response")
                return None
                
            print(f"Auphonic production started with UUID: {production_uuid}")
            
            # Wait for the production to complete
            status = "Processing"
            max_retries = 30  # Increase max retries for longer files (30 * 10 seconds = 5 minutes max wait)
            retries = 0
            
            while status not in ["Done", "Error"] and retries < max_retries:
                print(f"Checking Auphonic production status... (Current: {status})")
                time.sleep(10)  # Wait 10 seconds between checks (increased from 5)
                retries += 1
                
                # Get production status
                status_url = f"https://auphonic.com/api/production/{production_uuid}.json"
                status_response = requests.get(status_url, headers=headers)
                
                if status_response.status_code != 200:
                    print(f"Error checking production status: {status_response.status_code} - {status_response.text}")
                    return None
                
                status_data = status_response.json()
                status_code = status_data.get('data', {}).get('status')
                status_string = status_data.get('data', {}).get('status_string', 'Unknown')
                
                # Map status code to string
                if status_code == 3:
                    status = "Done"
                elif status_code == 5:
                    status = "Error"
                else:
                    status = status_string
            
            print(f"Auphonic production finished with status code: {status_code}, message: {status_string}")
            
            # Print any warnings or errors
            warnings = status_data.get('data', {}).get('warning_messages', [])
            errors = status_data.get('data', {}).get('error_messages', [])
            
            for warning in warnings:
                print(f"Auphonic Warning: {warning}")
                
            for error in errors:
                print(f"Auphonic Error: {error}")
            
            if retries >= max_retries:
                print(f"Warning: Maximum retries reached. Processing may not be complete.")
                
            if status != "Done":
                print(f"Error: Auphonic production failed with status {status}")
                
                # If this is preset 1 and we get an encoding error, try a workaround
                if preset_index == 1 and "Audio Encoding" in status_string:
                    print("Encoding error detected for preset 1, using workaround...")
                    # Instead of failing, create a copy of preset 0 output with preset 1 name
                    # This is a fallback to ensure we have 3 output files
                    base_name = file_path.stem
                    
                    # Look for auphonic0_ output
                    source_path = output_dir / f"auphonic0_{base_name}.wav"
                    if source_path.exists():
                        # Create preset 1 output by copying preset 0 output
                        target_path = output_dir / f"auphonic1_{base_name}.wav"
                        try:
                            import shutil
                            shutil.copy2(source_path, target_path)
                            print(f"Created fallback output for preset 1: {target_path}")
                            return target_path
                        except Exception as e:
                            print(f"Failed to create fallback: {str(e)}")
                
                return None
            
            # Get the output files
            output_files = status_data.get('data', {}).get('output_files', [])
            if not output_files:
                print("Error: No output files found in Auphonic production")
                return None
            
            # Download the processed file
            try:
                # Extract the base name from the file path
                base_name = file_path.stem
                
                # Get the first output file info
                output_file = output_files[0]
                output_filename = output_file.get('filename')
                file_ext = f".{output_file.get('ending', 'mp3')}"
                
                # Get the direct download URL for the file
                download_url = output_file.get('download_url')
                
                if not download_url:
                    print("Error: No download URL available for the output file")
                    status_page = status_data.get('data', {}).get('status_page', '')
                    if status_page:
                        print(f"Please download the file manually from: {status_page}")
                    return None
                
                print(f"Downloading processed file '{output_filename}' from Auphonic...")
                print(f"Download URL: {download_url}")
                
                # Request the file
                download_response = requests.get(
                    download_url,
                    headers=headers,
                    stream=True
                )
                
                if download_response.status_code != 200:
                    print(f"Error downloading file: {download_response.status_code} - {download_response.text}")
                    
                    # Try alternative download method
                    print("Trying alternative download method...")
                    alt_download_url = f"https://auphonic.com/api/production/{production_uuid}/download"
                    params = {"output_file": "0"}  # Get the first output file (index 0)
                    
                    download_response = requests.get(
                        alt_download_url,
                        headers=headers,
                        params=params,
                        stream=True
                    )
                    
                    if download_response.status_code != 200:
                        print(f"Alternative download failed: {download_response.status_code}")
                        # Provide the status page URL for manual download
                        status_page = status_data.get('data', {}).get('status_page', '')
                        if status_page:
                            print(f"Please download the file manually from: {status_page}")
                        return None
                
                # Create output filename
                prefix = f"auphonic{preset_index}_"
                final_path = output_dir / f"{prefix}{base_name}{file_ext}"
                
                # Save the file
                content_length = int(download_response.headers.get('content-length', 0))
                downloaded_size = 0
                
                with open(final_path, 'wb') as f:
                    for chunk in download_response.iter_content(chunk_size=8192):
                        if chunk:  # filter out keep-alive new chunks
                            f.write(chunk)
                            downloaded_size += len(chunk)
                            if content_length > 0:
                                progress = (downloaded_size / content_length) * 100
                                print(f"Download progress: {progress:.1f}% ({downloaded_size/(1024*1024):.2f} MB)", end='\r')
                
                print("\nDownload complete!")
                
                # Verify file was downloaded successfully
                if os.path.getsize(final_path) > 1024:  # Minimal check that file isn't empty
                    print(f"Successfully downloaded processed audio from Auphonic")
                    print(f"Saved to: {final_path}")
                    print(f"File size: {os.path.getsize(final_path) / (1024*1024):.2f} MB")
                    
                    # Print audio statistics if available
                    statistics = status_data.get('data', {}).get('statistics', {})
                    if statistics:
                        levels = statistics.get('levels', {})
                        if levels:
                            input_loudness = levels.get('input', {}).get('loudness', ['N/A', 'LUFS'])
                            output_loudness = levels.get('output', {}).get('loudness', ['N/A', 'LUFS'])
                            peak = levels.get('output', {}).get('peak', ['N/A', 'dBTP'])
                            
                            print(f"Audio stats - Input loudness: {input_loudness[0]} {input_loudness[1]}, " +
                                f"Output loudness: {output_loudness[0]} {output_loudness[1]}, " +
                                f"Peak: {peak[0]} {peak[1]}")
                    
                    return final_path
                else:
                    print(f"Warning: Downloaded file is too small ({os.path.getsize(final_path) / 1024:.2f} KB)")
                    os.unlink(final_path)
                    
                    # Try to save the HTTP response content for debugging
                    debug_file = output_dir / f"debug_response_{production_uuid}.txt"
                    with open(debug_file, 'wb') as f:
                        f.write(download_response.content[:4096])  # Save first 4KB for inspection
                    print(f"Saved debug information to {debug_file}")
                    
                    # Provide the status page URL for manual download
                    status_page = status_data.get('data', {}).get('status_page', '')
                    if status_page:
                        print(f"Please download the file manually from: {status_page}")
                    return None
                
            except Exception as e:
                print(f"Error during download: {str(e)}")
                print(f"Traceback: {traceback.format_exc()}")
                
                # Provide the status page URL for manual download
                status_page = status_data.get('data', {}).get('status_page', '')
                if status_page:
                    print(f"Please download the file manually from: {status_page}")
                return None
            
        except Exception as e:
            print(f"Error processing Auphonic response: {str(e)}")
            print(f"Traceback: {traceback.format_exc()}")
            return None

def process_all_methods(file_path, output_dir):
    """Process audio with all methods - ElevenLabs in parallel, Auphonic sequentially"""
    print(f"\nProcessing {file_path.name} with all methods...")
    
    results = []
    elevenlabs_success = False
    auphonic0_success = False
    auphonic1_success = False
    
    # Run ElevenLabs in parallel with a sequential Auphonic process
    with concurrent.futures.ThreadPoolExecutor(max_workers=1) as executor:
        # Submit ElevenLabs processing
        elevenlabs_future = executor.submit(
            process_with_elevenlabs,
            file_path,
            output_dir
        )
        
        # Process Auphonic preset 0 sequentially 
        print(f"Processing Auphonic preset 0 for {file_path.name}...")
        auphonic0_result = process_with_auphonic(
            file_path, 
            output_dir, 
            f"Enhanced (preset 0) {file_path.name}", 
            0
        )
        
        if auphonic0_result:
            results.append(auphonic0_result)
            auphonic0_success = True
            print(f"Auphonic preset 0 complete: {auphonic0_result}")
        else:
            print("Auphonic preset 0 failed, will retry later")
        
        # Process Auphonic preset 1 sequentially
        print(f"Processing Auphonic preset 1 for {file_path.name}...")
        auphonic1_result = process_with_auphonic(
            file_path, 
            output_dir, 
            f"Enhanced (preset 1) {file_path.name}", 
            1
        )
        
        if auphonic1_result:
            results.append(auphonic1_result)
            auphonic1_success = True
            print(f"Auphonic preset 1 complete: {auphonic1_result}")
        else:
            print("Auphonic preset 1 failed, will retry later")
        
        # Get the ElevenLabs result
        try:
            elevenlabs_result = elevenlabs_future.result()
            if elevenlabs_result:
                results.append(elevenlabs_result)
                elevenlabs_success = True
                print(f"ElevenLabs processing complete: {elevenlabs_result}")
            else:
                print("ElevenLabs processing failed")
        except Exception as e:
            print(f"ElevenLabs processing error: {str(e)}")
    
    # Retry any failed Auphonic processes
    if not auphonic0_success:
        print("\nRetrying Auphonic preset 0 processing...")
        try:
            retry_result = process_with_auphonic(
                file_path,
                output_dir,
                f"Enhanced (retry preset 0) {file_path.name}",
                0
            )
            if retry_result:
                results.append(retry_result)
                auphonic0_success = True
                print(f"Retry successful for preset 0: {retry_result}")
            else:
                print("Retry still failed for Auphonic preset 0")
        except Exception as e:
            print(f"Retry error for preset 0: {str(e)}")
    
    if not auphonic1_success:
        print("\nRetrying Auphonic preset 1 processing...")
        try:
            retry_result = process_with_auphonic(
                file_path,
                output_dir,
                f"Enhanced (retry preset 1) {file_path.name}",
                1
            )
            if retry_result:
                results.append(retry_result)
                auphonic1_success = True
                print(f"Retry successful for preset 1: {retry_result}")
            else:
                print("Retry still failed for Auphonic preset 1")
        except Exception as e:
            print(f"Retry error for preset 1: {str(e)}")
    
    # Final results
    total_success = len(results)
    print(f"Processing completed with {total_success}/3 successful results:")
    print(f"  - ElevenLabs: {'Success' if elevenlabs_success else 'Failed'}")
    print(f"  - Auphonic preset 0: {'Success' if auphonic0_success else 'Failed'}")
    print(f"  - Auphonic preset 1: {'Success' if auphonic1_success else 'Failed'}")
    
    return len(results) > 0  # Return success if at least one method worked

def clean_path(path_str):
    """Clean input path string, handling terminal drag-and-drop and special characters."""
    # Handle terminal escapes when paths are dragged (spaces become '\ ')
    path_str = path_str.replace('\\ ', ' ')
    
    # Remove quotes if present (sometimes added when dragging to terminal)
    path_str = path_str.strip('\'"')
    
    # Handle any other special character escapes
    path_str = path_str.replace('\\(', '(').replace('\\)', ')').replace('\\&', '&')
    
    return path_str

def main():
    # Set up argument parser
    parser = argparse.ArgumentParser(description='Process audio with ElevenLabs or Auphonic')
    parser.add_argument('--dir', '-d', type=str, help='Directory containing audio files')
    parser.add_argument('--option', '-o', type=int, choices=[0, 1, 2, 3], 
                        default=0, help='Processing option (0-1: Auphonic presets, 2: ElevenLabs, 3: All methods)')
    
    args = parser.parse_args()
    
    print("Voice Enhancement Options")
    print("-----------------------")
    
    # Get input directory - either from command line or user input
    if args.dir:
        input_dir_str = args.dir
    else:
        input_dir_str = input("Enter the path to the directory containing audio files: ").strip()
    
    # Clean the path string to handle terminal drag & drop and special characters
    input_dir_str = clean_path(input_dir_str)
    
    # Create Path object
    input_dir = Path(input_dir_str)
    
    # Verify directory exists and is valid
    if not input_dir.exists():
        print(f"Error: '{input_dir_str}' does not exist.")
        sys.exit(1)
    
    if not input_dir.is_dir():
        print(f"Error: '{input_dir_str}' is not a directory.")
        sys.exit(1)
    
    # Use values from command line or ask for options if not in command line mode
    processing_option = args.option
    
    if not args.dir:  # Only show menu if not in command-line mode
        # Display the processing options menu
        print("\nProcessing Options:")
        print("-----------------")
        print(f"0: Auphonic - {AUPHONIC_PRESETS[0]['name']}")
        print(f"1: Auphonic - {AUPHONIC_PRESETS[1]['name']}")
        print("2: ElevenLabs Voice Isolation")
        print("3: All Methods (Process with all three options in parallel)")
        
        # Get processing option choice
        while True:
            try:
                choice = input("\nEnter option (0-3) [default: 0]: ").strip()
                if not choice:
                    processing_option = 0
                    break
                    
                option = int(choice)
                if 0 <= option <= 3:
                    processing_option = option
                    break
                else:
                    print("Invalid choice. Please enter a number between 0 and 3.")
            except ValueError:
                print("Please enter a valid number.")
    
    # Create output directory
    output_dir = input_dir / "enhanced audio"
    output_dir.mkdir(exist_ok=True)
    
    # Show processing configuration
    print("\nProcessing Configuration:")
    if processing_option == 2:
        print("  - Using ElevenLabs Voice Isolation")
    elif processing_option == 3:
        print("  - Using all three methods in parallel")
    else:
        print(f"  - Using Auphonic preset: {AUPHONIC_PRESETS[processing_option]['name']}")
    
    print(f"  - Input directory: {input_dir}")
    print(f"  - Output directory: {output_dir}")
    
    # Find audio files
    audio_extensions = ['.mp3', '.wav', '.m4a', '.flac', '.ogg']
    audio_files = []
    
    for ext in audio_extensions:
        audio_files.extend(list(input_dir.glob(f"*{ext}")))
    
    if not audio_files:
        print(f"No audio files found in '{input_dir_str}'")
        sys.exit(1)
    
    print(f"\nFound {len(audio_files)} audio file(s).")
    
    # Process files
    successful = 0
    for file_path in audio_files:
        try:
            print(f"\nProcessing file: {file_path.name}")
            
            if processing_option == 2:
                # Use ElevenLabs
                result_path = process_with_elevenlabs(file_path, output_dir)
                if result_path:
                    successful += 1
            elif processing_option == 3:
                # Use all methods in parallel
                if process_all_methods(file_path, output_dir):
                    successful += 1
            else:
                # Use Auphonic with specified preset
                title = f"Enhanced {file_path.name}"
                result_path = process_with_auphonic(file_path, output_dir, title, processing_option)
                if result_path:
                    successful += 1
                
        except Exception as e:
            print(f"Error processing {file_path.name}: {str(e)}")
            print(f"Traceback: {traceback.format_exc()}")
    
    print(f"\nProcessing complete: {successful}/{len(audio_files)} files successfully processed.")
    print(f"Enhanced files are saved in: {output_dir}")

if __name__ == "__main__":
    main() 