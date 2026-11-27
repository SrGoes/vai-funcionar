# Ferramentas de arquivos do jogo

Cada máquina clona estas ferramentas numa pasta fora do repositório; o caminho fica no `CLAUDE.local.md` dela.

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
