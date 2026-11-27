# GTA VI Port · Kit de agentes e docs (26/11/2026)

Kit para o trabalho no **SrGoes/vai-funcionar** (runtime VIctory/cool_fox, Rust, D3D12, foco GeForce).

## O que tem aqui
| Caminho | Para quê |
|---|---|
| `prompts/00-master-greenline-v2.md` | Prompt mestre (Greenline v2) amarrado ao repo real |
| `prompts/01..09-*.md` | Sub-prompts: perf, shaders, crash, memória, bugs visuais, HLE, port de forks, revisor, upscaler |
| `prompts/greenline-v1-original.md` | Seu prompt original, intacto, para comparar |
| `agents/AGENTS.md` | Proposta de AGENTS.md para o repo (comandos, crates, regras) |
| `templates/` | Relatório de ciclo, tabela de gargalos, CSV de benchmark |
| `research/portwatch-snapshot.md` + CSVs | Estado do PortWatch em 26/11 22:34 BRT |
| `research/vai-funcionar-estado.md` | Onde o seu repo está, com números da RTX 5060 |
| `links.md` | ~130 links verificados, cada um com o porquê |

## Como usar
1. **No repo (PC Windows):** copie `agents/AGENTS.md` por cima do atual *ou* (mais seguro)
   mova o mandato atual para `docs/agents/greenline.md` e use a proposta como AGENTS.md. Copie
   `cursor-rules/vai-funcionar.mdc` para `.cursor/rules/`. Combine com o Goes antes — o repo é dele.
2. **Abrir uma sessão de agente:** cole `prompts/00-master-greenline-v2.md` + o sub-prompt da
   tarefa + "Issue #N". Exemplo: memória → `04`; FPS → `01`; wave64 → `02`.
3. **Revisão:** outra sessão (outro modelo, se der) com `08-checker-independente.md` + o
   relatório de ciclo + o diff. Quem implementou não aprova.
4. **Registro:** cada run vira uma linha em `templates/benchmark.csv` (fora do repo) e um
   parágrafo no `docs/bug-journal.md`.

## Ordem sugerida para a RTX 5060 8 GB / 16 GB
1. #26 (testes intermitentes) e #22 (CI que nunca rodou) — sem isso nenhum "verde" vale.
2. #2/#3 memória (PR #19 já está no caminho) — a sessão morre aos ~9 min.
3. #10/#13 CPU de preparo: o muro das cópias CPU↔GPU (290 MB/quadro na loja).
4. #15 cache frio de pipelines (`victory shaders` + `victory warm --used`).
5. #11 wave64→wave32; depois bugs visuais #4–#9; upscaler #12 por último.
