# Prompt 09: portar o mgrein/victory-nvidia para o vai-funcionar (RTX 4080, Win10)

Antes deste prompt, cole o `00-regras-comuns.md`. Referência: `research/mgrein-victory-nvidia.md`. Este prompt **vem antes do 08 (Grimkaaah)** nos itens que se sobrepõem. O REAPER sai daqui, não do 08.

## Meta (do goal)
- 20–25 fps estáveis na 4080/Win10: média de 20 ou mais e 1% low de 15 ou mais, com PresentMon por 5 min na cena de referência, sem frame generation.
- Nenhum glitch: retângulos, texturas e shaders.
- RT só se a média continuar em 20 ou mais.

## Preparação
```
git remote add mg https://github.com/mgrein/victory-nvidia.git
git fetch mg
```
- O ancestral comum com o vai-funcionar é `0a43566`, então dá para fazer cherry-pick.
- Cada passo vai num branch próprio a partir do último passo aprovado, com `git cherry-pick -x <sha>` na ordem indicada.
- Num conflito, resolva mantendo o que o vai-funcionar já tem e acrescentando o que o commit traz. Documente arquivo e função no relatório.
- Se um conflito passar de cerca de 150 linhas, **pare e reimplemente** a partir de `git show <sha>`.
- Testes: `cargo test -p victory-shader -- --test-threads=1` e `cargo test -p victory-gpu -- --test-threads=1` no Windows.
- **Pré-requisito:** o E0 (Agility, prompt 01) e o E1 (linha de base) feitos. O mgrein não traz Agility.

## Protocolo de glitch (vale para os passos G)
1. Antes de mexer, grave 3 posições fixas de câmera para cada glitch (retângulos, texturas erradas, shader quebrado, sala escura). Registre a posição pelo save e pelo tempo da rota.
2. Para cada uma, faça um print sem HUD na mesma resolução, antes e depois, com `VICTORY_*` idênticas exceto o switch do passo. Salve como `shots/<passo>/<local>-antes.png` e `-depois.png`.
3. Rode o mesmo passo com o switch de volta ao antigo (o "=0" indicado). O glitch tem que voltar, senão a causa não foi provada.
4. No relatório, cada glitch fica com: corrigido, parcial ou não corrigido, a causa e o print.

## Passos, em ordem

| # | Branch | Commits (`cherry-pick -x`, nesta ordem) | Switch e A/B | Métrica | Manter se | Reverter se |
|---|---|---|---|---|---|---|
| **G1 Retângulos e tiles de luz** | `exp/mg-indirect-wide` | `b2f561e` (limpo), `8f2ac66` (1 conflito em `indirect.rs`: fica `wide_card = amd \|\| nvidia`) | Novo padrão: largo na NVIDIA. A/B com `VICTORY_INDIRECT_WIDE=0` | Prints da faixa de baixo da tela e de áreas com luz em tiles; teste de GPU `card_runs_wide_indirect_dispatches`; p50 | O glitch some com o padrão e volta com `=0`; p50 sem piorar mais de 3% | device hung ou TDR |
| **G2 Texturas pretas, flicker e popping** | `exp/mg-heap-split` | `d4b8094` (1 conflito em `descriptor/host.rs`) | Automático em heap limitado; `VICTORY_DYNAMIC_SLOTS` ajusta | Log "host view heap capped" com o split; tabelas recusadas por 5 s (devem ser 0); prints | Recusas em 0 e textura certa nos prints | Textura trocada nova |
| **G3 Correções do tradutor** | `exp/mg-translator` | `3f029ba`, `2bbad48`, `2056dae` (2 blocos em `emit/lower.rs`), `16fc3d7`, `77f07f0` (2 blocos em `structure.rs`), `3cf024b`. Incremente `TRANSLATOR_VERSION` se a retradução mudar algo. | `VICTORY_GROUP_SYNC` sem definir fica automático. Os outros são correções sem switch. | `victory translate <pasta> --validate --pipelines`: recusados antes e depois; `victory shaders` (quantos mudaram); prints de shaders quebrados; VRAM (o `3f029ba` tira local memory); device hung em 30 min | Menos recusados, nenhum erro de validação novo e prints iguais ou melhores | Qualquer erro de validação novo ou glitch novo |
| **P-A Perfil das portas na NVIDIA (E5/E6)** | `exp/mg-ports` | Só configuração no `Play-VIctory.cmd`: `PORT_CONSTANTS=1`, `PORT_CBV=1`, `PORT_DESCRIPTORS=1`, `PORT_GUARD=1`, `PORT_JOURNAL=1` (como `3032ff0` e `2c40c82`). **Não** ligar `PORT_CBV_SLOT`. | Uma porta por vez, depois todas | p50 e p95; preparo do envio (`VICTORY_PANEL`); estrada visível (#15); exceção C++; device loss em 30 min | p50 cai 10% ou mais (na 4070: 80 → 67 ms) e o visual fica igual | Estrada sumindo, exceção ou device loss |
| **P-B Threads do THIN mais baratas (E12)** | `exp/mg-views` | `c263560` (limpo), `f9bcd8a`, `fcb2789`, `b1737be`, `fc02789`, `6381d40` (conflitos em `descriptor*.rs`), `54ef67f`, `982b9f0` (conflitos em `descriptor/host.rs`, `lib.rs`, `shadow.rs`), `8a34ef8`, `81b6d6f`, `f62988c` | `VICTORY_VIEW_COPIES`, `VICTORY_CHUNK_VIEWS`, `VICTORY_EXACT_ARENA` ligados por padrão; A/B com `=0` em cada um | ms das threads de tradução e de gravação (nsys ou `VICTORY_PANEL`); p50; `THIN=1` contra `0` | Threads 10% mais baratas ou mais e p50 cai 5% ou mais | Lixo de descritor (textura trocada) |
| **P-C REAPER e listas (substitui o P3 do 08)** | `exp/mg-lists` | `a2cf68b` (conflito em `host.rs`), `94ae120`, `cc94c34`, `d9466bd`, `8e172e0`, `ed83072`, `a132b1d`, `0b44af5` | `VICTORY_REAPER` (padrão ligado, `=0` volta ao inline), `LIST_POOL_CLOSED`, `LIST_FAT_SWAP`, `PRELUDE_POOL` | Preparo do envio; listas criadas e descartadas por quadro (linha "frame account"); RAM privada | 2 ms ou mais a menos no preparo, com RAM estável | Device removed |
| **P-D Guarda de RAM (E9, lado da RAM)** | `exp/mg-ramtrim` | `046c163` (conflitos em `host.rs` e `pipeline/make.rs`), `4519ec6`, `44f2b17` | `VICTORY_RAM_TRIM_MB`, `VICTORY_PIPELINE_COMMIT_MB` | Soak de 30 min: OOM, folga de commit e quadros acima de 200 ms | 30 min sem OOM | Mais travadas que a base |
| **P-E Memória e texturas** | `exp/mg-mem` | `3393174`, `46911ac`, `5edad3d` (title-windows); `cb65249`, `5340e71`, `045a270`, `19b34e9` (memory-inplace); `3de3c24` | Na 4080, forçar `VICTORY_TEXTURE_MAX=<MB>` (não ativa acima de 12 GiB) só se a VRAM passar de 90% | VRAM e memória compartilhada; textura atrasada (prints); p50 | Memória compartilhada 1 GB menor ou mais, sem textura errada | Texturas borradas fixas ou lixo |
| **P-F Write watch (E7)** | `exp/mg-watch` | `871d2d9`, `d90b4e4`, `89b9f66`, `ea38855`, `1ff04bc` (limpo). **Não** trazer as guardas frias (`7725f06`), retiradas em `08f24db`. | não se aplica | ms de write watch no painel; MB enviados por quadro | 2 ms ou mais a menos, sem escrita perdida | Ruído em textura ou dado velho |
| **P-G Build e latência** | `exp/mg-build` | `5790469` (conflito em `Cargo.toml`: só as flags de LTO e x86-64-v3), `404a293`, `87274d9` (limpo) | `VICTORY_FRAMES_AHEAD=2` (padrão na NVIDIA). A/B contra o P2 do 08 (`SUBMIT_FRAMES=2`), primeiro sozinhos e depois juntos. Testar `VICTORY_LANE_CORE` forçado no 9800X3D. | p50 e p99; tempo de espera no present | p50 cai 3% ou mais e o p99 não piora | Engasgos |
| **G4 Builder de RT em 32 lanes (pré-requisito de RT)** | `exp/mg-wave-same` | `f48bacb` (3 blocos: `emit/lower.rs`, `emit/mod.rs`, `wave64_tests.rs`), `a7be7cf`, `9c4acb6`, `db37d50` | `VICTORY_WAVE_SAME` e `VICTORY_WAVE_LIVE` (padrão ligado; `=0` para o A/B). Muda a chave do cache: rode `victory shaders` de novo. | `victory translate --pipelines XboxBvhBuilder.dll` com `VICTORY_HOST_WAVE=32` (deve dar 52/52); com `BVH_BUILDER=1`, prints de interiores (escuros antes) e fps | 52/52 e interiores com luz | Device hung |
| **RT-1 DXR 1.1 (último)** | `exp/mg-native-rt` | `73b6e81` (limpo), `674d5f7` (cerca de 7 blocos), `41273b8` (cerca de 5 blocos em `host.rs`), `c4b9570` (limpo), `6c1614b`. Acima de cerca de 150 linhas de conflito, trazer `dxr.rs`, `rayquery_lift.rs` e `bvh_builder.rs` inteiros e reimplementar só os hooks. | `VICTORY_NATIVE_RT=1` mais `BVH_BUILDER=1`, depois `victory shaders`. A/B com o caminho emulado (`NATIVE_RT` desligado). `NATIVE_RT_SUPPRESS_BUILDER=1` em separado. | `RaytracingTier` no log; `victory translate` ("ray tracing …": programas lifted, o README diz 47/64); fps e 1% low; VRAM e RAM (na 4070 deu 28–29 GB de pico); prints de interiores | **Só entra no build final se a média ficar em 20 ou mais e o 1% low em 15 ou mais com RT**, sem estourar memória em 30 min | Qualquer queda abaixo da meta: RT fica desligado e o relatório diz o número |

## Não trazer
- `-SmoothMotion` / `victory_sm.exe`: é geração de quadro pelo driver, e o goal proíbe.
- `VICTORY_PORT_CBV_SLOT=1`: derrubou o device e depois não ganhou nada na 4070 (`877d5b4`, `9849f04`).
- As guardas de faixas frias (`7725f06`, `COLD_GUARDS`): triplicavam as cópias.
- `VICTORY_POOL_DRAIN=1`: custou quadros com RT.
- `VICTORY_WINDOWS_BAR_MB`: só funciona no Win11.
- `VICTORY_REFLEX=1` nas medições. Pode ser testado depois, para latência.

## Relatório por passo
Use o formato do `00`, mais:
- SHAs aplicados, e os conflitos resolvidos por arquivo e função;
- números da 4080 antes e depois (p50, p95, p99, 1% low, VRAM, RAM e a métrica do passo);
- a tabela de glitches com os prints;
- a linha preenchida no `checklists/plano-2-semanas-4080.md`.

No fim da sequência, monte um build só com os passos mantidos, com a tag `build-AAAA-MM-DD`. O veredito vai na 1ª linha (MELHOR, IGUAL ou PIOR), com os números reais.
