# docs/agents: o que cada sessão de agente precisa ler

Esta pasta mantém alinhadas as sessões de agente (Claude ou outro) que trabalham neste repositório em máquinas diferentes. O que está aqui é o que uma sessão nova não consegue tirar do código nem do histórico do git: o estado, o que foi provado, o que falta e de onde vêm as ideias.

## Ordem de leitura
1. `AGENTS.md` (raiz): o norte ("jogável" = piso visual e piso de desempenho juntos) e "Como rodar e medir".
2. `GLOSSARY.md` (raiz): os termos. Use-os em issues, PRs e relatórios.
3. `handoff.md`: onde o trabalho parou, o que está provado e o que não está.
4. `relatos.md`: o que foi visto jogando, numerado.
5. `gaps.md`: as lacunas abertas, com a evidência e o teste que decide cada uma.
6. `references.md`: os forks já lidos, o que vale portar de cada um e o que não vale.
7. `lessons.md`: regras de trabalho que custaram caro para aprender.
8. `findings/`: um arquivo datado por assunto, com os números e os lugares do código.

## Onde cada coisa mora
| O quê | Onde |
|---|---|
| Tarefas e defeitos | GitHub Issues (`issue-tracker.md`, `triage-labels.md`) |
| Vocabulário | `GLOSSARY.md` (`domain.md` diz como usar) |
| Decisões difíceis de desfazer | `docs/adr/` (criada na primeira decisão) |
| Estado e próximos passos | `handoff.md` |
| O que se vê jogando | `relatos.md`, um número por relato |
| Lacunas | `gaps.md`, com o número da issue quando existir |
| Achados com número | `findings/AAAA-MM-DD-assunto.md` |
| Ports de outros forks | `docs/ports/AAAA-MM-DD-fork.md` |
| Hardware de cada máquina | `machines.md` |
| Ferramentas de arquivos do jogo | `vi-tools.md` |

## Regras para manter isto vivo
- Ao terminar uma sessão de trabalho: atualize `handoff.md` e `gaps.md` no mesmo PR do trabalho.
- Todo defeito visto jogando ganha um número em `relatos.md` no mesmo dia, mesmo sem causa conhecida.
- Achado novo com número: arquivo novo em `findings/`, datado. Não reescreva um achado antigo; acrescente uma seção "Atualização" ou "Correção" com a data.
- Separe sempre "medido aqui", "afirmado por outro fork" e "hipótese". Diga em que máquina, perfil de console e rota o número foi medido.
- Nada de caminho local, log bruto, imagem ou arquivo do jogo nesta pasta (ver `docs/asset-policy.md`).
- Nos achados, "main", "worker" e "pi" são os nomes das três sessões que trabalharam em 26 e 27/11/2026: coordenação, código e medição.

## Termos dos achados que não estão no `GLOSSARY.md`
| Termo | O que é |
|---|---|
| Forma wide | Jeito aproximado de traduzir um programa de grupos de 64 para placa de grupos de 32. É forçada para alguns programas; `VICTORY_WAVE_WIDE=0` desliga |
| Forma paired | Jeito do fork fxceless de traduzir os mesmos programas com as duas metades do grupo emparelhadas. Não existe nesta árvore |
| Half-meets | Pontos em que as duas metades de 32 de um grupo de 64 se encontram para trocar dados (`order_waves`, `half_meet_at`) |
| Defrag | O programa `CS_DefragHeap_ComputeRelocations` (`02384a60fda6f2d0`) |
| Construtor de BVH | Os 52 programas do jogo que montam a estrutura do ray tracing dele |
| Views e tabelas de descritores | As referências a texturas e buffers que cada draw usa. A NVIDIA limita o heap a 1.000.000 de views |
| Ponto fixo | O lugar do save onde o jogo carrega e ninguém toca, usado para comparar rodadas |
| Kit antigo | O kit da rota amd (`scripts\play.ps1` com `-Safe`), usado por engano até 27/11 às 02:10 |
| Build antigo | O build `81ffe9e`, anterior ao PR #28 |
| T1, T2, T3, B1, OLD | Rodadas únicas de 27/11: kit nvidia; com envio assíncrono; com `VICTORY_THIN=1`; kit antigo parado; build antigo por 61 min |
| Matriz N | A matriz de medição que usa o kit da rota nvidia como base |

