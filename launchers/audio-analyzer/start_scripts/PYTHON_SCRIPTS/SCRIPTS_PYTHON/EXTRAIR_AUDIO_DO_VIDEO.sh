#!/bin/sh

# VERSÃO 1.0: Extrai o áudio de ficheiros de vídeo para .wav (sem perdas).
# Procura em todos os ficheiros .mp4, .mov e .mxf dentro da pasta fornecida.

# --- Funções ---
check_ffmpeg() {
  if ! command -v ffmpeg >/dev/null 2>&1; then
    echo "Erro: ffmpeg não está instalado." >&2; exit 1;
  fi
}

# --- Script Principal ---
check_ffmpeg

echo "Por favor, arraste e solte a PASTA contendo os vídeos dos quais deseja extrair o áudio, depois pressione Enter:"
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
  \( -iname "*.mov" -o -iname "*.mp4" -o -iname "*.mxf" \) \
  -print0 | {
  while IFS= read -r -d $'\0' INPUT_VID; do
    # Ignora ficheiros que começam com "._" (ficheiros de metadados do macOS)
    if [[ "$(basename "$INPUT_VID")" == ._* ]]; then
        continue
    fi

    FOUND_FILES=$((FOUND_FILES + 1))
    
    echo "\n----------------------------------------"
    echo "Vídeo encontrado: $(basename "$INPUT_VID")"

    VID_FILENAME=$(basename -- "$INPUT_VID")
    # Remove a extensão original para adicionar a nova
    VID_NAME_ONLY="${VID_FILENAME%.*}"
    OUTPUT_AUDIO="$(dirname "$INPUT_VID")/${VID_NAME_ONLY}.wav"
    
    echo "  -> Extraindo áudio para: $(basename "$OUTPUT_AUDIO")"

    # Comando para extrair o áudio sem perdas para o formato WAV (PCM)
    # -vn: ignora o vídeo
    # -c:a pcm_s16le: define o codec de áudio para PCM de 16-bit, o padrão para .wav.
    # Isto garante a qualidade sem perdas, sem recompressão.
    ffmpeg -hide_banner -loglevel error \
      -i "$INPUT_VID" \
      -vn \
      -c:a pcm_s16le \
      -y "$OUTPUT_AUDIO"

    if [ $? -eq 0 ]; then
      echo "  -> ✅ Sucesso! Áudio extraído."
    else
      echo "  -> ❌ Erro: FFmpeg falhou ao extrair o áudio deste ficheiro."
    fi
  done

  echo "\n----------------------------------------"
  if [ $FOUND_FILES -eq 0 ]; then
    echo "Nenhum ficheiro de vídeo (.mp4, .mov, .mxf) foi encontrado para processar."
  else
    echo "Processo concluído. Foram processados $FOUND_FILES ficheiro(s)."
  fi
}
