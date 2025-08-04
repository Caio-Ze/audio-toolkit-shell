# --- YouTube Downloader com FFmpeg ---

# Dependências principais: yt_dlp (download) e pyffmpeg (fornece ffmpeg estático)
# Se yt_dlp não estiver instalado no ambiente, instala automaticamente.

import subprocess, sys, os, tempfile, shutil, uuid
from pathlib import Path
import time
import threading
from itertools import cycle

# Garantir yt_dlp
try:
    import yt_dlp
except ImportError:
    print("Instalando yt-dlp …")
    subprocess.call([sys.executable, "-m", "pip", "install", "yt-dlp"])
    import yt_dlp

# Agora os demais imports
import tempfile
import shutil
import uuid
from pathlib import Path

# Auto-install dependencies if not present
try:
    from pyffmpeg import FFmpeg
except ImportError:
    print("Instalando pyffmpeg (necessário para processamento de áudio/vídeo)...")
    subprocess.call([sys.executable, "-m", "pip", "install", "pyffmpeg"])
    from pyffmpeg import FFmpeg

# --- Spinner util custom "Segura a onda" ---
C_CYAN = '\033[1;36m'
C_GREEN = '\033[1;32m'
C_RESET = '\033[0m'

spinner_running = False
spinner_thread = None


def _spinner_loop():
    frames = ['   ', '.  ', '.. ', '...']
    idx = 0
    msg = f"{C_CYAN}Segura a onda que vai rolar"
    while spinner_running:
        print(f"\r{msg}{frames[idx]}   {C_RESET}", end='', flush=True)
        idx = (idx + 1) % len(frames)
        time.sleep(0.4)


def start_spinner():
    global spinner_running, spinner_thread
    if spinner_running:
        return
    spinner_running = True
    spinner_thread = threading.Thread(target=_spinner_loop, daemon=True)
    spinner_thread.start()


def stop_spinner():
    global spinner_running, spinner_thread
    if not spinner_running:
        return
    spinner_running = False
    if spinner_thread:
        spinner_thread.join()
    # Clear line and print success
    print(f"\r{C_GREEN}✔ FESHOW!!!{C_RESET}                ")

def get_ffmpeg_path():
    """Obtém o caminho para o binário ffmpeg através do pyffmpeg"""
    try:
        ff = FFmpeg()
        
        # Tentamos vários métodos para obter o caminho
        if hasattr(ff, 'ffmpeg_bin'):
            path = ff.ffmpeg_bin
        elif hasattr(ff, '_FFmpeg__ff_bin'):
            path = ff._FFmpeg__ff_bin
        else:
            # Procura no diretório padrão onde pyffmpeg instala o ffmpeg
            home = str(Path.home())
            default_path = os.path.join(home, '.pyffmpeg', 'bin', 'ffmpeg')
            if os.path.exists(default_path):
                path = default_path
            else:
                raise Exception("Não foi possível encontrar o caminho do ffmpeg")
        
        print(f"ℹ️ Usando ffmpeg embutido do pyffmpeg: {path}")
        return path
    except Exception as e:
        print(f"⚠️ Erro ao localizar ffmpeg: {e}")
        return None

def get_url() -> str:
    return input("\n🔗 Digite a URL do YouTube: ").strip()

def get_output_path() -> str:
    # Obtém o caminho padrão de Downloads
    default_downloads_path = str(Path.home() / "Downloads")

    while True:
        print(f"\nDigite o caminho da pasta de saída (ou arraste a pasta para aqui)")
        print(f"(Pressione Enter para usar o padrão: {default_downloads_path})")
        print("Digite '0' para cancelar e voltar ao menu")
        path_input = input("\n📁 Caminho: ").strip()
        
        if path_input == '0':
            return None
        
        # Se o usuário pressionar Enter, use o caminho padrão
        if not path_input:
            path = default_downloads_path
            print(f"INFO: Usando pasta Downloads padrão: {path}")
        else:
            path = path_input
            
        path = path.strip('"').strip("'")
        path = path.replace('\\\\ ', ' ') 
        path = path.replace('\\ ', ' ')
        
        # Verifica se o diretório existe
        if os.path.isdir(path):
            return path
        elif path == default_downloads_path and not os.path.exists(path):
            try:
                os.makedirs(path)
                print(f"INFO: Pasta Downloads padrão criada: {path}")
                return path
            except OSError as e:
                print(f"❌ ERRO: Não foi possível criar a pasta Downloads padrão: {path}\n{e}")
                continue 
        print("❌ Caminho de pasta inválido. Tente novamente ou verifique as permissões.")

def download_audio(url: str, output_path: str, ffmpeg_path: str) -> None:
    """Baixa somente o áudio e gera MP3 a 192 kbps.
    Mais rápido: prioriza m4a sem re‐encode; converte apenas no final."""

    ydl_opts = {
        'format': 'bestaudio[ext=m4a]/bestaudio/best',
        'outtmpl': os.path.join(output_path, '%(title)s.%(ext)s'),
        'ffmpeg_location': ffmpeg_path,
        'concurrent_fragment_downloads': 4,
        'noprogress': False,
        'progress_hooks': [print_progress],
        'postprocessors': [{
            'key': 'FFmpegExtractAudio',
            'preferredcodec': 'mp3',
            'preferredquality': '192',
        }],
        'postprocessor_hooks': [post_hook_spinner],
    }
    
    print("\n⏳ Baixando áudio...")
    try:
        with yt_dlp.YoutubeDL(ydl_opts) as ydl:
            ydl.download([url])
        print("✅ Download de áudio concluído!")
    except Exception as e:
        print(f"\n❌ Erro durante o download de áudio: {str(e)}")

def download_video(url: str, output_path: str, ffmpeg_path: str) -> None:
    """Baixa vídeo com fallback e garante arquivo MP4."""

    format_attempts = [
        'bv*[vcodec^=avc1][ext=mp4]+ba[ext=m4a]/b[ext=mp4]/best',
        'bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best',
        'bestvideo+bestaudio/best'
    ]

    for fmt in format_attempts:
        ydl_opts = {
            'format': fmt,
            'outtmpl': os.path.join(output_path, '%(title)s.%(ext)s'),
            'merge_output_format': 'mp4',
            'ffmpeg_location': ffmpeg_path,
            'noprogress': False,
            'progress_hooks': [print_progress],
            'postprocessor_hooks': [post_hook_spinner],
        }
        print(f"\n⏳ Tentando formato: {fmt}")
        try:
            with yt_dlp.YoutubeDL(ydl_opts) as ydl:
                ydl.download([url])
            print("✅ Download de vídeo concluído!")
            return
        except yt_dlp.utils.DownloadError as err:
            print(f"⚠️ Falhou: {err}")
            continue
        except Exception as e:
            print(f"❌ Erro inesperado: {e}")
            return
    print("❌ Não foi possível baixar o vídeo em nenhum formato suportado.")

def download_video_protools(url: str, output_path: str, ffmpeg_path: str) -> None:
    """Baixa vídeo em MP4 e aplica parâmetros compatíveis com Pro Tools."""

    base_formats = [
        'bv*[vcodec^=avc1][ext=mp4][height<=1080]+ba[ext=m4a]',
        'bestvideo[ext=mp4]+bestaudio[ext=m4a]',
        'best'
    ]

    for fmt in base_formats:
        ydl_opts = {
            'format': fmt,
            'outtmpl': os.path.join(output_path, '%(title)s.%(ext)s'),
            'merge_output_format': 'mp4',
            'ffmpeg_location': ffmpeg_path,
            'noprogress': False,
            'progress_hooks': [print_progress],
            'postprocessor_hooks': [post_hook_spinner],
            # Re-encode apenas se necessário
            'postprocessor_args': [
                '-vcodec', 'libx264',
                '-preset', 'fast',
                '-crf', '20',
                '-acodec', 'aac',
                '-b:a', '320k',
                '-movflags', '+faststart'
            ],
        }
        print(f"\n⏳ Formato PT: {fmt}")
        try:
            with yt_dlp.YoutubeDL(ydl_opts) as ydl:
                ydl.download([url])
            print("✅ Vídeo PT concluído!")
            return
        except yt_dlp.utils.DownloadError as err:
            print(f"⚠️ Falhou formato '{fmt}': {err}")
            continue
        except Exception as e:
            print(f"❌ Erro inesperado: {e}")
            return
    print("❌ Não foi possível baixar/convertê-lo para Pro Tools.")

spinner_active = False


def print_progress(d):
    if d['status'] == 'downloading':
        p = d.get('_percent_str', '?%')
        speed = d.get('_speed_str', '? KiB/s')
        eta = d.get('_eta_str', '? s')
        print(f'\rBaixando: {p} a {speed}, ETA: {eta}', end='')
    elif d['status'] == 'finished':
        print('\rDownload finalizado. Iniciando pós-processamento…       ')

# Postprocessor hook used by yt_dlp to manage spinner
def post_hook_spinner(d):
    if d['status'] == 'started':
        start_spinner()
    elif d['status'] == 'finished':
        stop_spinner()

def main_menu():
    # Obtém o caminho do ffmpeg embutido no pyffmpeg
    ffmpeg_path = get_ffmpeg_path()
    if not ffmpeg_path:
        print("❌ ERRO CRÍTICO: Não foi possível encontrar o ffmpeg através do pyffmpeg.")
        print("Por favor, verifique se o pyffmpeg está instalado corretamente.")
        print("Tentando instalar pyffmpeg novamente...")
        subprocess.call([sys.executable, "-m", "pip", "install", "--force-reinstall", "pyffmpeg"])
        ffmpeg_path = get_ffmpeg_path()
        if not ffmpeg_path:
            print("❌ Falha ao instalar/configurar o pyffmpeg. O programa pode não funcionar corretamente.")
    
    while True:
        print("\n=== 📺 Aquisição de Conteúdo do YouTube ===")
        print("1. 🎵 Baixar Somente Áudio (MP3)")
        print("2. 🎥 Baixar Vídeo (Melhor Qualidade)")
        print("3. 🎬 Baixar Vídeo (Otimizado para Pro Tools)")
        print("0. ↩️ Voltar ao Menu Principal (Sair do Script)")

        try:
            choice = input("\nDigite sua escolha (0-3): ").strip()

            if choice == "0":
                print("Saindo do YouTube Downloader.")
                break
                
            elif choice in ["1", "2", "3"]:
                url = get_url()
                if not url:
                    print("❌ URL não pode estar vazia.")
                    continue
                
                output_path = get_output_path()
                if output_path is None:
                    continue
                
                # Verifica novamente se temos o caminho do ffmpeg
                if not ffmpeg_path:
                    print("⚠️ Aviso: O caminho do ffmpeg não está disponível. Tentando continuar mesmo assim...")
                
                if choice == "1":
                    download_audio(url, output_path, ffmpeg_path)
                elif choice == "2":
                    download_video(url, output_path, ffmpeg_path)
                else:
                    download_video_protools(url, output_path, ffmpeg_path)
                    
            else:
                print("❌ Por favor, digite uma opção válida (0-3)")
                
        except Exception as e:
            print(f"\n❌ Ocorreu um erro inesperado no menu: {str(e)}")
            
        input("\nPressione Enter para continuar...")

if __name__ == "__main__":
    # Instalando dependências (isso também é feito no início do script)
    try:
        import yt_dlp
    except ImportError:
        print("Instalando yt-dlp...")
        subprocess.call([sys.executable, "-m", "pip", "install", "yt-dlp"])
        import yt_dlp
    
    main_menu() 