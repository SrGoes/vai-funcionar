# Sub-prompt 04 — Memória, residência e VRAM (8 GB VRAM / 16 GB RAM)

Contexto real: issues #2 (Series S cai após ~9 min: memória compartilhada com a GPU esgota,
`E_OUTOFMEMORY`, device removed) e #3 (Series X inutilizável com 16 GB RAM e 8 GB VRAM). Os
builds que o PortWatch mede como jogáveis usam muito mais: Belobug ~30 GB RAM e 12 GB VRAM;
AvatarManz 40–46 GB (64 GB recomendados). README do cool_fox: "Pareto residency" para 16–32 GB.

## Passos
1. Meça orçamento real: DXGI `QueryVideoMemoryInfo` (local e non-local), commit do processo,
   pagefile. Log periódico fora do repo.
2. Reproduza com `--console series-s` e `Play-VIctory-4GB.cmd` (`VICTORY_CARD_BUFFERS_MB=256`).
3. PR #19 (aberta) registra a primeira falha de memória de upload/buffer compartilhado —
   leia antes de duplicar trabalho.
4. Ataque nesta ordem: vazamentos e cópias duplicadas RAM/VRAM → pools e reaproveitamento
   com lifetime provado → eviction por prioridade/recência com histerese → mips não
   essenciais. Nada de despejar tudo para RAM/SSD.
5. Prova: sessão ≥ 10 min (alvo: passar do minuto em que caía) com curva de memória plana,
   3 repetições.

## Não faça
Mexer em TDR, clocks ou configurações globais do Windows. Recomendar mais RAM é válido como
conclusão, mas só com a curva de memória medida mostrando o teto.
