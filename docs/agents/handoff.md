# Passagem de bastão (estado em 27/11/2026)

## Atualização do fim de 27/11 (ler primeiro)
A `main` recebeu, depois do que está descrito abaixo, os PRs #46 a #50, #52 e #54. O que muda:

- **Antes de abrir depois de atualizar: refaça o pré-aquecimento, os dois passos.** O #46 mexe no tradutor e muda a chave do cache de shaders. Rode `target\diag\victory.exe shaders <arquivos .rpf>` e depois `target\diag\victory.exe warm --used`, com o mesmo `VICTORY_CACHE_DIR` que o launcher usa (na rota nvidia, `%LOCALAPPDATA%\VIctory\cache-pareto`).
- **Com o cache de pipelines frio a rota nvidia cai no primeiro quadro do mundo** (`0xc0000005`, escrita em 0, dentro do código do jogo): issue #53. Visto três vezes no mesmo executável: duas quedas só com `victory shaders`, nenhuma depois de `warm --used`. Numa máquina onde o cache da rota nunca foi usado, `warm --used` não tem lista de pipelines para seguir.
- **#46, tradução de verdade do #30:** o defrag (`02384a60fda6f2d0`) passou a traduzir e a rodar. Ficam 231 sem tradução, 36 deles em grupos de 64. Os dois testes na placa batem bit a bit. No jogo: 2.137 dispatches em 3 minutos sem queda; **a imagem não foi conferida** (outro fork viu grade colorida nos rostos com o defrag rodando). `VICTORY_DEFRAG=0` desliga, mas a chave não entra na chave do cache: use um cache separado para comparar.
- **#47, limpeza de listas fora da thread de envio (do mgrein):** entrou sem medição. `VICTORY_REAPER=0` volta ao caminho antigo.
- **#49:** o `Play-VIctory.cmd` escolhe o perfil de console por palavra (`series-s`, `series-x`) e, sem palavra, pelo automático. **#50:** contadores de diagnóstico (views nulas por quadro; um aviso por conjunto de shaders descartado). **#52:** o `AGENTS.md` é a fonte única; máquinas em `machines.md`. **#54:** `scripts/measure/presentmon.ps1`.
- **Teste de abertura na máquina A, pela pasta principal na `main`:** `Play-VIctory.cmd` chegou ao mundo em 40 s e rodou a 13-15 FPS com ~4.200 draws por quadro (release, Series S pelo automático, cache aquecido). No build de diagnóstico a memória privada chegou a 27,6 GB, com menos de 1 GB de RAM livre.
- **Onde está o trabalho que falta:** #36 a #40 (relatos do playtest), #41 (hipótese no reaproveitamento; a leitura do código não a confirma), #42 (limpezas), #43 (26 commits do upstream revisados, por ordem de valor; não usar "Sync fork"), #44 (primeira rodada do medidor pelo `play.ps1`), #45 (cabelo por padrão), #53 (queda com cache frio). O desenho da tradução do #30 e a classificação das recusas em grupos de 64 estão em `findings/2026-11-27-desenho-da-traducao-30-e-recusas-de-grupos-de-64.md`.

## Máquina B (16 GB de RAM), 27/11 à tarde: o que a fez chegar ao mundo aberto
Branch `fix/shared-budget-guard`. Rota nvidia, build de diagnóstico da `main` em `5487f38` mais o freio, Series S, cache aquecido (`victory shaders`, uma rodada com `VICTORY_PIPELINE_BOOK=1`, depois `warm --used`: 2.006 pipelines).

- **A queda da máquina B não era o cabelo nem o #53:** era a memória do PC compartilhada com a placa. O Windows dá à placa metade da RAM (7.446 MB nesta máquina). As janelas de memória do jogo (~5 GB) mais o staging das texturas de um mundo carregando passaram disso. Os buffers de upload falharam com `E_OUTOFMEMORY` (641 vezes numa rodada), e o driver caiu ao fechar uma lista em `Host::submit` (`nvwgf2umx.dll+0x422e03`, leitura em `0x36`). Isso aconteceu três vezes, entre 7 e 11 minutos. O commit e a RAM livre, que são o que os freios de `residency.rs` olhavam, não viam o problema: com pagefile de 40 GB, sobravam 30 GB de commit.
- **Freio novo (`residency.rs`):** o nível de uso do grupo compartilhado do DXGI entra nos freios que já existiam. A partir de 85% é pressão; de 90%, o staging especulativo para de crescer; de 95%, é crítico; de 97%, só sobem as texturas que o draw precisa. `VICTORY_SHARED_GUARD=0` desliga. Sozinho, só atrasou a queda (de 7 para 11 min), porque as janelas continuavam crescendo.
- **O que fez passar:** a chave nova `-SmallRam` do `play.ps1`, com o atalho `Play-VIctory-16GB.cmd`:
  - saída em 1280x720 com o upscaler do próprio jogo (`VICTORY_OUTPUT`, `VICTORY_UPSCALER=6`). O FidelityFX da Series S divide por zero abaixo de 1080p, em `+0x28f03e8`, e foi visto uma vez;
  - `VICTORY_TEXTURE_MAX=1024`;
  - `VICTORY_WINDOWS_BAR_MB=1536`;
  - cabelo ligado;
  - defrag do #46 de fora.

  Com isso a rodada passou pela cutscene da prisão e chegou à perseguição de barco do mundo aberto, a 7-10 FPS, com a memória compartilhada em 5,6-5,8 GB. Rodou mais de 20 minutos sem cair, apesar de 296 falhas de upload no pico de carregamento.
- **Imagem dessa rodada:** cabelo presente (Lucia e NPCs), personagens com textura, água e céu bons. Nas rodadas anteriores, o preto do cenário vinha de uma fila de 13 a 25 mil texturas esperando subir (`VICTORY_TEXTURE_NO_WAIT=1`). Com o teto de 1024, a fila caiu para 6-8 mil no mundo. Os interiores continuam escuros e chapados por causa da luz substituta (`constant:0.8`), já que o ray tracing está fora. O ruído colorido num cartaz continuou mesmo com o defrag fora.
- **Nada disso é padrão do kit nvidia:** é a chave `-SmallRam`. Mudar o padrão fica para o Bruno, com as imagens.
- **Não medido:** FPS em ponto fixo com 3 rodadas. Os números acima são de uma rodada por configuração, em cenas que andam sozinhas. As janelas finas (`VICTORY_WINDOW_KB=256` com `VICTORY_WINDOW_TAILS=1`) não reduziram as janelas (4,95 GB em 8.653 janelas) e o FPS pareceu menor, por isso ficaram de fora. O kit da madrugada (PR #56) não mostrou ganho aqui numa rodada (6,13 FPS na base, 5,76 com o kit); `VICTORY_REAPER=0` deu 4,79.
- **Para onde vai o quadro com `-SmallRam`** (linha do tempo aos 300 s e paginação por WMI; ver `findings/2026-11-27-trava-exact-e-paginacao-maquina-b.md`):
  - 94% da espera dentro das chamadas de lista longas é a trava de `shadow::exact`, que varre ~4.000 cópias a cada miss (300-400 misses por quadro) e cria buffers segurando a trava;
  - a RAM vai para o disco o tempo todo (mediana de 9.644 páginas por segundo; RAM livre de 911 MB).

  **Próximo passo:** abrir uma vez o build `test/night-smallram` (PR #56 com o PR #57, já compilado) com `-SmallRam` e a linha do tempo nos mesmos horários, para medir o `EXACT_LAZY` aqui. Depois, tirar a criação de buffer de dentro da trava.

## Em uma frase
O port abre e chega ao mundo em placa NVIDIA no Windows 10, a 8-12 FPS e sem o roxo, mas está abaixo dos dois pisos de "jogável": falta FPS, a água tem retângulos, a vegetação pisca ao andar e a iluminação do jogo não roda.

## As duas máquinas
O trabalho acontece em duas máquinas, as duas com GeForce. Todo número diz de qual delas veio.

| | Máquina A (medições de 26 e 27/11) | Máquina B (dono do repositório; a do `CLAUDE.md`) |
|---|---|---|
| Placa | RTX 4080 16 GB, driver 621.24, Resizable BAR ligado | RTX 5060 8 GB, driver 621.40 |
| Processador | Ryzen 7 9800X3D | Core i5-13400F |
| RAM | 32 GB | 16 GB |
| Sistema | Windows 10 Pro 19045 (precisa de `cargo xtask agility`) | não registrado aqui |
| Rota | nvidia | nvidia |
| Perfil de console usado | Series S, por um launcher local com o kit da rota nvidia e o cache normal | Series S (#2) e Series X (#3) |

- **Rota igual nas duas:** nvidia. Número medido pela rota amd não vale para nenhuma.
- **O que as separa hoje é a RAM, não a placa.** O port usa 24-25 GB de RAM na máquina A com o kit da rota nvidia. Na máquina B, com 16 GB, a sessão em Series S cai depois de ~9 min (#2) e Series X não se sustenta (#3). Baixar a memória do port é o que destrava a máquina B.
- **Perfil de console:** o `Play-VIctory.cmd` escolhe pela memória do PC (Series S abaixo de 48 GB de RAM ou 12 GB de VRAM) e aceita `series-s` ou `series-x` como primeira palavra. Antes do PR #49 ele forçava Series X. Nada foi medido em Series X na máquina A.
- As duas placas têm folga para o jogo: na máquina A a placa fica 43-54% ocupada e quem limita o FPS é o processador preparando cada quadro (ver abaixo).

## Noite de 27/11 (máquina A, PR perf/night-1127)
Medido pelo `bench.py` numa cena fixa (o save é restaurado antes de cada rodada: o título salva sozinho e trocou a cena original às 06:24), rota nvidia, Series S, build diag:

| | ontem (main 5487f38) | noite (kit nvidia novo) |
|---|---|---|
| 3 rodadas de 100 s | 9,30 FPS, p50 95,7 ms, p99 262 ms | 12,1-13,4 FPS, p50 57,7 ms |
| 15 min contínuos | 9,75 FPS, 1% low 3,14, p50 96, p99 219, sem crash | 15,44 FPS, 1% low 3,89, p50 58, p99 159, sem crash |

- **O que limitava:** as 5 threads de gravação do título passavam 70-80 ms/quadro nas chamadas de lista, na maior parte esperando a trava `EXACT` (`shadow::exact`), que varria ~8.700 cópias a cada miss. Perfil por função: `VICTORY_TIMELINE_RECORDERS=1` com `VICTORY_TIMELINE`.
- **O que entrou no kit nvidia** (`scripts/play.ps1`): `EXACT_LAZY` (varre a cada 64 misses; a maior parte do ganho), `COPY_BY_RANGE`, `FLUSH_GROUPED`, `CARD_STEPS_JOIN`, `PORT_GUARD`, `PORT_GUARD_PLAN`, `ASK_NORESET`. Fora do kit: `TEXTURE_ONCE_A_FRAME` (sem A/B de imagem), `PLACED_QUICK` e `THIN=1` (sem ganho).
- **Sem ganho medido:** `WAVE_WIDE=0`, `UPSCALER=6`+`RENDER_SCALE=0.5` (a placa não limita), `settings.xml` com LOD menor (sem efeito nos draws), `VICTORY_OUTPUT=1280x720` (cai: FSR4 exige 1920x1080).
- **Ray tracing:** o construtor (52 programas de `XboxBvhBuilder.dll`) traduz e valida em grupos de 32 (`VICTORY_HOST_MAX_WAVE=32 translate --validate`); rodada no jogo: ver `RELATORIO-AMANHA.md`.
- **Ferramentas novas:** `VICTORY_FRAME_TIMES=<csv>` (tempo de cada quadro, para 1% low/p99), `VICTORY_AB` cobre as chaves novas.
- **Memória:** o pico subiu de ~28,2 para ~29,1 GB em 15 min; a RAM livre chegou a ~120-320 MB em 32 GB. É o próximo risco.

## O que entrou na main em 27/11 (PRs #31 a #34)
| Peça | Commits | Estado |
|---|---|---|
| Reaproveitamento das cópias de views (conserto da regressão de 12 para 3 FPS que entrou com o PR #28; issue #29, fechada) | PR #32, mesclado | Medido: 3,35 FPS sem, 12,49 com, 3 rodadas no mesmo ponto |
| Agility SDK para o Windows 10 (`cargo xtask agility`) | PR #31, mesclado | Sem ele o port cai no primeiro quadro no Windows 10 |
| Conserto mínimo do pânico do tradutor (#30) | `f0e268b` | `victory shaders` termina: 25.657 traduzidos, 232 sem tradução (195 sem programa e 37 em grupos de 64 sem conversão). O número do `CLAUDE.md` (229, com 26) é de outra máquina e de uma árvore anterior |
| Reserva de tabelas dinâmicas em heap limitado (do mgrein `d4b8094`) | `69c50a0`, `413bbc0` | Sem efeito medido em Series S (0 tabelas recusadas); vale para Series X. O `413bbc0` tira o heap de samplers desse ramo (defeito achado em revisão, inofensivo para a imagem) |
| Dispatch indireto largo na NVIDIA (do mgrein `b2f561e`, `8f2ac66`) | `019b8ba`, `eeba481` | Só dispara em 2560x1440 (Series X); não medido aqui |
| Eixo do analógico em `VICTORY_KEYS` (vale em qualquer build) e captura por tempo, com tempos contados do primeiro quadro do mundo (só no build de diagnóstico; no release o tempo `w<s>` nunca dispara) | `b5b2513` | Testes unitários ok; giro de câmera no jogo ainda não provado |
| Launcher com rota explícita, norte, glossário, `docs/agents` | PR #33, mesclado | Dois revisores aprovaram; os dois atalhos abrem o jogo |
| Esta passagem de bastão e os relatos de playtest | PR #34 | |
| Medidor (`bench.py`, `matrix.py`) e comparadores de imagem (`ab.py`, `vis360.py`) em `scripts/measure/`, com README | PR #35, sobre este | Testados offline nas rodadas de 26 e 27/11; a primeira rodada pelo `play.ps1` ainda não foi feita |

Os achados citam os hashes dos branches de origem: o conserto mínimo do #30 é `36b998b` em `fix/30-half-meet-block` (aqui `f0e268b`). O filtro de sondas magenta (`562e4a5`, do Belobug `e942896`) foi testado e NÃO entrou: não resolveu o roxo no modo estimado.

**Cache de shaders:** o `f0e268b` mexe no tradutor, então a chave do cache muda em relação à `main` anterior: quem vem dela precisa refazer o pré-aquecimento. Os consertos do mgrein e a captura não mudam a chave.

## O que está provado aqui
- **Regressão do PR #28 (issue #29):** o commit `8977bb0` manda toda view estruturada fora de 16 bytes para `shadow::exact()`, cujo cache é por endereço; o jogo muda o endereço todo quadro, e a NVIDIA (sem slot gêmeo) criava ~600 buffers por quadro. O reaproveitamento resolve. Detalhe em `findings/2026-11-27-regressao-28-e-reaproveitamento.md`.
- **Roxo:** vem da luz substituta em modo estimado. `VICTORY_AMBIENT=constant:0.8` (o valor da rota nvidia) tira. O preço é a imagem chapada.
- **O que falta em cada quadro (log do build de diagnóstico):** ~900-940 execuções "shades ray-tracing hits without BVH" (a iluminação do jogo), 258-283 estágios do construtor de ray tracing, e 1 por quadro de cada compute sem tradução: `8b437ab61d8008fe`, `477b52e73ddeb6f2`, `17af85f170e66f30` e o defrag `02384a60fda6f2d0`. Quase tudo o que falta é ray tracing.
- **O limite de FPS é o processador:** 74-94 ms de quadro, dos quais 51-57 ms de preparo (cópias 14-22 ms, texturas 11-19 ms) e 12-17 ms de cópia de volta; 34-72 mil chamadas de lista por quadro. Com o kit da rota nvidia (que inclui `VICTORY_CARD_BUFFERS_MIN_KB=64`) a placa cai de 92% para 43-54% ocupada e o FPS não sobe; qual chave do kit responde por isso não foi isolado.
- **PresentMon (kit antigo, 60 s):** 11,1 FPS, quadro mediano 90 ms, p95 145, p99 181, sem VSync. Release e diagnóstico rendem igual.
- **Memória:** 24-25 GB privados com o kit da rota nvidia, RAM livre caindo abaixo de 1 GB em 32 GB. Um vazamento pequeno, de 30-50 MB por minuto, foi medido numa sessão de 61 min do build antigo; na série de 6 min do build atual ele não aparece.
- **Reserva de tabelas:** de 14 a 28 mil tabelas pedidas por quadro conforme a cena (máximo visto 28.264) e 0 recusadas em Series S. Não é a causa do pisca aqui.

## O que NÃO está provado
- Qualquer ganho de FPS por chave: as rodadas de `VICTORY_THIN=1` e `VICTORY_ASYNC_SUBMIT=1` foram únicas e em cenas diferentes.
- As causas dos defeitos visuais em `gaps.md` e dos relatos em `relatos.md` (issues #36 a #40): são hipóteses com teste definido.
- A tradução de verdade do #30 (PR #46, já na `main`): os testes na placa passam; falta conferir a imagem com o defrag rodando.
- O efeito dos dois consertos do mgrein em Series X, e Series X nesta máquina (o modo automático só o escolhe com 48 GB de RAM).
- As portas do renderer na NVIDIA (`VICTORY_PORT_*`): um fork mediu ganho numa RTX 4070, outro mediu perda numa RTX 5070.
- O ray tracing do jogo em placa de grupos de 32: nunca houve rodada com `VICTORY_BVH_BUILDER=1` aqui.

## Próximos passos, por ordem
Os passos 1 a 3 usam o medidor e os comparadores de `scripts/measure/` (falta a primeira rodada por eles, #44).

1. Comparação visual com giro de câmera, cada uma contra a base: `VICTORY_WAVE_WIDE=0` (retângulos e árvores), `VICTORY_TEXTURE_NO_WAIT=0` e `VICTORY_UPLOAD_AHEAD=0` (braço e escada pretos), guarda de textura no padrão.
2. Se o item 1 confirmar a forma wide: portar a forma paired do fxceless atrás de chave (ver `references.md`), com o defrag fora.
3. FPS por configuração, no mesmo ponto fixo, 3 rodadas: portas ligadas, `VICTORY_CARD_AWAKE` desligada, `VICTORY_PIPELINE_BOOK=0`, juntas e separadas.
4. FPS por código: medir o `a2cf68b` do mgrein, que já entrou (PR #47), com e sem `VICTORY_REAPER=0`; depois medir quanto do preparo é criação de views antes de portar a pilha de cópias de views.
5. Conferir a imagem com o defrag rodando (PR #46) e decidir entre estender os half-meets e a forma paired para os 36 programas.
6. Ray tracing, passo 0, sem jogo: traduzir o construtor com `VICTORY_HOST_MAX_WAVE=32` e listar o que recusa (`findings/2026-11-27-ray-tracing-em-grupos-de-32.md`). O caminho da DLL do construtor, que fica na pasta do jogo, é o usuário quem fornece.
7. Memória: `VICTORY_TEXTURE_BUDGET_MB=4096`, `VICTORY_WINDOW_BUDGET_MB=1024` com `VICTORY_WINDOW_IDLE_MS=10000`, `VICTORY_UPLOAD_AHEAD=0`.

## Branches e PRs fora da `main`
| O quê | Estado |
|---|---|
| `fix/26-gpu-test-isolation` (#26) | Pausada, só na máquina A |
| PR #19, diagnóstico de memória | Antigo; rebasear ou fechar |
