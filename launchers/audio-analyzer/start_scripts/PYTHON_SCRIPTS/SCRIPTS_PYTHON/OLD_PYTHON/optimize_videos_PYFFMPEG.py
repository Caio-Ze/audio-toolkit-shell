#!/usr/bin/env python3
# -*- coding: utf-8 -*-
#
# optimize_videos_PYFFMPEG.py
# -----------------------------------
# Converte arquivos .mp4, .mov, ou .m4v para H.264 .mov pequenos.
# Usa pyffmpeg (com ffmpeg embutido) em vez de ffmpeg-python.
#
# - Resolução ~480p (scale=854:-1).
# - Áudio AAC (tamanho baixo).
# - CRF=28 (qualidade moderada).
# - Preset 'ultrafast' para codificação mais rápida.

import os
import time
import glob
import subprocess
import sys
import tempfile
import shutil
import uuid
from datetime import datetime

# Tenta importar pyffmpeg
try:
    from pyffmpeg import FFmpeg
except ImportError:
    print("O módulo pyffmpeg não está instalado. Instalando...")
    subprocess.call([sys.executable, "-m", "pip", "install", "pyffmpeg"])
    from pyffmpeg import FFmpeg

def get_directory_path():
    """
    Solicita ao usuário o caminho da pasta para processar vídeos.
    Suporta arrastar e soltar pastas no terminal.
    Trata caminhos com espaços e caracteres especiais.
    """
    print("\nPor favor, arraste e solte a pasta com os vídeos para processar no terminal")
    print("ou digite o caminho completo da pasta:")
    
    # Obtém o caminho da pasta do usuário
    directory = input("Caminho: ").strip()
    
    # Remove aspas se o usuário arrastou a pasta para o terminal (comum em macOS/Linux)
    if directory.startswith('"') and directory.endswith('"'):
        directory = directory[1:-1]
    elif directory.startswith("'") and directory.endswith("'"):
        directory = directory[1:-1]
    
    # IMPORTANTE: Substituir barras invertidas seguidas por espaço (problema comum ao arrastar pastas)
    directory = directory.replace("\\ ", " ")
    
    # Substitui outras sequências de escape comuns em caminhos
    directory = directory.replace("\\-", "-")
    directory = directory.replace("\\(", "(")
    directory = directory.replace("\\)", ")")
    directory = directory.replace("\\&", "&")
    
    # Expande o caminho do usuário (~ para home directory)
    directory = os.path.expanduser(directory)
    
    # Tenta obter o caminho absoluto para normalizar o caminho
    try:
        directory = os.path.abspath(directory)
    except Exception:
        pass  # Se falhar, mantém o caminho original
    
    # Verifica se o caminho existe
    if not os.path.exists(directory):
        print(f"Erro: O caminho não existe.")
        print(f"Caminho tentado: {directory}")
        print("Dica: Se você arrastou a pasta, tente digitar o caminho manualmente sem as barras invertidas (\\).")
        
        retry = input("Deseja tentar novamente? (s/n): ").strip().lower()
        if retry == 's' or retry == 'sim' or retry == '':
            return get_directory_path()  # Tenta novamente
        else:
            print("Operação cancelada pelo usuário.")
            sys.exit(0)
    
    # Verifica se é um diretório
    if not os.path.isdir(directory):
        print(f"Erro: '{directory}' não é uma pasta válida.")
        return get_directory_path()  # Tenta novamente
    
    print(f"Pasta selecionada: {directory}")
    return directory

def process_video_with_pyffmpeg(input_file, output_file, ff_instance):
    """
    Processa um vídeo usando pyffmpeg com arquivos temporários para evitar problemas
    com espaços nos caminhos.
    """
    # Cria um diretório temporário sem espaços no nome
    temp_dir = tempfile.mkdtemp(prefix="vidprocess_")
    print(f"Usando diretório temporário: {temp_dir}")
    
    try:
        # Nomes temporários sem espaços
        temp_input = os.path.join(temp_dir, f"input_{uuid.uuid4().hex}{os.path.splitext(input_file)[1]}")
        temp_output = os.path.join(temp_dir, f"output_{uuid.uuid4().hex}.mov")
        
        print(f"Copiando arquivo de entrada para temporário...")
        # Copia o arquivo original para o temporário
        shutil.copy2(input_file, temp_input)
        
        # Constrói as opções de conversão para pyffmpeg
        conversion_options = [
            '-loglevel', 'info',  # Mais detalhes de log
            '-y',  # Sobrescreve o arquivo de saída se existir
            '-i', temp_input,  # Arquivo de entrada
            '-vf', 'scale=854:-1',  # Redimensiona para largura 854px, altura proporcional
            '-c:v', 'libx264',  # Codec de vídeo h264
            '-crf', '28',  # Qualidade do vídeo (quanto maior, menor qualidade)
            '-preset', 'ultrafast',  # Velocidade de codificação
            '-pix_fmt', 'yuv420p',  # Formato de pixel
            '-c:a', 'aac',  # Codec de áudio AAC
            '-b:a', '128k',  # Bitrate de áudio
            '-ar', '48000',  # Taxa de amostragem de áudio
            '-movflags', '+faststart',  # Otimiza para streaming
            temp_output  # Arquivo de saída
        ]
        
        print(f"Executando FFmpeg com opções: {conversion_options}")
        
        # Chama o pyffmpeg
        ff_instance.options(conversion_options)
        
        # Verifica se houve erro
        conversion_error = None
        if hasattr(ff_instance, 'error') and ff_instance.error:
            conversion_error = ff_instance.error
            raise Exception(f"pyffmpeg reportou erro: {conversion_error}")
        
        # Verifica se o arquivo de saída foi criado
        if os.path.exists(temp_output) and os.path.getsize(temp_output) > 0:
            print(f"Conversão temporária concluída, copiando resultado para destino final...")
            shutil.copy2(temp_output, output_file)
            
            if os.path.exists(output_file) and os.path.getsize(output_file) > 0:
                return True
            else:
                raise Exception("Falha ao copiar arquivo temporário para destino final")
        else:
            raise Exception("Arquivo temporário de saída não foi criado corretamente")
            
    except Exception as e:
        print(f"ERRO durante processamento: {e}")
        return False
        
    finally:
        # Limpa os arquivos temporários
        try:
            shutil.rmtree(temp_dir)
            print("Arquivos temporários removidos.")
        except:
            print(f"AVISO: Não foi possível remover o diretório temporário: {temp_dir}")

def main():
    # Obtém o caminho da pasta do usuário
    directory = get_directory_path()
    
    # Muda para o diretório especificado
    original_dir = os.getcwd()
    try:
        os.chdir(directory)
        print(f"Diretório de trabalho alterado para: {directory}")
        print(f"Os arquivos processados serão salvos nesta mesma pasta com '_smallref' no nome.")
    except Exception as e:
        print(f"Erro ao mudar para o diretório: {str(e)}")
        return
    
    # Encontra os arquivos
    extensions = ["*.mp4", "*.mov", "*.m4v"]
    files = []
    for ext in extensions:
        files.extend(glob.glob(ext))

    # Verifica se algum arquivo foi encontrado
    if not files:
        print("Nenhum arquivo .mp4, .mov, ou .m4v encontrado no diretório especificado.")
        os.chdir(original_dir)  # Retorna ao diretório original
        return

    print(f"Encontrados {len(files)} arquivos para processar (um de cada vez).")
    start_time_total = time.time()

    # Inicializa o pyffmpeg
    try:
        ff_instance = FFmpeg()
        print("pyffmpeg inicializado com sucesso.")
        
        # Tenta encontrar o caminho do ffmpeg
        ffmpeg_path = None
        if hasattr(ff_instance, 'ffmpeg_bin'):
            ffmpeg_path = ff_instance.ffmpeg_bin
        elif hasattr(ff_instance, '_FFmpeg__ff_bin'):
            ffmpeg_path = ff_instance._FFmpeg__ff_bin
            
        if ffmpeg_path and os.path.exists(ffmpeg_path):
            print(f"Usando FFmpeg embutido do pyffmpeg em: {ffmpeg_path}")
        else:
            print("AVISO: Não foi possível determinar o caminho exato do FFmpeg")
    except Exception as e:
        print(f"Falha ao inicializar pyffmpeg: {e}")
        print("Por favor, verifique se o pyffmpeg está instalado corretamente.")
        return

    # Itera sobre a lista de arquivos
    for f in files:
        output_file = f"{os.path.splitext(f)[0]}_smallref.mov"
        print("--------------------------------------------------")
        print(f"Iniciando processamento para: {f} -> {output_file}")
        start_time_file = time.time()

        try:
            # Processa o vídeo
            success = process_video_with_pyffmpeg(f, output_file, ff_instance)
            
            if success:
                end_time_file = time.time()
                print(f"Concluído: {output_file} (Tempo: {int(end_time_file - start_time_file)} segundos)")
            else:
                print(f"ERRO: Falha ao converter {f}")
        
        except Exception as e:
            print(f"ERRO: Falha ao converter {f}")
            print(f"Detalhes do erro: {str(e)}")

    end_time_total = time.time()
    print("--------------------------------------------------")
    print("Processamento de todos os arquivos concluído.")
    print(f"Tempo total geral: {int(end_time_total - start_time_total)} segundos.")
    print(f"Arquivos processados foram salvos em: {directory}")
    print(f"Cada arquivo processado tem '_smallref' adicionado ao nome original.")
    
    # Retorna ao diretório original
    os.chdir(original_dir)

if __name__ == "__main__":
    main() 