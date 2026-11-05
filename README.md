# Lumina Launcher

Launcher para Minecraft Bedrock (Store UWP), escrito em Rust com UI em Slint.

## Estado atual (v1)

- Detecta Minecraft UWP e Gaming Services.
- Abre o Minecraft e espera o jogo ficar pronto.
- Injeta uma DLL configurável (padrão: build local do Flarial Client OSS) e valida a injeção.
- Home + Settings com persistência em `%LOCALAPPDATA%\Lumina\Launcher\lumina.json`.
- Sem login/API nesta versão.

## Build

```powershell
cargo build --release -p lumina-app
```

O executável fica em `target\release\lumina-launcher.exe`.

## Diagnóstico

```powershell
cargo run -p lumina-app -- --diagnose
```

O relatório também é gravado em `%LOCALAPPDATA%\Lumina\Launcher\diagnose.txt`.

## Injeção direta (com Minecraft aberto)

```powershell
cargo run -p lumina-app -- --inject "C:\caminho\para\Flarial.dll"
```

## DLL de teste

```powershell
cargo build -p lumina-testdll
```

Use `target\debug\lumina_testdll.dll` como DLL no modo Custom para validar o pipeline de injeção. Ao ser injetada, ela escreve `%TEMP%\lumina_injected.txt`.

## DLL do Flarial Client OSS

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\setup-flarial-test.ps1
```

O script copia (ou compila, com `-Build`) o `Flarial.dll` para `%LOCALAPPDATA%\Lumina\Client\Flarial.Client.dll`.

## Configuração

O arquivo `%LOCALAPPDATA%\Lumina\Launcher\lumina.json` é criado automaticamente no primeiro run. Os campos principais:

| Campo | Descrição |
| --- | --- |
| `client.custom_dll` | Caminho da DLL injetada no modo Custom. |
| `client.client_mutex` | Nome do mutex criado pelo cliente injetado (padrão `Flarial`). |
| `client.client_window_class` | Classe da janela do cliente (padrão `Flarial Client`). |
| `minecraft.package_family` | Família do pacote UWP do Minecraft. |
| `minecraft.window_class` | Classe da janela principal do jogo. |
| `supported_versions` | Lista de prefixos aceitos; vazio aceita qualquer versão. |

## Limitações da v1

- Somente Minecraft Store UWP (sideloaded e Windows Beta fora do escopo).
- Sem autenticação/conta e sem canais remotos Release/Beta.
- Versions/MSIX e atualizações automáticas do launcher ficam para a próxima fase.
