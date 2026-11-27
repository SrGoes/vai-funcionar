# Sub-prompt 07 — Trazer melhorias de outros forks VIctory

Forks ativos (stars/push em 26/11/2026, ver links.md): Marvisak/VIctory, fxceless/victory-fork,
avatarmanz/VIctory-7900XTX, DexNom/VIctory, Belobug/VIctory-Belobug-AMD,
stellarr-studio/kelvaroq (NVIDIA), uiliannn/VItoriaBr (PT-BR), DoubleGhosty/cool_fox (upstream direto).

## Passos
1. `git remote add <nome> <url>` e `git fetch` — leitura. Não abra PR em repo alheio sem o humano pedir.
2. Licença: GPL-3.0(-or-later) é compatível; XCL é GPL-2.0-only → estudar, não copiar.
3. Cherry-pick com atribuição preservada, um tema por PR.
4. A/B na SUA máquina: ganho medido numa RX 7900 XTX ou RX 9070 XT não vale automaticamente
   para RTX 8 GB (issue #13 prova isso).
5. Documente em `docs/ports/AAAA-MM-DD-<origem>.md`: portado, deixado de fora, por quê, commits.
6. Avise o humano se o fork tiver lixo suspeito (binários, scripts que baixam coisas,
   tokens/cripto) — o PortWatch já marcou repos falsos com malware.
