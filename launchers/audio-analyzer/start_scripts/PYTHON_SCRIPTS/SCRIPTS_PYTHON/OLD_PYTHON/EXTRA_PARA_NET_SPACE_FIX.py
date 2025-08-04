#!/usr/bin/env python3

import os
import glob
import shlex
from pyffmpeg import FFmpeg
import sys
import subprocess
import tempfile
import shutil
import uuid

# Código para PyInstaller encontrar o ffmpeg empacotado
if getattr(sys, 'frozen', False) and hasattr(sys, '_MEIPASS'):
    # Estamos rodando em um pacote PyInstaller
    ffmpeg_exe_name = 'ffmpeg.exe' if os.name == 'nt' else 'ffmpeg'
    bundled_ffmpeg_path = os.path.join(sys._MEIPASS, ffmpeg_exe_name)
    if os.path.exists(bundled_ffmpeg_path):
        print(f"INFO: Executável PyInstaller - Configurando FFMPEG_BIN para: {bundled_ffmpeg_path}")
        os.environ['FFMPEG_BIN'] = bundled_ffmpeg_path
    else:
        print(f"AVISO: Executável PyInstaller - ffmpeg esperado em {bundled_ffmpeg_path} não encontrado!")

# --- Configuração ---
OUTPUT_SUFFIX = "_NET"
TARGET_EXTENSION = ".wav"
# ---------------------

def get_folder_path_from_user():
    """Solicita ao utilizador o caminho da pasta a ser processada."""
    while True:
        raw_path = input("Por favor, arraste a pasta a ser processada para esta janela do terminal e pressione Enter: \n-> ")
        
        cleaned_path = raw_path.strip()
        if cleaned_path.startswith("'") and cleaned_path.endswith("'"):
            cleaned_path = cleaned_path[1:-1]
        if cleaned_path.startswith('"') and cleaned_path.endswith('"'):
            cleaned_path = cleaned_path[1:-1]
        
        cleaned_path = cleaned_path.replace('\\ ', ' ')

        if os.path.isdir(cleaned_path):
            return cleaned_path
        else:
            print(f"ERRO: O caminho '{cleaned_path}' não é um diretório válido. Por favor, tente novamente.")

# Cria uma instância do FFmpeg
def init_pyffmpeg():
    try:
        ff_instance = FFmpeg()
        print("Instância do pyffmpeg inicializada com sucesso.")
        return ff_instance
    except Exception as e:
        print(f"Falha ao inicializar o pyffmpeg: {e}")
        print("Por favor, garanta que o pyffmpeg está instalado corretamente.")
        exit(1)

def process_audio_file(ff_instance, input_file_path, output_file_path):
    # Definição do filtro de áudio
    audio_filter_chain = "volume=10dB,compand=attacks=30ms:decays=60ms:gain=0:points=-4/0.5,loudnorm=I=-10:TP=-1.5:LRA=8"
    
    # Criamos um diretório temporário sem espaços no nome
    temp_dir = tempfile.mkdtemp(prefix="audioprocessing_")
    print(f"Usando diretório temporário: {temp_dir}")
    
    try:
        # Gera nomes de arquivo temporários sem espaços
        temp_input = os.path.join(temp_dir, f"input_{uuid.uuid4().hex}.wav")
        temp_output = os.path.join(temp_dir, f"output_{uuid.uuid4().hex}.wav")
        
        print(f"Copiando arquivo de entrada para temporário: {temp_input}")
        # Copia o arquivo original para o temporário
        shutil.copy2(input_file_path, temp_input)
        
        # Usa o pyffmpeg com os arquivos temporários sem espaços
        conversion_options = [
            '-loglevel', 'error',
            '-y',
            '-i', temp_input,
            '-filter:a', audio_filter_chain,
            '-ar', '48000',
            '-c:a', 'pcm_s24le',
            temp_output
        ]
        
        print(f"Executando pyffmpeg com caminhos temporários...")
        ff_instance.options(conversion_options)
        
        # Verifica se ocorreu erro
        conversion_error = None
        if hasattr(ff_instance, 'error') and ff_instance.error:
            conversion_error = ff_instance.error
            print(f"Erro reportado pelo pyffmpeg: {conversion_error}")
            return False
        
        # Verifica se o arquivo temporário de saída foi criado com sucesso
        if os.path.exists(temp_output) and os.path.getsize(temp_output) > 0:
            # Copia o resultado de volta para o caminho original
            print(f"Copiando resultado de volta para: {output_file_path}")
            shutil.copy2(temp_output, output_file_path)
            
            # Verifica se a cópia de volta foi bem-sucedida
            if os.path.exists(output_file_path) and os.path.getsize(output_file_path) > 0:
                return True
            else:
                print(f"ERRO: Falha ao copiar arquivo temporário de volta para {output_file_path}")
                return False
        else:
            print(f"ERRO: Arquivo temporário de saída não foi criado corretamente: {temp_output}")
            return False
            
    except Exception as e:
        print(f"ERRO durante processamento: {e}")
        return False
    finally:
        # Limpeza: remove arquivos e diretório temporário
        try:
            print(f"Limpando arquivos temporários em {temp_dir}")
            shutil.rmtree(temp_dir)
        except Exception as cleanup_error:
            print(f"AVISO: Falha ao limpar diretório temporário: {cleanup_error}")


def main():
    # Obtém o diretório de processamento
    PROCESSING_DIR = get_folder_path_from_user()
    
    print(f"\nIniciando processamento de áudio em lote no diretório: {PROCESSING_DIR}")
    print(f"Procurando por arquivos '*{TARGET_EXTENSION}'...")
    
    # Encontra todos os arquivos .wav no diretório especificado
    wav_files = glob.glob(os.path.join(PROCESSING_DIR, f"*{TARGET_EXTENSION}"))
    
    if not wav_files:
        print(f"Nenhum arquivo '*{TARGET_EXTENSION}' encontrado no diretório: {PROCESSING_DIR}")
        exit()
    
    print(f"Encontrados {len(wav_files)} arquivo(s) '*{TARGET_EXTENSION}' para processar.")
    
    # Inicializa o pyffmpeg
    ff_instance = init_pyffmpeg()
    
    # Processa cada arquivo
    for input_file_path in wav_files:
        try:
            # Constrói o nome do arquivo de saída
            base_name = os.path.basename(input_file_path) 
            name_without_ext, ext = os.path.splitext(base_name)
            
            if name_without_ext.endswith(OUTPUT_SUFFIX):
                print(f"\nPulando arquivo já processado: {input_file_path}")
                continue
    
            output_filename = f"{name_without_ext}{OUTPUT_SUFFIX}{ext}"
            output_file_path = os.path.join(PROCESSING_DIR, output_filename)
    
            print("\n" + "-" * 50)
            print(f"Processando: {input_file_path}") 
            print(f"Saída será: {output_file_path}")
            
            # Processa o arquivo usando arquivos temporários sem espaços
            success = process_audio_file(ff_instance, input_file_path, output_file_path)
            
            if success:
                print(f"SUCESSO: '{output_filename}' criado com sucesso.")
            else:
                print(f"FALHA: Não foi possível criar '{output_filename}'.")
                
        except Exception as e:
            print(f"ERRO PYTHON ao processar {input_file_path}: {e}")
        finally:
            print("-" * 50)
    
    print("\nProcessamento em lote finalizado.")


if __name__ == "__main__":
    main() 