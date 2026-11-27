# docs/agents: o que cada sessão de agente precisa ler

Esta pasta mantém alinhadas as sessões de agente (Claude ou outro) que trabalham neste repositório em máquinas diferentes. O que está aqui é o que uma sessão nova não consegue tirar do código nem do histórico do git: o estado, o que foi provado, o que falta e de onde vêm as ideias.

## Ordem de leitura
1. `AGENTS.md` (raiz): o norte ("jogável" = piso visual e piso de desempenho juntos) e "Como rodar e medir".
2. `GLOSSARY.md` (raiz): os termos. Use-os em issues, PRs e relatórios.
3. `handoff.md`: onde o trabalho parou, o que está provado e o que não está.
4. `gaps.md`: as lacunas abertas, com a evidência e o teste que decide cada uma.
5. `references.md`: os forks já lidos, o que vale portar de cada um e o que não vale.
6. `lessons.md`: regras de trabalho que custaram caro para aprender.
7. `findings/`: um arquivo datado por assunto, com os números e os lugares do código.

## Onde cada coisa mora
| O quê | Onde |
|---|---|
| Tarefas e defeitos | GitHub Issues (`issue-tracker.md`, `triage-labels.md`) |
| Vocabulário | `GLOSSARY.md` (`domain.md` diz como usar) |
| Decisões difíceis de desfazer | `docs/adr/` (criada na primeira decisão) |
| Estado e próximos passos | `handoff.md` |
| Lacunas | `gaps.md`, com o número da issue quando existir |
| Achados com número | `findings/AAAA-MM-DD-assunto.md` |
| Ports de outros forks | `docs/ports/AAAA-MM-DD-fork.md` |

## Regras para manter isto vivo
- Ao terminar uma sessão de trabalho: atualize `handoff.md` e `gaps.md` no mesmo PR do trabalho.
- Achado novo com número: arquivo novo em `findings/`, datado. Não reescreva um achado antigo; acrescente uma seção "Atualização" ou "Correção" com a data.
- Separe sempre "medido aqui", "afirmado por outro fork" e "hipótese". Diga em que máquina, perfil de console e rota o número foi medido.
- Nada de caminho local, log bruto, imagem ou arquivo do jogo nesta pasta (ver `docs/asset-policy.md`).
- Nos achados, "main", "worker" e "pi" são os nomes das três sessões que trabalharam em 26 e 27/11/2026: coordenação, código e medição.
