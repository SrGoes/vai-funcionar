# Relatório de ciclo — <data BRT> — <branch>

**Alvo:** ALVO_A (20 FPS reais) | **Issue:** #<n> | **Máquina:** GPU / VRAM / driver / CPU / RAM / pagefile

## Cena
save/ponto · duração · VICTORY_OUTPUT · VICTORY_RENDER_SCALE · --console · build (diag/release) · switches efetivos (linha do log) · cache frio/quente

## Hipótese e evidência
- Gargalo dominante: … (fonte: log/painel/Nsight, run id)
- Hipótese: …

## Mudança
- Arquivos lidos: …
- Arquivos alterados: … (diff resumido)
- Flag: `VICTORY_…` (padrão: off)

## Comandos (com exit code)
| comando | exit |
|---|---|

## Testes
passaram X · falharam Y · pulados Z (listar motivo) · policy ok?

## Resultados (≥ 3 runs cada)
| métrica | baseline | candidato | Δ |
|---|---|---|---|
| FPS quadros novos (média) | | | |
| frametime mediana / p95 / p99 (ms) | | | |
| 1% low (método: …) | | | |
| RAM pico / VRAM pico | | | |
| thread de preparo (ms) / GPU ocupada (ms) | | | |
| crashes / minutos sem falha | | | |

## Regressões / efeitos visuais
…

## Decisão
MANTER / REVERTER — motivo.

## Próximo gargalo
…
