# Sub-prompt 01 — Profiling de desempenho (GeForce)

Use junto com `00-master-greenline-v2.md`. Objetivo: achar o gargalo dominante de UMA cena e
provar uma melhoria, sem mexer em nada que não seja medido.

## Entrada que você precisa (pergunte se faltar)
- Cena (save/ponto), duração (ex.: 300 s como o Kelvaroq usa), resolução de saída, escala,
  `--console`, build (diag/release), driver, GPU/VRAM/RAM.

## Passos
1. **Baseline** com `scripts\play.ps1` (switches padrão) e com `-Safe`. ≥ 3 runs cada.
   Capture com PresentMon (CSV: `MsBetweenPresents`, `MsGPUBusy`/`MsGPUTime` quando
   disponível na versão instalada — anote quais colunas existiam).
2. **Divida o quadro** com `VICTORY_PANEL=1` (F3) e os logs de timing do runtime: tempo do
   título, thread de preparo de submissão, relay, espera por fences, GPU ocupada. Lembre:
   etapas concorrentes não se somam.
3. **Nsight Systems** para a linha do tempo CPU/GPU (threads do título × preparo × fila).
   **Nsight Graphics / GPU Trace** só se a GPU estiver no caminho crítico.
4. **Replay** (`VICTORY_RECORD=<pasta fora do repo>`, `VICTORY_RECORD_AT=f<n>`, depois
   `victory replay`) para iterar mudanças de GPU em segundos sem o título.
5. Monte a **tabela de gargalos** (`templates/tabela-gargalos.md`): evidência, hipótese,
   teste, custo previsto, custo medido, decisão.
6. Implemente a menor mudança, atrás de flag, compare na mesma cena. Mantenha só com ganho
   > ruído (~5%) em ≥ 3 runs.

## Pistas já documentadas (confirme antes de usar)
- Na linhagem VIctory o muro é o preparo de submissão + sincronização de memória
  CPU↔GPU (docs/findings.md; README do Kelvaroq: GPU ~30% ocupada, CPU de submissão limita).
- Issue #10: 6–9 fps em cenas simples, 1–3 em cenas cheias, CPU preparando cópias.
- Issue #13: as chaves rápidas do `play.ps1` não ganham nada na RTX 5060 — meça em vez de
  herdar defaults medidos numa RX 7900 XTX.

## Saída
Relatório de ciclo + CSV bruto + linha nova no `docs/bug-journal.md` com run ids.
