#!/usr/bin/env python3

import os
import glob
import tempfile
import shutil
import uuid
import sys
import subprocess
from pathlib import Path

# Auto-install pyffmpeg if needed
try:
    from pyffmpeg import FFmpeg
except ImportError:
    print("Instalando pyffmpeg (necessário para processamento de áudio)...")
    subprocess.call([sys.executable, "-m", "pip", "install", "pyffmpeg"])
    from pyffmpeg import FFmpeg

def get_folder_path_from_user():
    """Solicita ao utilizador o caminho da pasta a ser processada."""
    while True:
        raw_path = input("Por favor, arraste a pasta a ser processada para esta janela do terminal e pressione Enter: \n-> ")
        
        cleaned_path = raw_path.strip()
        if cleaned_path.startswith("'") and cleaned_path.endswith("'"):
            cleaned_path = cleaned_path[1:-1]
        if cleaned_path.startswith('"') and cleaned_path.endswith('"'):
            cleaned_path = cleaned_path[1:-1]
        cleaned_path = cleaned_path.replace('\\ ', ' ')  # Remove barras invertidas de escape de espaços

        if os.path.isdir(cleaned_path):
            return cleaned_path
        else:
            print(f"ERRO: O caminho '{cleaned_path}' não é um diretório válido. Por favor, tente novamente.")

def get_ffmpeg_path(ff_instance):
    """Obtém o caminho para o binário ffmpeg do pyffmpeg"""
    try:
        # Tentamos vários métodos para obter o caminho
        if hasattr(ff_instance, 'ffmpeg_bin'):
            path = ff_instance.ffmpeg_bin
        elif hasattr(ff_instance, '_FFmpeg__ff_bin'):
            path = ff_instance._FFmpeg__ff_bin
        else:
            # Procura no diretório padrão onde pyffmpeg instala o ffmpeg
            home = str(Path.home())
            default_path = os.path.join(home, '.pyffmpeg', 'bin', 'ffmpeg')
            if os.path.exists(default_path):
                path = default_path
            else:
                return None
        
        print(f"INFO: Usando ffmpeg embutido do pyffmpeg: {path}")
        return path
    except Exception as e:
        print(f"AVISO: Erro ao localizar caminho do ffmpeg: {e}")
        return None

def process_audio_file(ff_instance, input_file, output_file, conversion_options=None):
    """
    Processa um arquivo de áudio usando diretório temporário para evitar problemas com espaços.
    
    Args:
        ff_instance: Instância do FFmpeg
        input_file: Caminho do arquivo de entrada
        output_file: Caminho do arquivo de saída
        conversion_options: Lista de opções adicionais (opcional)
    
    Returns:
        bool: True se o processamento foi bem-sucedido, False caso contrário
    """
    # Cria diretório temporário sem espaços
    temp_dir = tempfile.mkdtemp(prefix="audioprocessing_")
    print(f"Usando diretório temporário: {temp_dir}")
    
    try:
        # Gera nomes temporários sem espaços (mantendo extensões originais)
        input_ext = os.path.splitext(input_file)[1]
        output_ext = os.path.splitext(output_file)[1]
        
        temp_input = os.path.join(temp_dir, f"input_{uuid.uuid4().hex}{input_ext}")
        temp_output = os.path.join(temp_dir, f"output_{uuid.uuid4().hex}{output_ext}")
        
        print(f"Copiando arquivo de entrada para temporário...")
        # Copia o arquivo original para o temporário
        shutil.copy2(input_file, temp_input)
        
        # Constrói as opções para o pyffmpeg
        if conversion_options is None:
            conversion_options = []
            
        options = ['-y', '-i', temp_input]
        options.extend(conversion_options)
        options.append(temp_output)
        
        print(f"Executando pyffmpeg...")
        ff_instance.options(options)
        
        # Verifica se houve erro
        conversion_error = None
        if hasattr(ff_instance, 'error') and ff_instance.error:
            conversion_error = ff_instance.error
            print(f"Erro reportado pelo pyffmpeg: {conversion_error}")
            return False
        
        # Verifica se o arquivo temporário de saída foi criado
        if os.path.exists(temp_output) and os.path.getsize(temp_output) > 0:
            # Copia o resultado de volta para o caminho original
            print(f"Copiando resultado para: {output_file}")
            shutil.copy2(temp_output, output_file)
            
            # Verifica se a cópia de volta foi bem-sucedida
            if os.path.exists(output_file) and os.path.getsize(output_file) > 0:
                return True
            else:
                print(f"ERRO: Falha ao copiar arquivo temporário para destino final")
                return False
        else:
            print(f"ERRO: Arquivo temporário de saída não foi criado corretamente")
            return False
            
    except Exception as e:
        print(f"ERRO durante processamento: {e}")
        return False
    finally:
        # Limpeza: remove arquivos e diretório temporário
        try:
            shutil.rmtree(temp_dir)
            print("Arquivos temporários removidos.")
        except Exception as e:
            print(f"AVISO: Não foi possível remover o diretório temporário: {e}")

def convert_mp3_to_wav(processing_dir, ff_instance):
    """Converte todos os arquivos MP3 em WAV no diretório especificado.
    Retorna uma lista dos caminhos dos arquivos WAV criados.
    """
    print("\n--- Fase 1: Convertendo MP3 para WAV ---")
    mp3_files = glob.glob(os.path.join(processing_dir, "*.mp3"))
    created_wav_files = []  # Lista para guardar os WAVs criados
    
    if not mp3_files:
        print("Nenhum arquivo .mp3 encontrado para converter para .wav.")
        return created_wav_files

    print(f"Encontrados {len(mp3_files)} arquivo(s) .mp3.")
    for input_mp3_path in mp3_files:
        try:
            base_name = os.path.basename(input_mp3_path)
            name_without_ext, _ = os.path.splitext(base_name)
            output_wav_filename = f"{name_without_ext}.wav"  # Ex: musica.mp3 -> musica.wav
            output_wav_path = os.path.join(processing_dir, output_wav_filename)

            print(f"  Convertendo: {base_name} -> {output_wav_filename}")
            
            # Converte usando processo com diretório temporário
            success = process_audio_file(ff_instance, input_mp3_path, output_wav_path)

            if success:
                print(f"  SUCESSO: '{output_wav_filename}' criado.")
                created_wav_files.append(output_wav_path)  # Adiciona à lista de criados
            else:
                print(f"  ERRO: Falha ao converter '{base_name}' para WAV.")
        except Exception as e:
            print(f"  ERRO PYTHON ao converter {base_name} para WAV: {e}")
    return created_wav_files

def convert_wav_to_mp3_320k(processing_dir, ff_instance, wavs_to_keep):
    """Converte arquivos WAV em MP3 (320kbps) no diretório especificado,
    EXCETO os que estão na lista wavs_to_keep.
    """
    print("\n--- Fase 2: Convertendo WAV originais para MP3 (320k) ---")
    all_wav_files = glob.glob(os.path.join(processing_dir, "*.wav"))

    # Filtra os WAVs que não devem ser convertidos (os que foram criados na Fase 1)
    wav_files_to_convert = [f for f in all_wav_files if f not in wavs_to_keep]

    if not wav_files_to_convert:
        print("Nenhum arquivo .wav original encontrado para converter para .mp3 (ou todos os WAVs foram criados na Fase 1).")
        return

    print(f"Encontrados {len(wav_files_to_convert)} arquivo(s) .wav original(is) para converter.")
    for input_wav_path in wav_files_to_convert:
        try:
            base_name = os.path.basename(input_wav_path)
            name_without_ext, _ = os.path.splitext(base_name)
            output_mp3_filename = f"{name_without_ext}.mp3"  # Ex: musica.wav -> musica.mp3
            output_mp3_path = os.path.join(processing_dir, output_mp3_filename)

            print(f"  Convertendo: {base_name} -> {output_mp3_filename}")
            
            # Converte usando processo com diretório temporário, adicionando opções específicas para MP3
            conversion_options = ['-b:a', '320k']  # Bitrate de áudio para 320kbps
            success = process_audio_file(ff_instance, input_wav_path, output_mp3_path, conversion_options)

            if success:
                print(f"  SUCESSO: '{output_mp3_filename}' criado.")
            else:
                print(f"  ERRO: Falha ao converter '{base_name}' para MP3.")
        except Exception as e:
            print(f"  ERRO PYTHON ao converter {base_name} para MP3: {e}")

# --- Script Principal ---
if __name__ == "__main__":
    try:
        # Inicializa pyffmpeg
        ff_instance = FFmpeg()
        print("Instância do pyffmpeg inicializada com sucesso.")
        
        # Verifica o caminho do ffmpeg
        ffmpeg_path = get_ffmpeg_path(ff_instance)
        if ffmpeg_path:
            print(f"FFmpeg encontrado em: {ffmpeg_path}")
        else:
            print("AVISO: Não foi possível determinar o caminho do ffmpeg, mas tentaremos continuar.")
    except Exception as e:
        print(f"Falha ao inicializar o pyffmpeg: {e}")
        print("Tentando reinstalar pyffmpeg...")
        subprocess.call([sys.executable, "-m", "pip", "install", "--force-reinstall", "pyffmpeg"])
        try:
            ff_instance = FFmpeg()
            print("Instância do pyffmpeg reinicializada com sucesso após reinstalação.")
        except Exception as e2:
            print(f"Falha crítica ao inicializar o pyffmpeg mesmo após reinstalação: {e2}")
            print("Por favor, garanta que o pyffmpeg está instalado corretamente.")
            exit(1)

    # Obtém o diretório de processamento
    PROCESSING_DIR = get_folder_path_from_user()
    print(f"\nIniciando processamento de áudio em duas fases no diretório: {PROCESSING_DIR}")

    # Fase 1: MP3 para WAV
    # Guarda a lista de ficheiros WAV que foram criados a partir de MP3
    created_wav_files_paths = convert_mp3_to_wav(PROCESSING_DIR, ff_instance)
    print(f"\n{len(created_wav_files_paths)} arquivo(s) WAV foram criados a partir de MP3s e serão mantidos como WAV.")

    # Fase 2: WAV para MP3 320k
    # Passa a lista de WAVs criados para que não sejam reconvertidos
    convert_wav_to_mp3_320k(PROCESSING_DIR, ff_instance, created_wav_files_paths)

    print("\nProcessamento em duas fases finalizado.") 