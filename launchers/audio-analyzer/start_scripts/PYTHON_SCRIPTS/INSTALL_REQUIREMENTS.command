#!/bin/bash

# ---------- cores para feedback ----------
C_GREEN=$'\e[92m'; C_RED=$'\e[91m'; C_YELLOW=$'\e[93m'; C_BOLD=$'\e[1m'; C_END=$'\e[0m'
C_CYAN=$'\e[96m'

# Flags de status para resumo ao final
VENV_STATUS="FAIL"; REQ_STATUS="N/A"; SH_STATUS="OK"; BIN_STATUS="OK"

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# Change to the script's directory
cd "$SCRIPT_DIR"

# header progresso
echo "${C_CYAN}${C_BOLD}⌛ Preparing environment…${C_END}"

# ---------------------------------------------------------------------------
# Solicitar senha de administrador uma única vez para usar sudo sem repetir
# ---------------------------------------------------------------------------
SHOW_LOGS=${SHOW_LOGS:-0}
log() { [[ "$SHOW_LOGS" == "1" ]] && echo "$*"; }

echo "${C_YELLOW}🔑 Administrator password will be requested to complete the setup…${C_END}"
if ! sudo -v; then
  echo "${C_RED}Incorrect administrator password. Aborting.${C_END}"
  exit 1
fi

# Executando ajustes de permissões em scripts .sh
echo "Ensuring .sh scripts in SCRIPTS_PYTHON are executable..."

# Try to chmod +x and collect any that still fail
if [ -d "$SCRIPT_DIR/SCRIPTS_PYTHON" ]; then
    # Array to store scripts that remain non-executable
    declare -a failed_perms=()

    # Loop through each .sh file in the subfolder
    while IFS= read -r -d '' sh_file; do
        # Attempt to add execute permission if needed
        if [ ! -x "$sh_file" ]; then
            chmod +x "$sh_file" 2>/dev/null || true
        fi

        # Check again; if still not executable, record it
        if [ ! -x "$sh_file" ]; then
            failed_perms+=("$(basename "$sh_file")")
        fi
    done < <(find "$SCRIPT_DIR/SCRIPTS_PYTHON" -maxdepth 1 -type f -name "*.sh" -print0)

    if [ ${#failed_perms[@]} -eq 0 ]; then
        echo "All .sh scripts are now executable."
        SH_STATUS="OK"
    else
        echo "WARNING: Failed to make the following scripts executable:"
        for fname in "${failed_perms[@]}"; do
            echo "   - $fname"
        done
        echo "You may need to adjust permissions manually (e.g., with sudo)."
        SH_STATUS="FAIL"
    fi
fi

echo "This script will help you activate your Python virtual environment and install requirements."
echo "Please ensure you have a virtual environment created for this project."
echo "--------------------------------------------------------------------------"

# Assumir virtualenv padrão "venv" dentro do projeto
VENV_PATH="$SCRIPT_DIR/venv"

# Construct the path to the activate script
ACTIVATE_SCRIPT="$VENV_PATH/bin/activate"

if [ -f "$ACTIVATE_SCRIPT" ]; then
    echo "Activating virtual environment: $ACTIVATE_SCRIPT"
    source "$ACTIVATE_SCRIPT"
    
    if [ $? -eq 0 ]; then
        echo "Virtual environment activated successfully."
        VENV_STATUS="OK"
        
        # Check for requirements.txt and install if it exists
        if [ -f "requirements.txt" ]; then
            echo "Installing packages from requirements.txt..."
            "$VENV_PATH/bin/pip" install -r requirements.txt
            if [ $? -eq 0 ]; then
                echo "Requirements installed successfully."
                REQ_STATUS="OK"
            else
                echo "ERROR: Failed to install requirements from requirements.txt."
                REQ_STATUS="FAIL"
            fi
        else
            echo "No requirements.txt file found in the current directory. Skipping installation of requirements."
            REQ_STATUS="SKIPPED"
        fi
        
        echo "--------------------------------------------------------------------------"
        echo "Setup complete. Your virtual environment should be active in this terminal session."
        echo "If you opened this by double-clicking, the terminal window might close automatically."
        echo "To keep it open, you might need to run this script from an existing terminal: ./start_env.command"
    else
        echo "ERROR: Failed to activate virtual environment. Please check the path and try again."
    fi
else
    echo "ERROR: The activation script was not found at '$ACTIVATE_SCRIPT'."
    echo "Please ensure the path to the virtual environment directory is correct and it contains a 'bin/activate' script."
fi

# Keep the terminal open for a moment if double-clicked (optional)
# You might need to adjust your terminal preferences for this to work reliably
# or instruct users to run from an existing terminal.
# echo "Press Enter to close this window..."
# read

# ---------------------------------------------------------------------------
# Garantir permissão de execução nos binários Rust/ffmpeg/yt-dlp
# ---------------------------------------------------------------------------
echo "Ensuring execute permission on binaries (Rust, ffmpeg, yt-dlp)…"

declare -a bin_targets=(
    "*_rust"
    "start_scripts_rust_arm"
    "start_scripts_rust_x86"
    "ffmpeg"
    "yt-dlp"  # inclui yt-dlp_macos se houver link simbólico
)

# Arquivos que falharam na limpeza de quarentena (precisam de sudo)
failed_quarantine=()

for pattern in "${bin_targets[@]}"; do
    # Procurar tanto na raiz (SCRIPT_DIR) quanto em SCRIPTS_PYTHON
    for search_dir in "$SCRIPT_DIR" "$SCRIPT_DIR/SCRIPTS_PYTHON"; do
        [ -d "$search_dir" ] || continue
    while IFS= read -r -d '' bin_file; do
            # 1. Tornar executável se necessário
        if [ ! -x "$bin_file" ]; then
            chmod +x "$bin_file" 2>/dev/null || true
        fi
            # 2. Remover atributos de quarentena/proveniência no macOS
        if command -v xattr >/dev/null 2>&1; then
                for attr in com.apple.quarantine com.apple.provenance; do
                    xattr -d "$attr" "$bin_file" 2>/dev/null || {
                        # se falhou, registra
                        [[ $? -eq 1 ]] && failed_quarantine+=("$bin_file")
                    }
                done
        fi
        done < <(find "$search_dir" -maxdepth 1 -type f -name "$pattern" -print0)
done
done

if [ ${#failed_quarantine[@]} -gt 0 ]; then
    echo "\n${C_YELLOW}⚠ The following binaries are still quarantined:${C_END}" 
    for f in "${failed_quarantine[@]}"; do echo "   - $f"; done

    # Automatically attempt sudo removal (password was cached earlier)
    for f in "${failed_quarantine[@]}"; do
        SYS_XATTR="/usr/bin/xattr"; [[ -x "$SYS_XATTR" ]] || SYS_XATTR="$(command -v xattr)"
        printf "→ removing quarantine from %s…\n" "$f"
        if sudo "$SYS_XATTR" -dr com.apple.quarantine "$f" 2>/dev/null || \
           sudo "$SYS_XATTR"   -d com.apple.quarantine "$f" 2>/dev/null || \
           sudo "$SYS_XATTR" -c "$f" 2>/dev/null; then
            echo "   ✔ liberated"
        else
            echo "   ⚠ failed – run manually: sudo /usr/bin/xattr -d com.apple.quarantine '$f'"
            BIN_STATUS="FAIL"
        fi
    done
    [[ "$BIN_STATUS" != "FAIL" ]] && BIN_STATUS="OK"
else
    echo "File permissions adjusted." 
fi

# ---------------------------------------------------------------------------
# Selecionar binário ffmpeg adequado (x86_64 ou arm64) e criar link simbólico
# ---------------------------------------------------------------------------
if [[ -f "$SCRIPT_DIR/SCRIPTS_PYTHON/ffmpeg_arm" || -f "$SCRIPT_DIR/SCRIPTS_PYTHON/ffmpeg_x86" ]]; then
    arch=$(uname -m)
    cd "$SCRIPT_DIR/SCRIPTS_PYTHON" || exit 1
    target=""
    if [[ "$arch" == "arm64" && -f ffmpeg_arm ]]; then
        target="ffmpeg_arm"
    elif [[ -f ffmpeg_x86 ]]; then
        target="ffmpeg_x86"
    fi

    if [[ -n "$target" ]]; then
        ln -sf "$target" ffmpeg
        echo "Link 'ffmpeg' updated → $target (arch: $arch)"
    else
        echo "⚠ No compatible ffmpeg binary found for architecture $arch."
    fi
    cd "$SCRIPT_DIR"
fi

# ---------------------------------------------------------------------------
# Selecionar binário start_scripts_rust adequado e criar link simbólico
# ---------------------------------------------------------------------------
if [[ -f "$SCRIPT_DIR/start_scripts_rust_arm" || -f "$SCRIPT_DIR/start_scripts_rust_x86" ]]; then
    arch=$(uname -m)
    target=""
    if [[ "$arch" == "arm64" && -f "$SCRIPT_DIR/start_scripts_rust_arm" ]]; then
        target="start_scripts_rust_arm"
    elif [[ -f "$SCRIPT_DIR/start_scripts_rust_x86" ]]; then
        target="start_scripts_rust_x86"
    fi

    if [[ -n "$target" ]]; then
        ln -sf "$target" "$SCRIPT_DIR/start_scripts_rust"
        echo "Link 'start_scripts_rust' updated → $target (arch: $arch)"
    else
        echo "⚠ No compatible start_scripts_rust binary found for architecture $arch."
    fi
fi

cd "$SCRIPT_DIR"  # ensure back to root dir

# ---------------- Resumo amigável ----------------
printf "\n${C_BOLD}===== SETUP SUMMARY =====${C_END}\n"
print_stat() {
  local label=$1; local status=$2
  if [[ "$status" == "OK" ]]; then
      printf "  %b✔%b %s\n" "$C_GREEN" "$C_END" "$label"
  elif [[ "$status" == "SKIPPED" ]]; then
      printf "  %b•%b %s (skipped)\n" "$C_YELLOW" "$C_END" "$label"
  else
      printf "  %b✘%b %s\n" "$C_RED" "$C_END" "$label"
  fi
}
print_stat "Virtual environment" "$VENV_STATUS"
print_stat "requirements.txt packages" "$REQ_STATUS"
print_stat "Executable .sh scripts" "$SH_STATUS"
print_stat "Rust / yt-dlp binaries quarantine free" "$BIN_STATUS"
# ffmpeg link info
if [[ -L "$SCRIPT_DIR/SCRIPTS_PYTHON/ffmpeg" ]]; then
   printf "  %b→%b ffmpeg points to $(readlink "$SCRIPT_DIR/SCRIPTS_PYTHON/ffmpeg")\n" "$C_GREEN" "$C_END"
else
   printf "  %b✘%b ffmpeg link not found\n" "$C_RED" "$C_END"
fi
# start_scripts_rust link info
if [[ -L "$SCRIPT_DIR/start_scripts_rust" ]]; then
   printf "  %b→%b start_scripts_rust points to $(readlink "$SCRIPT_DIR/start_scripts_rust")\n" "$C_GREEN" "$C_END"
else
   printf "  %b✘%b start_scripts_rust link not found\n" "$C_RED" "$C_END"
fi
printf "${C_BOLD}===================================${C_END}\n\n" 