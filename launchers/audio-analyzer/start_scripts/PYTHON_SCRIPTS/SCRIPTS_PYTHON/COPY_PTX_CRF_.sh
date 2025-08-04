#!/bin/bash

# --- CONFIGURAÇÃO ---
# Adiciona segurança ao script:
#   -e: Encerra o script imediatamente se um comando falhar.
#   -o pipefail: Faz com que um pipeline retorne o status do último comando que falhou, em vez de 0.
set -e -o pipefail

# Constantes
readonly AUDIO_FOLDER_NAME="Audio Files"
# Padrões de exclusão padrão. Adicione ou remova siglas conforme necessário.
exclusion_patterns=("PE" "BA" "RS" "RN" "PB" "POA" "AL" "xx" "XX")

# Variáveis para o resumo final
copied_to_folders=()
skipped_folders=() # Armazena entradas como "NomeDaPasta:Motivo"

# --- FUNÇÕES ---

# Função para imprimir seções do log de forma consistente.
print_header() {
    echo "-------------------------------------------------"
    echo "$1"
    echo "-------------------------------------------------"
}

# Função principal que orquestra todo o processo.
main() {
    # --- ENTRADA DO USUÁRIO E CONFIGURAÇÃO INICIAL ---
    echo -n "ARRASTE A PASTA QUE CONTÉM O TEMPLATE DE SÃO PAULO E TECLE ENTER: "
    read -r WORK_DIR_RAW

    # Remove todas as barras invertidas usadas pelo Finder para escapar de espaços
    # e em seguida remove uma eventual barra final.
    WORK_DIR="${WORK_DIR_RAW//\\/}"
    WORK_DIR="${WORK_DIR%/}"

    if [[ ! -d "$WORK_DIR" ]]; then
        echo "ERRO: O caminho fornecido não é uma pasta válida: '$WORK_DIR'" >&2
        exit 1
    fi

    # --- GERENCIAMENTO INTERATIVO DOS PADRÕES DE EXCLUSÃO ---
    while true; do
        echo ""
        echo "Os padrões de exclusão de praças atuais são: ${exclusion_patterns[*]}"
        echo "O que você deseja fazer?"
        echo "  [1] Adicionar novos padrões"
        echo "  [2] Remover padrões existentes"
        echo "  [Enter] Continuar com os padrões atuais e iniciar o script"
        read -p "Sua escolha: " -r choice

        case "$choice" in
            1)
                read -p "Digite as siglas para ADICIONAR (separadas por espaço): " -r patterns_to_add
                if [[ -n "$patterns_to_add" ]]; then
                    exclusion_patterns+=($(echo "$patterns_to_add" | tr '[:lower:]' '[:upper:]'))
                    echo "Padrões atualizados."
                else
                    echo "Nenhuma sigla adicionada."
                fi
                ;;
            2)
                read -p "Digite as siglas para REMOVER (separadas por espaço): " -r patterns_to_remove
                if [[ -n "$patterns_to_remove" ]]; then
                    local to_remove_array=($(echo "$patterns_to_remove" | tr '[:lower:]' '[:upper:]'))
                    local new_patterns=()
                    for pattern in "${exclusion_patterns[@]}"; do
                        local found=false
                        for to_remove in "${to_remove_array[@]}"; do
                            if [[ "$pattern" == "$to_remove" ]]; then
                                found=true
                                break
                            fi
                        done
                        if [[ "$found" == false ]]; then
                            new_patterns+=("$pattern")
                        fi
                    done
                    exclusion_patterns=("${new_patterns[@]}")
                    echo "Padrões atualizados."
                else
                    echo "Nenhuma sigla removida."
                fi
                ;;
            "")
                echo "Continuando com a lista de exclusão atual..."
                break
                ;;
            *)
                echo "Opção inválida. Por favor, tente novamente."
                ;;
        esac
    done

    print_header "INICIANDO SCRIPT"
    echo "Pasta de trabalho (template): $WORK_DIR"
    echo "Padrões de exclusão em uso: ${exclusion_patterns[*]}"

    local PARENT_DIR
    PARENT_DIR=$(dirname "$WORK_DIR")
    echo "Pastas de destino serão procuradas em: $PARENT_DIR"
    
    # --- IDENTIFICA ITENS PARA COPIAR ---
    local source_ptx_files=()
    while IFS= read -r -d $'\0' ptx_file; do
        source_ptx_files+=("$ptx_file")
    done < <(find "$WORK_DIR" -maxdepth 1 -type f -name "*.ptx" -print0)

    local source_audio_folder="${WORK_DIR}/${AUDIO_FOLDER_NAME}"
    local has_audio_folder=false
    [[ -d "$source_audio_folder" ]] && has_audio_folder=true

    if [[ ${#source_ptx_files[@]} -eq 0 && "$has_audio_folder" = false ]]; then
        print_header "AVISO"
        echo "Nenhum arquivo .ptx ou pasta '${AUDIO_FOLDER_NAME}' encontrados na pasta de trabalho."
        echo "Script concluído sem ações."
        exit 0
    fi

    echo ""
    echo "Itens a serem copiados da pasta de trabalho:"
    if [[ ${#source_ptx_files[@]} -gt 0 ]]; then
        for ptx in "${source_ptx_files[@]}"; do
            printf "  - Arquivo: %s\n" "$(basename "$ptx")"
        done
    fi
    [[ "$has_audio_folder" = true ]] && printf "  - Pasta:   %s\n" "$AUDIO_FOLDER_NAME"

    # --- PROCESSA PASTAS DE DESTINO ---
    print_header "ANALISANDO PASTAS DE DESTINO"
    
    # Popula um array com as pastas de destino
    local destination_folders=()
    while IFS= read -r -d $'\0' dir; do
        destination_folders+=("$dir")
    done < <(find "$PARENT_DIR" -maxdepth 1 -mindepth 1 -type d -print0)

    if [[ ${#destination_folders[@]} -eq 0 ]]; then
        echo "Nenhuma pasta de destino encontrada em '$PARENT_DIR'."
    else
        for dest_path in "${destination_folders[@]}"; do
            process_destination_folder "$dest_path" "$WORK_DIR" "$has_audio_folder" "${source_ptx_files[@]}"
        done
    fi

    # --- RESUMO FINAL ---
    print_summary
}

# Processa cada pasta de destino individualmente.
process_destination_folder() {
    local dest_path="$1"
    local work_dir="$2"
    local has_audio_folder="$3"
    shift 3                     # Remove the three parameters já processados
    local -a source_ptx_files=("$@")  # O restante dos parâmetros são os arquivos .ptx
    
    local dest_basename
    dest_basename=$(basename "$dest_path")

    echo ""
    printf -- "--- Verificando: '%s' ---\n" "$dest_basename"

    # Verificação 1: É a própria pasta de trabalho?
    if [[ "$dest_path" == "$work_dir" ]]; then
        echo "  Ação: Pular (é a pasta de trabalho)."
        skipped_folders+=("${dest_basename}:Pasta de trabalho")
        return
    fi

    # Verificação 2: Corresponde a um padrão de exclusão?
    local first_10_chars_dest_name="${dest_basename:0:10}"
    for pattern in "${exclusion_patterns[@]}"; do
        if [[ "$first_10_chars_dest_name" == *"$pattern"* ]]; then
            local reason="Padrão de exclusão ('$pattern')"
            echo "  Ação: Pular ($reason)."
            skipped_folders+=("${dest_basename}:$reason")
            return
        fi
    done
    
    # Verificação 3: Já existe um arquivo .ptx no destino? Se sim, não faz mais nada.
    local existing_ptx_path
    existing_ptx_path=$(find "$dest_path" -maxdepth 1 -type f -name "*.ptx" -print -quit)
    if [[ -n "$existing_ptx_path" ]]; then
        local reason="Arquivo .ptx já existente: $(basename "$existing_ptx_path")"
        echo "  Ação: Pular. ($reason)"
        skipped_folders+=("${dest_basename}:$reason")
        return
    fi

    # Se chegou aqui, a pasta é um alvo válido para cópia.
    echo "  Destino válido. Tentando copiar os itens..."
    local copied_anything_to_this_dest=false

    # --- CÓPIA DOS ARQUIVOS .PTX ---
    if [[ ${#source_ptx_files[@]} -gt 0 ]]; then
        for ptx_to_copy in "${source_ptx_files[@]}"; do
            # Nomeia o arquivo .ptx de destino utilizando o nome da pasta (<Pasta>.ptx)
            local destination_ptx_path="${dest_path}/${dest_basename}.ptx"
            # O comando `cp -v` (verbose) informa o que está fazendo.
            cp -v "$ptx_to_copy" "$destination_ptx_path"
            copied_anything_to_this_dest=true
            # Há apenas um arquivo .ptx de interesse, portanto podemos sair do loop
            break
        done
    fi

    # --- CÓPIA DA PASTA DE ÁUDIO ---
    if [[ "$has_audio_folder" = true ]]; then
        # `cp -r -v` é verboso e recursivo
        cp -rv "${work_dir}/${AUDIO_FOLDER_NAME}" "$dest_path"
        copied_anything_to_this_dest=true
    fi

    # --- ATUALIZA RESUMO ---
    if [[ "$copied_anything_to_this_dest" = true ]]; then
        copied_to_folders+=("$dest_basename")
    else
        # Este caso só ocorre se a pasta de trabalho não tiver itens para copiar,
        # mas a pasta de destino não foi pulada por outros motivos.
        skipped_folders+=("${dest_basename}:Nenhum item novo precisou ser copiado")
    fi
}

# Imprime o resumo detalhado no final da execução.
print_summary() {
    print_header "RESUMO DETALHADO"

    local copied_count=${#copied_to_folders[@]}
    local skipped_count=${#skipped_folders[@]}

    # PASTAS QUE RECEBERAM ARQUIVOS
    printf "PASTAS QUE RECEBERAM ARQUIVOS (%d):\n" "$copied_count"
    if [[ $copied_count -gt 0 ]]; then
        # Lê, ordena e imprime cada linha
        printf "%s\n" "${copied_to_folders[@]}" | sort | while IFS= read -r folder_name; do
            printf "  - %s\n" "$folder_name"
        done
    else
        echo "  Nenhuma pasta recebeu arquivos novos."
    fi

    echo ""
    # PASTAS QUE FORAM PULADAS
    printf "PASTAS PULADAS OU SEM ALTERAÇÕES (%d):\n" "$skipped_count"
    if [[ $skipped_count -gt 0 ]]; then
        # Lê, ordena e imprime cada linha
        printf "%s\n" "${skipped_folders[@]}" | sort | while IFS= read -r entry; do
            local folder_name="${entry%%:*}"
            local reason="${entry#*:}"
            printf "  - %-40s (Motivo: %s)\n" "$folder_name" "$reason"
        done
    else
        echo "  Nenhuma pasta foi pulada."
    fi

    print_header "FIM DO SCRIPT"
    if [[ $copied_count -eq 0 ]]; then
        echo "Resultado: Nenhuma pasta de destino foi modificada."
    else
        echo "Resultado: Script concluído. $copied_count pasta(s) atualizada(s), $skipped_count pasta(s) pulada(s) ou sem alterações."
    fi
}


# --- PONTO DE ENTRADA DO SCRIPT ---
# Chama a função principal para iniciar a execução.
main
