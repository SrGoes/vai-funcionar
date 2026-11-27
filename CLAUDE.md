@AGENTS.md

Acompanhe as tarefas no [GitHub Issues deste repositório](https://github.com/SrGoes/vai-funcionar/issues).

## Agent skills

### Issue tracker

As issues ficam no GitHub Issues de `SrGoes/vai-funcionar` (CLI `gh`). Veja `docs/agents/issue-tracker.md`.

### Triage labels

Os cinco rótulos padrão, com o mesmo nome: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. Veja `docs/agents/triage-labels.md`.

### Domain docs

Contexto único: `GLOSSARY.md` na raiz e `docs/adr/` (criado quando houver a primeira decisão). Veja `docs/agents/domain.md`.


# Contexto desta máquina

Esta seção descreve a máquina do Goes (dono do repositório). Em outra máquina, confira a placa, a RAM e o sistema antes de usar estes números.

- GPU: NVIDIA GeForce RTX 5060, 8 GB de VRAM, driver 621.40. CPU: Intel i5-13400F (AVX2). RAM: 16 GB (o README recomenda 64 GB; o perfil mais enxuto do projeto mira 32 GB).
- Toolchain: Rust 1.98.1 (fixado em `rust-toolchain.toml`, em `%USERPROFILE%\.cargo\bin`), VS Build Tools 2026 com C++ e Windows SDK 10.0.26100.
- Build usado nas medições: `cargo build --locked --profile diag -p victory --features diagnostics` → `target\diag\victory.exe`. Build release: `cargo build --release --locked -p victory`.
- O dump fica fora do repositório e é passado por `VICTORY_GAME_DIR`, que o usuário define na própria janela. Se não estiver definida, pergunte o caminho ao usuário; não procure nem copie o dump.
- `victory.toml` (ignorado pelo git) existe com `game_dir` comentado. Cache: `%LOCALAPPDATA%\victory\cache` (shaders traduzidos: 25.756 ok, 229 sem tradução, sendo 26 programas wave64 sem conversão para wave32, que é o que a NVIDIA executa). Saves: `%LOCALAPPDATA%\victory\data`.
- Lançamento: em placa NVIDIA, `Play-VIctory.cmd` (duplo clique) ou `scripts\play.ps1 -Exe "$env:VICTORY_GAME_DIR\game_scarlett_master.exe" -Route nvidia`; placa de 4 GB, `Play-VIctory-4GB.cmd`. Em placa AMD, `scripts\play.ps1 -Exe ... -Route amd` (o padrão). Meça sempre pela rota da placa. `-DryRun` imprime o kit sem abrir o jogo; `-Safe` desliga os caminhos novos; `-RayTracing` liga o ray tracing; `-Hair` liga o cabelo. No Windows 10, `cargo xtask agility` antes do build.
- O log só vai para o console. Para guardar, rode com `2>&1 | Tee-Object <arquivo>` e com o nível desejado: `RUST_LOG` na janela, ou `-Vars "VICTORY_LOG=info"` no `play.ps1` e nos `.cmd` (o `play.ps1` apaga as `VICTORY_*` definidas na janela). Não grave logs dentro do repositório.

# Ferramentas de arquivos (clonadas em `C:\Users\gabri\Downloads\vi-tools`)

Código-fonte clonado de repositórios criados entre 19 e 24 de novembro de 2026, de autores desconhecidos. Regras:
- Leia o código antes de compilar. Revise especialmente `build.rs`, scripts `.ps1`/`.bat`/`.cmd` e scripts `install`/`prepare` do `package.json`, porque eles executam código na máquina.
- Compile você mesmo. Não baixe nem execute binários das releases desses repositórios.
- Use as ferramentas só nos arquivos da cópia do próprio usuário. Não obtenha chaves de outras fontes e não contorne proteções, como o AGENTS.md já exige.
- Saídas extraídas ficam fora do repositório.

| Pasta | Para quê | Linguagem |
|---|---|---|
| `forge9` | Inspecionar, extrair e reempacotar RPF9 com as chaves da própria cópia | Rust |
| `PercyStar` | Explorador RPF9 com player Bink2 e decoder de áudio ADAT Opus | Rust |
| `rpf9-explorer` | Explorador RPF no Windows com previews de modelo, textura, áudio e vídeo | C# |
| `OpenVI` | Explorador no estilo OpenIV, visualiza modelos e cataloga dados | TypeScript/Tauri |
| `SixNG` | CLI e app: modelos para glTF, texturas e dados de mapa | Rust |
| `vice-lens` | Explorador de mapa, modelos, animações, vídeos e som | Rust |
| `gta6-explorer` | Kit Python somente leitura: texturas, áudio, vídeo e shaders | Python |
| `rage-tools-vi` | Kit RPF9 em Python: reconstrói entradas, decodifica áudio e schema de script | Python |
| `XBOX-Inspector` | Inventário somente leitura do build do Xbox e checagem de cabeçalhos RPF9 | Rust |
| `bcpack-rs` | Decoder clean-room de texturas BCPack do Xbox (BC1 funciona, BC7 em andamento) | Rust |
| `ttpf-studio` | Ver, exportar e substituir texturas `.ttpf` | Python |
| `rage9-re` | Decodifica dados PSO de script e animações, exporta glTF | Rust |
| `AWC_from_RPF9_Extractor` | Extrai streams de música AWC | Java |
| `xbox-hle-lab` | Sondas de GPU e scripts para testar XCL em VM | C++ |
| `6stars` | Motor do zero que desenha o mapa (só macOS, Metal) | C++ |
| `GTA6-Asset-Info` | Notas de formato: RPF9, texturas, BCPack, modelos, mapa | docs |
| `0x52504639` | Notas e catálogos do formato RPF9 | docs/Python |
| `rpf9-research` | Engenharia reversa do cabeçalho, TOC e tabela de nomes do RPF9 (em espanhol) | docs/Python |
| `gta6-pc-port-hub` | Notas compartilhadas dos ports: inventário de pacotes, censo de shaders, bloqueios comuns | docs |
| `awesome-gta6-reverse-engineering` | Lista curada de ferramentas e camadas | docs |

Para o trabalho de render, as mais úteis são `bcpack-rs`, `ttpf-studio` e `GTA6-Asset-Info` (formatos e decodificação de textura, para comparar com o que o runtime sobe à GPU), `gta6-explorer` (extrair shaders e comparar com o censo do tradutor) e `gta6-pc-port-hub` (bloqueios conhecidos dos outros ports).
