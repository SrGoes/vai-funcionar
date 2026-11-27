# Passagem de bastão (estado em 27/11/2026)

## Em uma frase
O port abre e chega ao mundo em placa NVIDIA no Windows 10, a 8-12 FPS e sem o roxo, mas está abaixo dos dois pisos de "jogável": falta FPS, a água tem retângulos, a vegetação pisca ao andar e a iluminação do jogo não roda.

## Onde foi medido
| | Máquina das medições de 26 e 27/11 |
|---|---|
| Placa | GeForce RTX 4080 16 GB, driver 621.24, Resizable BAR ligado (BAR1 de 16 GB) |
| Processador | Ryzen 7 9800X3D |
| RAM | 32 GB (31 GB visíveis) |
| Sistema | Windows 10 Pro 19045 |
| Rota e perfil | rota nvidia, perfil de console Series S, cache normal |

A máquina descrita no `CLAUDE.md` é outra (RTX 5060 8 GB, 16 GB de RAM). Número de uma não vale para a outra.

## O que este conjunto traz para a main
| Peça | Commits | Estado |
|---|---|---|
| Reaproveitamento das cópias de views (conserto da regressão de 12 para 3 FPS, #29) | PR #32 | Medido: 3,35 FPS sem, 12,49 com, 3 rodadas no mesmo ponto |
| Agility SDK para o Windows 10 (`cargo xtask agility`) | PR #31 | Sem ele o port cai no primeiro quadro no Windows 10 |
| Conserto mínimo do pânico do tradutor (#30) | `f0e268b` | `victory shaders` termina: 25.657 traduzidos, 232 sem tradução |
| Filtro de sondas magenta na luz substituta (do Belobug `e942896`) | `64b9e8c` | NÃO resolveu o roxo no modo estimado; não age com `VICTORY_AMBIENT=constant`. Pode ser revertido sem efeito na rota nvidia |
| Reserva de tabelas dinâmicas em heap limitado (do mgrein `d4b8094`) | `69c50a0` | Sem efeito medido em Series S (0 tabelas recusadas); vale para Series X |
| Dispatch indireto largo na NVIDIA (do mgrein `b2f561e`, `8f2ac66`) | `019b8ba`, `eeba481` | Só dispara em 2560x1440 (Series X); não medido aqui |
| Captura por tempo e eixo do analógico em `VICTORY_KEYS` (só no build de diagnóstico) | `b5b2513` | Testes unitários ok; giro de câmera no jogo ainda não provado |
| Launcher com rota explícita, norte, glossário, `docs/agents` | PR #33 | Dois revisores aprovaram; os dois atalhos abrem o jogo |
| Esta passagem de bastão | este PR | |

## O que está provado aqui
- **Regressão #29:** o commit `8977bb0` manda toda view estruturada fora de 16 bytes para `shadow::exact()`, cujo cache é por endereço; o jogo muda o endereço todo quadro, e a NVIDIA (sem slot gêmeo) criava ~600 buffers por quadro. O reaproveitamento resolve. Detalhe em `findings/2026-11-27-regressao-28-e-reaproveitamento.md`.
- **Roxo:** vem da luz substituta em modo estimado. `VICTORY_AMBIENT=constant:0.8` (o valor da rota nvidia) tira. O preço é a imagem chapada.
- **O que falta em cada quadro (log do build de diagnóstico):** ~900-940 execuções "shades ray-tracing hits without BVH" (a iluminação do jogo), 258-283 estágios do construtor de ray tracing, e 1 por quadro de cada compute sem tradução: `8b437ab61d8008fe`, `477b52e73ddeb6f2`, `17af85f170e66f30` e o defrag `02384a60fda6f2d0`. Quase tudo o que falta é ray tracing.
- **O limite de FPS é o processador:** 74-94 ms de quadro, dos quais 51-57 ms de preparo (cópias 14-22 ms, texturas 11-19 ms) e 12-17 ms de cópia de volta; 34-72 mil chamadas de lista por quadro. Com `VICTORY_CARD_BUFFERS_MIN_KB=64` a placa cai de 92% para 43-54% ocupada e o FPS não sobe.
- **PresentMon (kit antigo, 60 s):** 11,1 FPS, quadro mediano 90 ms, p95 145, p99 181, sem VSync. Release e diagnóstico rendem igual.
- **Memória:** 24-25 GB privados com a rota nvidia, RAM livre caindo abaixo de 1 GB em 32 GB. Vazamento real pequeno, 30-50 MB por minuto.
- **Reserva de tabelas:** até 28.264 tabelas pedidas por quadro e 0 recusadas em Series S. Não é a causa do pisca aqui.

## O que NÃO está provado
- Qualquer ganho de FPS por chave: as rodadas de `VICTORY_THIN=1` e `VICTORY_ASYNC_SUBMIT=1` foram únicas e em cenas diferentes.
- As causas dos defeitos visuais em `gaps.md`: são hipóteses com teste definido.
- A tradução de verdade do #30 (`fix/30-exchange-in-current-block` `fdd93ea`): os testes na placa dão "device removed". Não levar ao jogo.
- O efeito dos dois consertos do mgrein em Series X, e Series X nesta máquina (o modo automático só o escolhe com 48 GB de RAM).
- As portas do renderer na NVIDIA (`VICTORY_PORT_*`): um fork mediu ganho numa RTX 4070, outro mediu perda numa RTX 5070.
- O ray tracing do jogo em placa de grupos de 32: nunca houve rodada com `VICTORY_BVH_BUILDER=1` aqui.

## Próximos passos, por ordem
1. Comparação visual com giro de câmera, cada uma contra a base: `VICTORY_WAVE_WIDE=0` (retângulos e árvores), `VICTORY_TEXTURE_NO_WAIT=0` e `VICTORY_UPLOAD_AHEAD=0` (braço e escada pretos), guarda de textura no padrão.
2. Se o item 1 confirmar a forma wide: portar a forma paired do fxceless atrás de chave (ver `references.md`), com o defrag fora.
3. FPS por configuração, no mesmo ponto fixo, 3 rodadas: portas ligadas, `VICTORY_CARD_AWAKE` desligada, `VICTORY_PIPELINE_BOOK=0`, juntas e separadas.
4. FPS por código: `a2cf68b` do mgrein (limpeza de listas fora da thread de envio), depois medir quanto do preparo é criação de views antes de portar a pilha de cópias de views.
5. Provar `fdd93ea` na placa (o harness precisa ligar o root UAV do contador de esperas) e decidir entre estender os half-meets e a forma paired para os 36 programas.
6. Ray tracing, passo 0, sem jogo: traduzir o construtor com `VICTORY_HOST_MAX_WAVE=32` e listar o que recusa (`findings/2026-11-27-ray-tracing-em-grupos-de-32.md`).
7. Memória: `VICTORY_TEXTURE_BUDGET_MB=4096`, `VICTORY_WINDOW_BUDGET_MB=1024` com `VICTORY_WINDOW_IDLE_MS=10000`, `VICTORY_UPLOAD_AHEAD=0`.

## Branches fora deste conjunto
| Branch | O que é | Por que ficou fora |
|---|---|---|
| `fix/30-exchange-in-current-block` | Tradução de verdade do #30 (`fdd93ea`) | Não provada na placa |
| `diag/visual-counters` | Contadores de diagnóstico | Trabalho sem commit |
| `fix/26-gpu-test-isolation` | Testes de GPU intermitentes (#26) | Pausada |
| `diag/issue-2-memory` | Diagnóstico de memória (PR #19) | Antigo; rebasear ou fechar |

## Ferramentas que ainda não estão no repositório
O medidor (uma rodada, matriz de 3 rodadas em rodízio), o comparador de imagens por blocos e o comparador do giro de 360 graus são scripts Python locais. O método está em `findings/2026-11-27-medicao-e-comparacao-de-imagem.md`. Trazer para `scripts/` é uma lacuna aberta.
