# Passagem de bastão (estado em 27/11/2026)

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
- **Perfil de console:** o `Play-VIctory.cmd` escolhe pela memória do PC (Series S abaixo de 48 GB de RAM ou 12 GB de VRAM) e aceita `series-s` ou `series-x` como primeira palavra. Até 27/11 ele forçava Series X. Nada foi medido em Series X na máquina A.
- As duas placas têm folga para o jogo: na máquina A a placa fica 43-54% ocupada e quem limita o FPS é o processador preparando cada quadro (ver abaixo).

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
- A tradução de verdade do #30 (`fix/30-exchange-in-current-block` `fdd93ea`): os testes na placa dão "device removed". Não levar ao jogo.
- O efeito dos dois consertos do mgrein em Series X, e Series X nesta máquina (o modo automático só o escolhe com 48 GB de RAM).
- As portas do renderer na NVIDIA (`VICTORY_PORT_*`): um fork mediu ganho numa RTX 4070, outro mediu perda numa RTX 5070.
- O ray tracing do jogo em placa de grupos de 32: nunca houve rodada com `VICTORY_BVH_BUILDER=1` aqui.

## Próximos passos, por ordem
Os passos 1 a 3 usam o medidor e os comparadores, que ainda são scripts locais (ver o fim deste arquivo).

1. Comparação visual com giro de câmera, cada uma contra a base: `VICTORY_WAVE_WIDE=0` (retângulos e árvores), `VICTORY_TEXTURE_NO_WAIT=0` e `VICTORY_UPLOAD_AHEAD=0` (braço e escada pretos), guarda de textura no padrão.
2. Se o item 1 confirmar a forma wide: portar a forma paired do fxceless atrás de chave (ver `references.md`), com o defrag fora.
3. FPS por configuração, no mesmo ponto fixo, 3 rodadas: portas ligadas, `VICTORY_CARD_AWAKE` desligada, `VICTORY_PIPELINE_BOOK=0`, juntas e separadas.
4. FPS por código: `a2cf68b` do mgrein (limpeza de listas fora da thread de envio), depois medir quanto do preparo é criação de views antes de portar a pilha de cópias de views.
5. Provar `fdd93ea` na placa (o harness precisa ligar o root UAV `u252`, `SLOT_MEET`, o contador dos encontros) e decidir entre estender os half-meets e a forma paired para os 36 programas.
6. Ray tracing, passo 0, sem jogo: traduzir o construtor com `VICTORY_HOST_MAX_WAVE=32` e listar o que recusa (`findings/2026-11-27-ray-tracing-em-grupos-de-32.md`). O caminho da DLL do construtor, que fica na pasta do jogo, é o usuário quem fornece.
7. Memória: `VICTORY_TEXTURE_BUDGET_MB=4096`, `VICTORY_WINDOW_BUDGET_MB=1024` com `VICTORY_WINDOW_IDLE_MS=10000`, `VICTORY_UPLOAD_AHEAD=0`.

## Branches fora deste conjunto
Publicados como PR em rascunho quando passarem nos testes; até lá existem só na máquina das medições.

| Branch | O que é | Por que ficou fora |
|---|---|---|
| `fix/30-exchange-in-current-block` | Tradução de verdade do #30 (`fdd93ea`) | Não provada na placa |
| `diag/visual-counters` | Contadores de diagnóstico | Trabalho sem commit |
| `fix/26-gpu-test-isolation` | Testes de GPU intermitentes (#26) | Pausada |
| `diag/issue-2-memory` | Diagnóstico de memória (PR #19) | Antigo; rebasear ou fechar |

## Ferramentas que ainda não estão no repositório
O medidor (uma rodada, matriz de 3 rodadas em rodízio), o comparador de imagens por blocos e o comparador do giro de 360 graus são scripts Python locais. O método está em `findings/2026-11-27-medicao-e-comparacao-de-imagem.md`. Trazer para `scripts/` é uma lacuna aberta.
