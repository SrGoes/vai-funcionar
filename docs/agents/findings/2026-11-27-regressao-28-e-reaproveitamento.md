# Regressão do #28 (12 para 3 FPS) e o reaproveitamento (PR #32)

## Causa (confirmada por medição)
- Commit **8977bb0** (+**1f6d319**), cherry-picks do Belobug (b68e65e, ce42cc7): "Keep structured buffer slots raw on hosts without descriptor twins".
- `crates/victory-gpu/src/descriptor/host.rs:1339-1340` (`off_elements`): `needs_raw = stride > 0 && stride % 4 == 0 && !has_raw_twins` manda TODA view estruturada com início fora de 16 bytes para `shadow::exact()`.
- `has_raw_twins` vem de `raw_views() != 0` (`host.rs:1383`), ligado só em `probe_view_limit` (`host.rs:1565-1568`) se o host aceitar heap de 2*2^21 views; a NVIDIA para em 1.000.000 (`PORTABLE_VIEWS`, `host.rs:1511`).
- `crates/victory-gpu/src/shadow.rs:5659` `exact(address, len)`: cache `EXACT` indexado por (endereço do jogo, bytes). O jogo põe essas views em posições de anel que mudam todo quadro, então todo pedido falha no cache e chama `host().shared_buffer()` (`shadow.rs:5686` -> `host.rs:10759`): CreateCommittedResource + resident + Map. ~600 por quadro, 251 mil shared buffers em 4 min (13 mil no build antigo).
- A correção do Belobug é legítima: sem gêmeo, leitura crua por view estruturada dá zeros na NVIDIA (`raw_window`, `host.rs:3988`).

## Conserto: reaproveitamento
- Buffers soltos na evicção vão para uma reserva por largura (`len.next_multiple_of(256)`); no próximo pedido pega-se um da mesma largura e zera-se. `Spares<T>`, `EXACT_SPARE`, teto de 64 MiB, quarentena por submissões (`host::submissions()`). Chave `VICTORY_VIEW_BYTES_REUSE=0` volta ao comportamento antigo. `VICTORY_VIEW_BYTES=0` desliga as cópias (views voltam a ler zeros).
- Commits: 3241593 (original), no branch do PR 93e3824 + aba7302 (ajustes da revisão). Teste `gpu_copies_of_views_at_new_places_take_the_buffers_let_go`: 2.400 pedidos, 512 buffers criados, 1.888 reaproveitados; antes do conserto "2400 buffers made for 2400 copies". `let_go_spare_views()` esvazia a reserva em `host::sweep` sob `residency::pressured()`.
- Revisão independente (Sonnet): "approve with required fixes"; nenhum defeito em lifetime, dados velhos, larguras, locks, teto, chave. Riscos não verificados: `map.retain` a cada miss com 256+ entradas pode virar gargalo; buffers em COMMON.

## Matriz 1 (diag, play.ps1 -Safe, Series S, casa, 3 rodadas, mediana)
| Célula | FPS | ms | shared buffers/s | placa ocupada |
|---|---|---|---|---|
| A: b2184b1 com cópias ligadas (`VICTORY_STRUCTURED_COPIES=1` sobre f07c0e1) | 3,35 | 298,2 | 1.780 | 23% |
| B: reaproveitamento | 12,49 | 80,1 | 41 | 91% |
| B0: reaproveitamento com `REUSE=0` | 3,42 | 292,6 | 1.776 | 24% |
| C: cópias desligadas (alternativa 64d8d3a) | 12,68 | 78,9 | 44,5 | 93% |
Referência anterior: build antigo 81ffe9e 12,0 FPS (83 ms), placa 71%. Arquivos: (arquivo local, fora do repositório)).

## Comparação da árvore (3 configurações, 96 quadros seguidos cada, varanda, câmera parada)
Blocos piscando na faixa da água: padrão 114, `REUSE=0` 152, `VIEW_BYTES=0` 185. Pisca fora da água: 0 nas três. Conclusão: o reaproveitamento não cria pisca; os retângulos da água piscam mesmo com as cópias desligadas.

## Gates
- #31 (80a380c): 2 Clippy = 0; testes 1.007 ok / 36 falhos / 47 ignorados, mesmos nomes da base salvo a troca intermitente da #26; com `xtask agility` os testes de victory-shader e victory-gpu dão 712 ok e 0 falhos.
- #32 (aba7302): 2 Clippy = 0; 1.008 ok / 35 falhos / 47 ignorados, nenhum nome novo.
