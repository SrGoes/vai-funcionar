---
project: vai-funcionar
type: tasks-index
updated: 2026-11-27
---

# ESTADO EM 27/11 03:36: main = 68b888b com os PRs #31 a #35; coordenação encerrada. A lista viva são as issues do GitHub (#2-#15, #26, #30, #36-#44) e `docs/agents/` na main. O que está abaixo é histórico da noite.

# Índice de trabalho (o status canônico está nas issues/PRs do GitHub; o quadro `.worktrees/COORDENACAO.md` foi encerrado)

## Abertos no GitHub (SrGoes/vai-funcionar)
- PR **#31** Agility: **MESCLADO** na main em 27/11 03:23 (ede17c7).
- PR **#32** Reaproveitamento: **MESCLADO** (e1e6757); a issue #29 fechou.
- PR **#34** (rascunho, 03:16) **TUDO JUNTO para a main**: branch `integra/2026-11-27-tudo-junto` (c91f68c, worktree `.worktrees/integra`) = #32 + #31 + #30 mínimo + filtro de magenta + consertos do mgrein + captura + `docs/ports/2026-11-27-mgrein.md` + #33 + `docs/agents/` (handoff, gaps, references, lessons, findings). Falta: suíte, clippy e política no branch (worker, quando o jogo fechar); depois tirar do rascunho e pedir o merge ao Bruno/Goes. Se entrar, #31, #32 e #33 fecham.
- PR **#33** Launcher + documentação: **MESCLADO** (2aa942e = ponta da main em 03:23).
- PR **#19** Diagnóstico da #2 (antigo, 81ffe9e): decidir se rebaseia ou fecha.
- Issue **#29** regressão 12 para 3 FPS (resolve com o #32). Issue **#30** pânico do tradutor: conserto mínimo pronto em `fix/30-half-meet-block` 36b998b, SEM PR ainda.
- Issues antigas: #2/#3 memória, #4/#5 pele e tatuagem, #6 triângulos esticados, #7 decals pretas, #8 personagens brancos, #9 bolhas de luz, #10/#13 desempenho, #11 grupos de 64 sem conversão, #12 FSR4 não carrega, #14 interiores escuros sem ray tracing, #15 cache frio, #22 Actions (o Bruno disse que dá para fazer sem), #26 testes intermitentes (pausada; handoff em `%TEMP%\vai-funcionar-handoff-26-22.md`).

## Fila imediata
1. **Port dos consertos do mgrein/victory-nvidia (worker, 02:47):** A1 `d4b8094`, A2 `b2f561e`+`8f2ac66`, depois tradutor (B1 correções, B2 grupos de 64). Ver `research/2026-11-27-mgrein-e-o-que-falta-no-quadro.md`. Quando o build existir: comparação de imagem pela pi e abrir para o Bruno (avisar antes).
2. **Launcher + documentação: ENTREGUE como PR #33 em rascunho** (27/11 02:54). Pendências: teste real de cada atalho com o jogo livre; `xtask repository_passes_policy`; proposta separada de cabelo ligado por padrão (decisão do Goes).
3. **Matriz 2 redesenhada (pi):** base = kit NVIDIA no build play-melhor de diagnóstico, ponto fixo (Enter único aos 10 s, amostra antes do corte da câmera ociosa), 3 rodadas, com comparação de imagem pelo `ab.py`. Células: THIN=1; ASYNC_SUBMIT=1; TEXTURE_GUARD ligada (padrão) contra 0; UPLOAD_AHEAD=0; CARD_WINDOWS_MB=1024; PORT_DESCRIPTORS=1; SUBMIT_FRAMES=2 + SUBMIT_DEPTH=128; PORT_CONSTANTS=1 (+PORT_CBV, PORT_CONSTANTS_ORDER); release contra diagnóstico. Visuais: WAVE_WIDE=0 (água e árvores; esfria o cache), RT_RESOLVE=1, kit com e sem AMBIENT_AIR.
4. **Memória (worker entregou a análise):** testar por chave `VICTORY_TEXTURE_BUDGET_MB=4096`, `VICTORY_WINDOW_BUDGET_MB=1024` + `VICTORY_WINDOW_IDLE_MS=10000`, `VICTORY_UPLOAD_AHEAD=0`; em código: linha de memória do jogo no log, contadores por tipo em Views e "our own allocations" (vazamento de 30-50 MB/min).
5. **Tradução de verdade do #30 (worker):** compilar e testar `fix-30-exchange`; medir quantos dos 37 passam a traduzir; comparar com portar a forma "paired" do fxceless.
6. **PR do #30 mínimo** (36b998b) depois do gate.
7. **Perfil do processador:** instalar o Windows Performance Toolkit (aprovado se precisar) ou outro caminho; o `wpr` da máquina falha ao salvar.
8. **Scouts:** Rubenpannen/VIctory-RTX5070 concluído (ver `research/2026-11-27-forks-e-referencias.md`: ports pequenos `e4822e1`, `RELEASE_THREAD`, `TEXTURE_VOLATILE`; `SUBMIT_FRAMES=2` só vale com envio assíncrono ligado); **mgrein/victory-nvidia** scout ainda rodando às 02:47; commits principais já lidos por mim (ver o relatório do item 1).
9. **Ports candidatos por ordem** (ver `research/2026-11-27-consertos-em-outros-ports.md`): forma paired (água, árvores, 37 programas), REAPER, TEXBACK_FREE, FILL_FORWARD, UP_QUEUE, MESH_LAUNCH, mesh linking, bake de pele.
10. **Defeitos vistos pelo Bruno no kit NVIDIA:** retângulos na água (menos, mas existem) e um retângulo no céu; braço e escada pretos ao mover; sombras bugando (print 02:22); imagem borrada em vegetação, barba e pelo ao mexer a câmera; um flash vermelho na tela; FPS 8-10.
11. **Decisão pendente do Bruno:** rodar a medição longa sem ele (ele disse "não" às 02:10 porque queria continuar testando; perguntar de novo quando for dormir).

## Fechado nesta sessão
- Regressão do #28 diagnosticada e consertada (PR #32). Agility portado (PR #31). #30 mínimo commitado. `texture.rs` do checkout principal restaurado. Glossário, norte, régua, `docs/agents/` e bloco "Agent skills" no `CLAUDE.md` (locais). PresentMon e samply instalados. Build integrado play-melhor.
