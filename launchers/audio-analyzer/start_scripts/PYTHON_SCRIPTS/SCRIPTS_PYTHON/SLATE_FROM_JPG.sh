#!/bin/bash

# VERSÃO 13 (PRO) - SINCRONIZAÇÃO DE ÁUDIO CORRIGIDA
# Resolve o erro "Invalid data found" ao garantir que AMBOS os segmentos de
# áudio (slate e principal) são recodificados com os mesmos parâmetros antes da
# concatenação. Isso cria um stream de áudio final limpo e válido.

# --- Funções ---
check_ffmpeg() {
  if ! command -v ffmpeg >/dev/null 2>&1 || ! command -v ffprobe >/dev/null 2>&1; then
    echo "Erro: ffmpeg/ffprobe não está instalado." >&2; exit 1;
  fi
}

# --- Script Principal ---
check_ffmpeg

# --- Limpeza automática de todos os ficheiros temporários ---
TEMP_FILES=()
trap 'echo "\nLimpando ficheiros temporários..."; rm -f "${TEMP_FILES[@]}"' EXIT HUP INT QUIT TERM

# --- Deteção de Ficheiros ---
echo "Por favor, arraste e solte a PASTA contendo o seu vídeo e a imagem do slate, depois pressione Enter:"
read -r FOLDER_PATH
FOLDER_PATH=$(echo "$FOLDER_PATH" | sed "s/'//g")

if [ ! -d "$FOLDER_PATH" ]; then
    echo "Erro: O caminho fornecido não é uma pasta válida." >&2
    exit 1
fi

# Deteção de ficheiros com mensagens de erro claras
VIDEO_FILES=()
while IFS= read -r -d $'\0' file; do VIDEO_FILES+=("$file"); done < <(find "$FOLDER_PATH" -maxdepth 1 -type f -not -name "._*" \( -iname "*.mp4" -o -iname "*.mov" -o -iname "*.mxf" \) -print0)
IMAGE_FILES=()
while IFS= read -r -d $'\0' file; do IMAGE_FILES+=("$file"); done < <(find "$FOLDER_PATH" -maxdepth 1 -type f -not -name "._*" \( -iname "*.jpg" -o -iname "*.jpeg" -o -iname "*.png" \) -print0)

if [ "${#VIDEO_FILES[@]}" -eq 0 ]; then echo "❌ Erro: Nenhum ficheiro de vídeo foi encontrado." >&2; exit 1; fi
if [ "${#IMAGE_FILES[@]}" -eq 0 ]; then echo "❌ Erro: Nenhuma imagem foi encontrada." >&2; exit 1; fi

INPUT_VID="${VIDEO_FILES[0]}"
SLATE_IMG="${IMAGE_FILES[0]}"

echo "\n-> Slate: $(basename "$SLATE_IMG")"
echo "-> Vídeo: $(basename "$INPUT_VID")"

# --- Inspeção de Parâmetros ---
echo "\nInspecionando propriedades para uma correspondência perfeita..."
VIDEO_W=$(ffprobe -v error -select_streams v:0 -show_entries stream=width -of default=noprint_wrappers=1:nokey=1 "$INPUT_VID")
VIDEO_H=$(ffprobe -v error -select_streams v:0 -show_entries stream=height -of default=noprint_wrappers=1:nokey=1 "$INPUT_VID")
VIDEO_FPS=$(ffprobe -v error -select_streams v:0 -show_entries stream=r_frame_rate -of default=noprint_wrappers=1:nokey=1 "$INPUT_VID")
VIDEO_SAR=$(ffprobe -v error -select_streams v:0 -show_entries stream=sample_aspect_ratio -of default=noprint_wrappers=1:nokey=1 "$INPUT_VID")
if [ -z "$VIDEO_SAR" ] || [ "$VIDEO_SAR" = "N/A" ] || [ "$VIDEO_SAR" = "0:1" ]; then VIDEO_SAR="1:1"; fi
SAR_FOR_FILTER=$(echo "$VIDEO_SAR" | sed 's/:/\//g')

AUDIO_CH_LAYOUT=$(ffprobe -v error -select_streams a:0 -show_entries stream=channel_layout -of default=noprint_wrappers=1:nokey=1 "$INPUT_VID")
AUDIO_SAMPLE_RATE=$(ffprobe -v error -select_streams a:0 -show_entries stream=sample_rate -of default=noprint_wrappers=1:nokey=1 "$INPUT_VID")
AUDIO_CODEC=$(ffprobe -v error -select_streams a:0 -show_entries stream=codec_name -of default=noprint_wrappers=1:nokey=1 "$INPUT_VID")
if [ -z "$AUDIO_CODEC" ]; then echo "\n❌ Erro: O vídeo de origem não tem faixa de áudio." >&2; exit 1; fi
SLATE_DURATION=7
AUDIO_TARGET_BITRATE="320k" # Bitrate de alta qualidade para o áudio

# --- Lógica Principal ---

# 1. Gerar o clipe de SLATE, codificando o áudio para o formato de destino
echo "\nPasso 1: Gerando clipe de slate temporário (.ts)..."
TEMP_SLATE_TS="${FOLDER_PATH}/temp_slate_$$.ts"; TEMP_FILES+=("$TEMP_SLATE_TS")
ffmpeg -hide_banner -loglevel error \
  -loop 1 -framerate "$VIDEO_FPS" -i "$SLATE_IMG" \
  -f lavfi -i "anullsrc=channel_layout=$AUDIO_CH_LAYOUT:sample_rate=$AUDIO_SAMPLE_RATE" \
  -t "$SLATE_DURATION" \
  -c:v libx264 -preset medium -crf 23 -pix_fmt yuv420p \
  -vf "scale=$VIDEO_W:$VIDEO_H,setsar=r=$SAR_FOR_FILTER" \
  -c:a aac -b:a "$AUDIO_TARGET_BITRATE" \
  -f mpegts \
  -y "$TEMP_SLATE_TS"
if [ $? -ne 0 ]; then echo "\n❌ Erro: Falha ao gerar o clipe de slate." >&2; exit 1; fi

# 2. Converter o VÍDEO PRINCIPAL, RECODIFICANDO O ÁUDIO para ser idêntico ao do slate
echo "Passo 2: Preparando vídeo principal para concatenação (.ts)..."
TEMP_MAIN_TS="${FOLDER_PATH}/temp_main_$$.ts"; TEMP_FILES+=("$TEMP_MAIN_TS")
ffmpeg -hide_banner -loglevel error \
  -i "$INPUT_VID" \
  -c:v copy -bsf:v h264_mp4toannexb \
  -c:a aac -b:a "$AUDIO_TARGET_BITRATE" \
  -f mpegts \
  -y "$TEMP_MAIN_TS"
if [ $? -ne 0 ]; then echo "\n❌ Erro: Falha ao preparar o vídeo principal." >&2; exit 1; fi

# 3. CONCATENAR os dois ficheiros .ts usando o protocolo robusto
echo "Passo 3: Concatenando os clipes para criar o vídeo final..."
VID_FILENAME=$(basename -- "$INPUT_VID")
VID_EXTENSION="${VID_FILENAME##*.}"
VID_NAME_ONLY="${VID_FILENAME%.*}"
OUTPUT_VID="${FOLDER_PATH}/${VID_NAME_ONLY}_SLATED.${VID_EXTENSION}"

ffmpeg -hide_banner -loglevel error \
  -i "concat:$TEMP_SLATE_TS|$TEMP_MAIN_TS" \
  -c copy \
  -y "$OUTPUT_VID"

if [ $? -ne 0 ]; then
  echo "\n❌ Erro: Falha ao concatenar os clipes de vídeo." >&2
else
  echo "\n✅ Sucesso! O vídeo final foi criado com um stream de áudio limpo e válido."
  echo "   -> Saída: $(basename "$OUTPUT_VID")"
fi
