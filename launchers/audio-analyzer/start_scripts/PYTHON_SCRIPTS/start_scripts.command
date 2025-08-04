#!/bin/zsh

# Tentativa de redimensionar janela (30 linhas x 45 colunas)
# Suportado no Terminal.app e iTerm2 (xterm CSI 8 ; rows ; cols t)
# Ghostty e outros emuladores podem ignorar. Detectamos via $TERM_PROGRAM.
if [[ "$TERM_PROGRAM" == "Apple_Terminal" || "$TERM_PROGRAM" == "iTerm.app" ]]; then
    printf '\033[8;30;45t'
    sleep 0.1
else
    # Ajusta somente o tamanho lógico para apps (não altera o tamanho da janela)
    stty rows 30 cols 45 2>/dev/null || true
fi

# Get the directory where this script is located
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
VENV_ACTIVATE="$SCRIPT_DIR/venv/bin/activate"
SCRIPTS_DIR="$SCRIPT_DIR/SCRIPTS_PYTHON"

# Check if the venv activate script exists
if [ ! -f "$VENV_ACTIVATE" ]; then
    echo "Erro: Script de ativação do ambiente virtual não encontrado em $VENV_ACTIVATE"
    echo "Certifique-se de que o ambiente virtual 'venv' existe e foi criado corretamente."
    exit 1
fi

# --- ANSI Color Codes (Optional but nice) ---
C_BLUE=$'\e[94m'
C_GREEN=$'\e[92m'
C_YELLOW=$'\e[93m'
C_RED=$'\e[91m'
C_BOLD=$'\e[1m'
C_ENDC=$'\e[0m'
C_CYAN=$'\e[96m'

# Activate the virtual environment
source "$VENV_ACTIVATE"
echo "${C_GREEN}${C_BOLD}✔ Ambiente virtual 'venv' ativado.${C_ENDC}"
echo # blank line

# Check if scripts directory exists
if [ ! -d "$SCRIPTS_DIR" ]; then
    echo "Erro: Diretório de scripts não encontrado em $SCRIPTS_DIR"
    exit 1
fi

# Change to the scripts directory for easier execution and listing
cd "$SCRIPTS_DIR" || exit 1 # Exit if cd fails

echo "${C_CYAN}${C_BOLD}Diretório de scripts:${C_ENDC} ${C_CYAN}$(pwd)${C_ENDC}"
echo # blank line

# Encontrar todos os arquivos não ocultos no nível atual
all_files=($(find . -maxdepth 1 -type f -not -name '.*' -print | sed 's|\./||'))

# Arquivos que NÃO devem aparecer para o usuário (dependências internas)
ignore=(
  "ffmpeg"          # binário interno
  "yt-dlp"          # alias linux
  "yt-dlp_macos"    # binário macOS universal
  "AudioConverter"  # dependência auxiliar
  "-23-to-0-plus_NET" # processador de loudness interno
  "cookies.txt"     # arquivos de suporte
)

# Separar por tipo para melhor visualização
py_items=()
sh_items=()
rust_items=()
other_items=()

for f in "${all_files[@]}"; do
    # ignorar se estiver na lista ignore
    for ign in "${ignore[@]}"; do
        if [[ "$f" == "$ign" ]]; then
            continue 2
        fi
    done

    if [[ "$f" == *.py ]]; then
        py_items+=("$f")
    elif [[ "$f" == *.sh ]]; then
        sh_items+=("$f")
    elif [[ "$f" == *_rust ]]; then
        rust_items+=("$f")
    else
        other_items+=("$f")
    fi
done

# Construir lista final (ordem de exibição)
items=("${py_items[@]}" "${sh_items[@]}" "${rust_items[@]}" "${other_items[@]}" "Sair")

if [ ${#items[@]} -eq 1 ]; then
    echo "Nenhum script encontrado em $SCRIPTS_DIR"
    cd "$SCRIPT_DIR" # Go back to original dir before exiting
    deactivate # Deactivate venv if possible (may not be needed depending on shell exit behavior)
    exit 0
fi

echo # Newline

# Prompt message for read
PROMPT="${C_YELLOW}Escolha o número do script para rodar: ${C_ENDC}"

while true; do
    echo "${C_BLUE}${C_BOLD}========================================${C_ENDC}"
    echo "${C_BLUE}${C_BOLD}           MENU DE SCRIPTS             ${C_ENDC}"
    echo "${C_BLUE}${C_BOLD}========================================${C_ENDC}"

    local idx=1
    if [ ${#py_items[@]} -gt 0 ]; then
        echo "${C_GREEN}Python (.py):${C_ENDC}"
        for item_name in "${py_items[@]}"; do
            printf "  %s%d%s: %s\n" "$C_YELLOW" "$idx" "$C_ENDC" "$item_name"
            ((idx++))
        done
    fi

    if [ ${#sh_items[@]} -gt 0 ]; then
        echo "${C_GREEN}Shell (.sh):${C_ENDC}"
        for item_name in "${sh_items[@]}"; do
            printf "  %s%d%s: %s\n" "$C_YELLOW" "$idx" "$C_ENDC" "$item_name"
            ((idx++))
        done
    fi

    if [ ${#rust_items[@]} -gt 0 ]; then
        echo "${C_GREEN}Executáveis Rust:${C_ENDC}"
        for item_name in "${rust_items[@]}"; do
            printf "  %s%d%s: %s\n" "$C_YELLOW" "$idx" "$C_ENDC" "$item_name"
            ((idx++))
        done
    fi

    if [ ${#other_items[@]} -gt 0 ]; then
        echo "${C_GREEN}Dependências:${C_ENDC}"
        for item_name in "${other_items[@]}"; do
            printf "  %s%d%s: %s\n" "$C_YELLOW" "$idx" "$C_ENDC" "$item_name"
            ((idx++))
        done
    fi

    # Imprimir opção Sair
    printf "  %s%d%s: %s%s%s\n" "$C_RED" "$idx" "$C_ENDC" "$C_RED" "Sair" "$C_ENDC"

    echo "----------------------------------------"

    # Read user input
    local choice
    # Print prompt without newline, then read
    printf "%s" "$PROMPT"
    read -r choice

    # Validate input
    # Check if it's a number and within the range 1 to number of items
    if [[ "$choice" =~ ^[0-9]+$ ]] && [ "$choice" -ge 1 ] && [ "$choice" -le ${#items[@]} ]; then
        # Get the selected item name
        local opt=${items[$choice]}

        # Handle Exit choice
        if [[ "$opt" == "Sair" ]]; then
            echo "Saindo..."
            cd "$SCRIPT_DIR" # Go back to original dir
            # Attempt to deactivate, might print a message if already inactive or function not found
            type deactivate >/dev/null 2>&1 && deactivate 
            exit 0
        fi

        # Execute the chosen script
        echo "\n${C_GREEN}--- Executando: $opt ---${C_ENDC}"
        
        # Determine how to run based on extension or name
        if [[ "$opt" == *".py" ]]; then
            python "./$opt"
        elif [[ "$opt" == *.sh ]]; then
            # Execute shell scripts explicitly with bash, regardless of execute bit
            bash "./$opt"
        elif [[ "$opt" == *" alias" ]] && [[ "$(uname)" == "Darwin" ]]; then
            # Use open for aliases on macOS
            open "./$opt"
        elif [[ -x "./$opt" ]]; then 
             # If it's executable, run it directly
            "./$opt"
        else
             # Otherwise, attempt to run with python as a fallback? Or just report error?
             # Let's report an error for clarity if not .py, alias, or executable
             echo "${C_RED}Erro: Não sei como executar '$opt'. Não é .py, ' alias' ou executável.${C_ENDC}"
        fi
        
        # Capture exit code
        exit_code=$?
        echo "----------------------------------------"
        if [ $exit_code -eq 0 ]; then
            echo "${C_GREEN}--- '$opt' finalizado com sucesso ---${C_ENDC}"
        else
            echo "${C_RED}--- '$opt' finalizado com erro (código: $exit_code) ---${C_ENDC}"
        fi
        echo # Add a newline for separation
        
    else
        # Invalid input
        echo "\n${C_RED}Seleção inválida '$choice'. Por favor, digite um número entre 1 e ${#items[@]}.${C_ENDC}\n"
        sleep 1 # Optional pause for user to read the error
    fi # End of input validation

    echo # Add a newline before showing the menu again
    # Optional pause
    # sleep 1 
done

# Go back to the original directory (might be useful if script is sourced)
cd "$SCRIPT_DIR"
# Attempt to deactivate
type deactivate >/dev/null 2>&1 && deactivate 