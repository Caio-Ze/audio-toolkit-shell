#!/bin/sh

# VERSÃO 6.1: Solução definitiva usando um re-mux para Transport Stream (TS).
# Adiciona o sufixo "_UNSLATED" ao ficheiro de saída para maior clareza.

SLATE_DURATION=7

# --- Funções ---
check_ffmpeg() {
  if ! command -v ffmpeg >/dev/null 2>&1; then
    echo "Erro: ffmpeg não está instalado." >&2; exit 1;
  fi
}

# --- Script Principal ---
check_ffmpeg

echo "Por favor, arraste e solte a PASTA contendo os vídeos que você deseja processar, depois pressione Enter:"
read -r FOLDER_PATH
# Limpa aspas que podem vir do drag-and-drop
FOLDER_PATH=$(echo "$FOLDER_PATH" | sed "s/'//g")

if [ ! -d "$FOLDER_PATH" ]; then
    echo "Erro: O caminho fornecido não é uma pasta válida." >&2
    exit 1
fi

echo "\nAnalisando pasta: $FOLDER_PATH"
FOUND_FILES=0

# O agrupamento { ... } garante que a variável FOUND_FILES funcione corretamente
find "$FOLDER_PATH" -maxdepth 1 -type f \
  \( -iname "*.mov" -o -iname "*.mp4" \) \
  -name "*_SLATED.*" \
  -print0 | {
  while IFS= read -r -d $'\0' INPUT_VID; do
    FOUND_FILES=$((FOUND_FILES + 1))
    
    echo "\n----------------------------------------"
    echo "Vídeo encontrado: $(basename "$INPUT_VID")"

    VID_FILENAME=$(basename -- "$INPUT_VID")
    # Substitui "_SLATED" por "_UNSLATED" para criar o novo nome de ficheiro
    OUTPUT_FILENAME=$(echo "$VID_FILENAME" | sed 's/_SLATED/_UNSLATED/')
    OUTPUT_VID="$(dirname "$INPUT_VID")/${OUTPUT_FILENAME}"
    TEMP_TS_FILE="$(dirname "$INPUT_VID")/temp_cut_$$.ts" # Ficheiro temporário único
    
    echo "  -> Saída será: $(basename "$OUTPUT_VID")"
    echo "  -> Removendo os primeiros $SLATE_DURATION segundos..."

    # --- COMANDO CORRIGIDO (V6) ---
    # Passo 1: Cortar de forma precisa para um formato intermediário (MPEG-TS).
    # O formato .ts não tem problemas com timestamps que não começam em zero,
    # permitindo um corte limpo com -ss depois de -i.
    ffmpeg -hide_banner -loglevel error \
      -i "$INPUT_VID" \
      -ss "$SLATE_DURATION" \
      -c copy \
      -map 0 \
      -f mpegts \
      -y "$TEMP_TS_FILE"

    if [ $? -ne 0 ]; then
      echo "  -> ❌ Erro: FFmpeg falhou na etapa de corte inicial."
      rm -f "$TEMP_TS_FILE"
      continue
    fi

    # Passo 2: Re-muxar o ficheiro .ts de volta para o container original (MP4/MOV).
    # Este processo reinicia os timestamps do vídeo para zero de forma natural e limpa.
    ffmpeg -hide_banner -loglevel error \
      -i "$TEMP_TS_FILE" \
      -c copy \
      -map 0 \
      -y "$OUTPUT_VID"

    if [ $? -eq 0 ]; then
      echo "  -> ✅ Sucesso! Corte preciso realizado e ficheiro finalizado."
    else
      echo "  -> ❌ Erro: FFmpeg falhou na etapa de finalização."
    fi
    
    # Limpar o ficheiro temporário
    rm -f "$TEMP_TS_FILE"
  done

  echo "\n----------------------------------------"
  if [ $FOUND_FILES -eq 0 ]; then
    echo "Nenhum arquivo de vídeo '*_SLATED.*' foi encontrado para processar na pasta."
  else
    echo "Processo concluído. Foram processados $FOUND_FILES arquivo(s)."
  fi
}
