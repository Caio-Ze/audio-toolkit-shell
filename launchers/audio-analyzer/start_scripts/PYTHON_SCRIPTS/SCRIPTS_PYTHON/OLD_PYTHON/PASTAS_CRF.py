#!/usr/bin/env python3
import os
import sys
import subprocess
import time
import re

def get_lines_from_clipboard():
    """Obtém o conteúdo da área de transferência (macOS) e divide em linhas."""
    try:
        result = subprocess.run(['pbpaste'], capture_output=True, text=True, check=True)
        # Divide o conteúdo em linhas e remove linhas vazias
        lines = [line.strip() for line in result.stdout.splitlines() if line.strip()]
        if not lines:
            print("ERRO: Área de transferência está vazia ou contém apenas linhas em branco.")
            return None
        # Validação básica de cada nome base
        valid_lines = []
        for line in lines:
             # Remove aspas simples ou duplas que podem vir do Finder
             if line.startswith("'") and line.endswith("'"):
                 line = line[1:-1]
             elif line.startswith('"') and line.endswith('"'):
                 line = line[1:-1]

             # Salva o estado da linha após a remoção de aspas, para log da transformação específica
             line_after_quotes = line

             # --- MODIFIED LÓGICA para substituir o espaço entre um bloco de números e um bloco de letras ---
             # Usa regex para maior robustez com diferentes tipos de espaços e para garantir o padrão NUMEROS<espaços>LETRAS.
             # Exemplo: "12345   ABC" se torna "12345_ABC"
             # Esta transformação é aplicada antes da validação de separadores de caminho.
             match = re.match(r"(\d+)\s+([a-zA-Z]+)$", line_after_quotes) # Regex for "NUMBERS<whitespace>LETTERS"
             if match:
                 first_part = match.group(1)
                 letters_part = match.group(2) # Já validado como letras pelo regex
                 transformed_line = f"{first_part}_{letters_part}"
                 # Apenas para log, se a linha foi realmente modificada por esta regra
                 if line_after_quotes != transformed_line:
                     print(f"  INFO: Specific pattern '{line_after_quotes}' transformed to '{transformed_line}' (number-text pattern with underscore).")
                 line = transformed_line
             # --- FIM DA MODIFIED LÓGICA ---

             if '/' in line or '\\\\' in line:
                 print(f"AVISO: Nome base '{line}' contém separadores de caminho (/, \\\\) e será pulado.")
                 continue
             if not line: # Pula se a linha ficar vazia após strip/remoção de aspas
                 continue
             valid_lines.append(line)

        if not valid_lines:
            print("ERRO: Nenhuma linha válida encontrada na área de transferência após validação.")
            return None

        return valid_lines
    except FileNotFoundError:
        print("ERRO: Comando 'pbpaste' não encontrado. Este script é para macOS.")
        sys.exit(1)
    except subprocess.CalledProcessError as e:
        print(f"ERRO ao executar pbpaste: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"ERRO inesperado ao ler da área de transferência: {e}")
        sys.exit(1)

def get_destination_directory_from_user():
    """Solicita ao utilizador o caminho da pasta de destino."""
    while True:
        raw_path = input("\nPor favor, arraste a PASTA DE DESTINO (onde as novas pastas serão criadas) para esta janela e pressione Enter: \n-> ")
        cleaned_path = raw_path.strip()
        if cleaned_path.startswith("'") and cleaned_path.endswith("'"):
            cleaned_path = cleaned_path[1:-1]
        if cleaned_path.startswith('\\"') and cleaned_path.endswith('\\"'):
            cleaned_path = cleaned_path[1:-1]
        if cleaned_path.startswith('"') and cleaned_path.endswith('"'):
            cleaned_path = cleaned_path[1:-1]
        # Lida com espaços escapados de arrastar e soltar
        cleaned_path = cleaned_path.replace('\\ ', ' ')
        cleaned_path = cleaned_path.replace(r'\ ', ' ')

        if os.path.isdir(cleaned_path):
            return cleaned_path
        else:
            print(f"ERRO: O caminho '{cleaned_path}' não é um diretório válido. Por favor, tente novamente.")

def create_and_rename_dir_structure(destination_dir, base_name):
    """Cria a estrutura de diretório base_name/LOC e renomeia base_name se necessário."""

    original_full_path = os.path.join(destination_dir, base_name)
    loc_path = os.path.join(original_full_path, "LOC")

    # --- 1. Criação ---
    try:
        os.makedirs(original_full_path, exist_ok=True)
        print(f"  Diretório criado: {original_full_path}")
        os.makedirs(loc_path, exist_ok=True)
        print(f"  Subdiretório criado: {loc_path}")
    except OSError as e:
        print(f"  ERRO ao criar diretórios para '{base_name}': {e}")
        return False # Falha na criação
    except Exception as e:
        print(f"  ERRO inesperado ao criar diretórios para '{base_name}': {e}")
        return False

    # --- 2. Renomeação (usando regex para todos os espaços) ---
    # Substitui todas as sequências de um ou mais caracteres de espaço por um único underscore
    potential_new_basename = re.sub(r"\s+", "_", base_name)

    if potential_new_basename != base_name: # Ocorreu uma substituição de espaços
        new_full_path = os.path.join(destination_dir, potential_new_basename)

        if os.path.exists(new_full_path):
            print(f"  ERRO: Destino da renomeação '{new_full_path}' já existe. Pulando renomeação de '{original_full_path}'.")
        else:
            try:
                print(f"  Renomeando: '{original_full_path}' -> '{new_full_path}'")
                os.rename(original_full_path, new_full_path)
                print(f"  Renomeado com sucesso para '{potential_new_basename}'.")
            except OSError as e:
                print(f"  ERRO ao renomear '{original_full_path}' para '{potential_new_basename}': {e}")
            except Exception as e:
                print(f"  ERRO inesperado ao renomear '{original_full_path}': {e}")
    else:
        # Nenhuma substituição de espaço ocorreu, nome já está no formato desejado ou não continha espaços.
        print(f"  INFO: Nome '{base_name}' não requer renomeação geral de espaços.")

    return True # Sucesso geral (criação + tentativa de renomear)

if __name__ == "__main__":
    # 1. Obter nomes base (um por linha) da área de transferência
    base_names = get_lines_from_clipboard()
    if not base_names:
        sys.exit(1)
    print(f"\n{len(base_names)} nomes base lidos da área de transferência:")
    for name in base_names:
        print(f"- {name}")

    # 2. Obter diretório de destino do usuário (uma vez)
    destination_dir = get_destination_directory_from_user()
    print(f"\nDiretório de destino selecionado: {destination_dir}")
    print("\n--- Iniciando criação e renomeação ---")

    # 3. Iterar, criar e renomear para cada nome base
    success_count = 0
    failure_count = 0
    processed_count = 0
    for name in base_names:
        processed_count += 1
        print(f"\nProcessando {processed_count}/{len(base_names)}: '{name}'")
        if create_and_rename_dir_structure(destination_dir, name):
            success_count +=1
        else:
            failure_count += 1

    print("\n--- Resumo ---")
    print(f"Total de nomes processados: {len(base_names)}")
    print(f"Estruturas criadas com sucesso (ou parcialmente*): {success_count}")
    print(f"Falhas na criação inicial: {failure_count}")
    print("*Nota: 'Sucesso' inclui casos onde a criação ocorreu mas a renomeação pode ter falhado se o destino já existia.")
    print("\nScript finalizado.") 