---
project: vai-funcionar
type: memory
updated: 2026-11-27
---

# vai-funcionar: conhecimento estável

## Norte e régua (definidos pelo Bruno em 27/11/2026)
- "Jogável" = dois pisos ao mesmo tempo. **Piso visual:** personagens com textura e cabelo, nada piscando ou sumindo, sem retângulos na água, sem materiais roxos ou pretos. **Piso de desempenho:** casa dos 20-30 FPS, 30 como mínimo a alcançar. **Meta:** imagem de console e 60 FPS.
- Nem "100% igual a 5 FPS" nem "30 FPS com boneco preto" servem. Shader sem tradução é defeito a corrigir; pular é paliativo temporário e registrado. Mudança que melhora um eixo e piora o outro: procurar a versão que mantém os dois; se não existir, fica atrás de chave e a decisão vai ao Bruno com números e imagens. Nunca decidir por limiar automático de FPS.
- Régua: **bloqueia** = personagem sem textura/careca, objeto que pisca ou some, retângulos na água, material roxo/preto, e o FPS. **Degrada** = iluminação pobre sem ray tracing (pele pálida, sem reflexo, "massinha"), névoa estranha. **Tolerável** = só aparece procurando ou some em segundos. Ordem de ataque: personagem, ambiente, atmosfera.
- Vocabulário: `GLOSSARY.md` na raiz do checkout (fora do git). Usar em tudo que o Bruno lê.

## Pessoas e máquinas
- **Bruno (Lag0):** desenvolvedor, não é de jogos/Rust/gráficos. Quer ser guiado com recomendação, não questionário. Este PC: Windows 10 Pro 19045, RTX 4080 16 GB, Ryzen 7 9800X3D (8 núcleos, um só complexo), 31 GB de RAM, pagefile de 57 GB em S:. Jogo em `S:\Games\GTA6.Xbox.v1.0.2.1-CYBERLEEK`. DualSense não é reconhecido (o port só lê XInput); teclado: Enter = botão A.
- **Goes (SrGoes):** dono do repo. RTX 5060 8 GB, i5-13400F, 16 GB de RAM, Windows 11. O `CLAUDE.md` do repo descreve a máquina DELE, não esta.
- O `kit` em `.agents/gta6-port-kit/` avisa (LEGAL-E-ESCOPO.md) que o build do jogo é vazado e que a Take-Two já derrubou forks por DMCA. O Bruno sabe; eu trabalho só no código GPL do port.

## Como rodar (aprendido a duras penas)
- **Windows 10 precisa do Agility SDK:** `cargo run --locked -p xtask -- agility` antes do build (só existe em `port/agility-sdk`/PR #31 e nos builds derivados). Sem ele o jogo cai no primeiro quadro (0xc0000005 em `victory-image…+0xb5a5f8`, aviso "root signature flags c00").
- **Dois launchers no repo:** `scripts/play.ps1` = rota AMD (põe `VICTORY_AMBIENT=0.8` estimado, DRED light, e `-Safe` desliga caminhos novos). `Play-VIctory.cmd` = rota NVIDIA do grupo CGBJ (força `--console series-x` e um cache próprio `cache-pareto`). A noite inteira medimos com o primeiro por engano.
- **Kit NVIDIA (o do Play-VIctory.cmd), que usei com Series S e o cache normal:** `VICTORY_AMBIENT_AIR=1.5;VICTORY_TEXTURE_NO_WAIT=1;VICTORY_RELAY=2;VICTORY_WIDE_BACK=1;VICTORY_CARD_REHASH_RUNS=1;VICTORY_JOIN_COPIES=1;VICTORY_INDIRECT_WINDOW=1;VICTORY_RELAY_COPY=1;VICTORY_CARD_STEPS=2;VICTORY_INDIRECT_BATCH=2;VICTORY_MAP_THREADS=1;VICTORY_CARD_AWAKE=1;VICTORY_SHOW_AFTER=1;VICTORY_AMBIENT=constant:0.8;VICTORY_TEXTURE_GUARD=0;VICTORY_THIN=0;VICTORY_JOIN_PARTS=0;VICTORY_PORT_CONSTANTS=0;VICTORY_PORT_CBV=0;VICTORY_LIBRARY_STORE_LATER=1;VICTORY_PIPELINE_WAIT_MS=0;VICTORY_UPLOAD_AHEAD=1;VICTORY_LEVELS_ON_DEMAND=1;VICTORY_CARD_BUFFERS_MIN_KB=64` mais os nossos `VICTORY_HAIR=1;VICTORY_PIPELINE_BOOK=1;VICTORY_KEYS=10:Enter`.
- **Abrir pelo meu launcher exato:** `%LOCALAPPDATA%\victory-tools\run-kit.ps1 -Root <worktree> -Build release|diag -Dir <pasta de log> -Console series-s -Vars "A=1;B=2"` (limpa as VICTORY_* e põe só as dadas). `jogar.ps1`/`jogar.cmd` na mesma pasta ainda usam o kit antigo do play.ps1: ATUALIZAR.
- **Enter:** o jogo espera um Enter no seletor de conta (pronto aos ~8 s); mundo = primeiro Enter + ~37 s. Usar `VICTORY_KEYS=10:Enter` e nenhum Enter depois (Enter no mundo é o botão A e mexe o personagem). O Bruno quer o Enter sempre injetado ao abrir.
- **Pré-aquecimento:** `victory.exe shaders <shaders.rpf> --jobs 8` e depois `victory.exe warm --used`. A chave do cache de shaders está em `shader_cache.rs` (~65-79) e inclui algumas `VICTORY_*` do tradutor (por exemplo `VICTORY_WAVE_LOCAL`; `VICTORY_WAVE_WIDE=0` também esfria): mudar uma delas faz o jogo traduzir tudo de novo. A pi conferiu no código que `VICTORY_DEFRAG` e `VICTORY_PORT_CBV` NÃO entram na chave, e o kit NVIDIA inteiro não esfria o cache (T1: 3.753 hits, 0 misses). Chaves do lado da placa (AMBIENT, CARD_*, TEXTURE_GUARD, HAIR) não esfriam.
- `VICTORY_HAIR=1` religa o cabelo e outros programas que o upstream passou a pular (`pipeline.rs:124-142`: também `CS_ProjDecalCut`, `CS_CullMaskPass`, `CS_LodBlend`). O teste é `is_some()`: qualquer valor liga.
- Cache: `%LOCALAPPDATA%\victory\cache`. Saves: `%LOCALAPPDATA%\victory\data`.

## Regras de coordenação
- Uma fila de build e uma fila de jogo nesta máquina. Ninguém compila durante medição de FPS; com o jogo aberto para teste visual só compilar em prioridade Idle e fora da worktree do exe em uso. Registrar no `COORDENACAO.md` antes de compilar, abrir o jogo ou criar issue/PR/branch; a linha JOGO fica no `SESSOES.md`.
- Worktrees só em `.worktrees/`. Não mexer no checkout principal (tem `CLAUDE.md` modificado e arquivos fora do git: `.agents/`, `GLOSSARY.md`, `docs/agents/`, `GTA_VI_NVIDIA_FIRST_PROMPT.txt`).
- Commits: identidade por variáveis `GIT_AUTHOR_*`/`GIT_COMMITTER_*` = "Bruno Reis Lago <92476366+Lag0@users.noreply.github.com>". PRs em português, no modelo da skill `pr` (Summary, Evidence, Merge Danger) e com os termos do glossário. Abrir PR não precisa de pergunta; **merge é do Bruno**.
- Prints e arquivos do jogo nunca vão para o GitHub. Comentários de triagem começam com "> *This was generated by AI during triage.*".
- Perguntas ao Bruno no formato da skill grilling (Qn + recomendação, "sim" aceita). Avisar antes de abrir o jogo, um teste por vez, e não abrir rodadas de comparação na frente dele sem necessidade.

## Fatos técnicos confirmados
- **Regressão do #28 (12 para 3 FPS):** commit 8977bb0 (+1f6d319, do Belobug) manda toda view estruturada fora de 16 bytes para `shadow::exact()`, cujo cache é indexado pelo endereço; o jogo muda o endereço todo quadro, então a NVIDIA (sem slot gêmeo, heap de 1.000.000 de views) criava ~600 buffers por quadro. **Conserto = reaproveitamento** (PR #32): matriz de 3 rodadas, A 3,35 FPS / B 12,49 / B0 3,42 / C 12,68.
- **Pânico do tradutor (#30):** `half_meet_at` reescreve um bloco já fechado (`lower.rs`, caminho `mask_dword` -> `exchange_halves` -> `order_waves`), vindo dos commits cool_fox c36f9f4/ca7e21a. Conserto mínimo 36b998b recusa o caso; `victory shaders` termina: 25.657 traduzidos, 232 sem tradução (195 contêineres sem programa, 37 em grupos de 64 sem conversão).
- **Roxo:** vem da luz substituta em modo estimado (`ambient.rs`, `max(was, found)` mantém o lixo das sondas). `VICTORY_AMBIENT=constant:0.8` tira (visto pelo Bruno). O filtro de magenta portado do Belobug (562e4a5) só trocou o magenta por azul-arroxeado: NÃO resolveu.
- **Placa esperando:** com o kit do play.ps1 a placa fica 92% ocupada puxando ~95 W; com o kit NVIDIA inteiro (que inclui `VICTORY_CARD_BUFFERS_MIN_KB=64`) cai para 43-54%; nenhuma rodada isolou qual chave responde por isso. Mas o FPS não sobe: o limite é o processador (preparo 51-57 ms por quadro, texturas 11-12 ms com a guarda desligada, cópia de volta 12 ms, e as threads do jogo gravando 34-72 mil chamadas de lista por quadro, 390-400 ms somando as threads).
- **PresentMon, build play-melhor release, kit play.ps1:** 11,1 FPS médio, quadro mediano 90 ms, p95 145, p99 181; processador ocupado 100% e placa 98% do quadro; sem VSync. Release e diagnóstico rendem igual (o perfil release já tem LTO thin e 1 codegen unit).
- **Memória:** kit play.ps1 ~20 GB privados; kit NVIDIA ~25 GB com RAM livre caindo a 0,8 GB e aviso "host memory pressure". Sessão de 62 min do build antigo chegou a 31,5 GB. O grupo CGBJ diz rodar com 8 GB de RAM de sistema.
- **Árvore piscando:** não reproduzida na varanda com câmera parada (3 configurações, 96 quadros seguidos cada); só os retângulos da água piscam, e igual nas três. O Bruno vê vegetação piscando ao andar.
- **Build antigo (81ffe9e) não era limpo:** capturas mostram clientes do bar sem roupa, um rosto coberto de ruído e a mesma luminária roxa.
- **Estado da arte público em NVIDIA (PortWatch 26/11):** 10-15 FPS com defeitos (Kelvaroq 10, darkworldhm 12-15, fxceless 3). CGBJ privado: 25 FPS.

## Armadilhas do ambiente
- No REPL, `$_` dentro de um comando PowerShell passado por `$`…`` vira "unsetenv": sempre escrever um `.ps1` e chamar com `-File`.
- `Start-Process` com `/low` esconde o código de saída: conferir "Finished"/"error" na saída.
- `git stash` é compartilhado entre worktrees: não usar stash nu.
- O PC teve 3 desligamentos inesperados em 26/11 (Kernel-Power 41 às 13:34, 14:21, 14:25); `texture.rs` do checkout principal ficou zerado e foi restaurado com `git restore`.
- `%TEMP%` guarda logs, o medidor e os relatórios; um reinício pode limpar. Cópias do essencial em `%LOCALAPPDATA%\victory-tools\arquivo\`.
- `wpr -stop` falha com 0x80010106 nesta máquina; o samply 0.13.1 precisa do `xperf` (não instalado) para gravar, mas lê ETL com `samply import`.

## Mapa de worktrees (27/11 02:30)
| Worktree | Branch / SHA | O que é |
|---|---|---|
| play-melhor | play/melhor-ate-agora 64b9e8c | Build integrado para jogar e medir (release e diag prontos) |
| 29-view-copies-reuse | fix/29-view-copies-reuse aba7302 | PR #32 (reaproveitamento) |
| agility-sdk | destacado em f07c0e1; branch port/agility-sdk 80a380c | PR #31 (Agility); f07c0e1 inclui a alternativa do #29 |
| fix-view-bytes-churn | fix/view-bytes-churn 5803032 | Binário B da matriz 1 |
| 29-structured-view-copies | fix/29-structured-view-copies 64d8d3a | Alternativa descartada (cópias desligadas por padrão) |
| fix-30-half-meet | fix/30-half-meet-block 36b998b | #30 mínimo, sem PR ainda |
| fix-30-exchange | fix/30-exchange-in-current-block | Tradução de verdade do #30, escrita e NÃO compilada |
| fix-ambient-magenta / test-magenta-agility | 562e4a5 / +Agility | Filtro de magenta (não resolveu) |
| diag-visual-counters | diag/visual-counters | Contadores de diagnóstico, sem commit; precisa recompilar |
| 26-gpu-test-isolation | fix/26-gpu-test-isolation b2184b1 + stash@{0} | #26, pausada |
| issue-2-memory-diag | diag/issue-2-memory 81ffe9e | PR #19 antigo |
| gate, pi-gate-29, pi-gate-agility | — | Worktrees de gate |

## Ferramentas (fora do repo)
- `%LOCALAPPDATA%\victory-tools\`: `presentmon\PresentMon-2.6.0-x64.exe` (assinatura Intel), `samply\samply.exe` 0.13.1, `medir.ps1` (PresentMon 60 s + wpr; precisa de administrador), `run-kit.ps1`, `jogar.ps1`/`jogar.cmd`, `arquivo\` (cópias de segurança).
- Medidor da pi: `%TEMP%\victory-bench-pi\` (`bench.py`, `matrix.py`, `run.ps1`). Comparador de imagem: `ab.py` (cópia em `arquivo\`). Uso: `python -I ab.py <rodadaA> <rodadaB> <saida> 264,304,360,400 --mask 460,130,760,440`.
