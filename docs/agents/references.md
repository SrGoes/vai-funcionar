# Forks e referências já lidos (27/11/2026)

Regras para trazer código de outro fork:
- Leia o diff inteiro antes de aplicar, e os `build.rs` e scripts antes de compilar. Nunca use binário pronto.
- `git cherry-pick -x` para manter o autor e a origem; port feito à mão cita a origem na mensagem.
- Registre em `docs/ports/AAAA-MM-DD-fork.md` o que entrou, os conflitos e o que não foi provado.
- Licenças: os forks abaixo são GPL-3.0 (Kelvaroq GPL-3.0-or-later). XCL é GPL-2.0-only: serve de ideia, nunca de código.
- Número de outro fork é afirmação dele, na placa dele. Meça aqui antes de adotar.

## De onde vem a nossa árvore
`Marvisak/VIctory` → `avatarmanz/VIctory-7900XTX` (ancestral comum `0a43566`, 25/11) → `DoubleGhosty/cool_fox` → este repositório. A `main` do cool_fox (`69718c0`) já é ancestral da nossa; o que há de novo lá está em PRs e issues abertos.

## `mgrein/victory-nvidia` (RTX 4070, Windows 11)
- Lido por inteiro o que foi portado; varredura de segurança limpa. 126 commits de um autor, todos de 26/11.
- **Portado:** `d4b8094` (reserva de tabelas em heap limitado), `b2f561e` e `8f2ac66` (dispatch indireto largo). Valem para Series X.
- **Vale medir, só configuração:** portas ligadas (`PORT_CONSTANTS`, `PORT_CBV`, `PORT_DESCRIPTORS`, `PORT_GUARD`, `PORT_JOURNAL`), `VICTORY_CARD_AWAKE` desligada, `VICTORY_PIPELINE_BOOK=0`. A nossa rota nvidia está ao contrário nas três. Lá: 80 para 64 ms numa área pesada, o conjunto.
- **Vale portar:** `a2cf68b` (~90 linhas, risco baixo: limpeza de listas e soltura de objetos fora da thread de envio).
- **Vale portar depois de medir:** a pilha de cópias de views (`982b9f0`, `54ef67f`, `f9bcd8a`, `b1737be`, `c263560` com `fc02789`, `6381d40`; ~1.900 linhas, risco médio-alto, mesma área do #29).
- **Não resolve os nossos 36 programas:** `nv/wave-same` (`f48bacb`) e `nv/translator-fixes` (`3f029ba` a `3cf024b`) miram o construtor de ray tracing (52 de 52 pipelines numa 4070). Úteis quando o ray tracing for o alvo.
- **Ray tracing nativo (`nv/native-rt`):** DXR 1.1 de verdade, ~15 mil linhas vindas de `maren-halv/VIctory-DXR`. Numa 4070: 6,5 FPS com a caminhada emulada, 9,3 com ray queries, ~31 GB de memória no pico; ninguém conferiu a olho se os interiores ficam iluminados.
- **Não se aplica:** `cc94c34` (o nosso `list.rs` já mantém 512 listas e não tem o trim de 4 GiB). **Não trazer:** `PORT_CBV_SLOT`, as guardas de faixas frias, `-SmoothMotion` (geração de quadro).

## `fxceless/victory-fork` (RTX 4070)
- Tem a **forma paired** para grupos de 64 (`emit/paired.rs`, `paired_fp32.rs`, `wave64_after_loop.rs`), que não temos: PR #854 (só a parte paired) e #1419, com `e3307c77` e `3ab5c12b`. Lá: falhas de compute de 39 para 5 numa RTX 5090 e água sem manchas. O defrag `02384a60` na forma paired dá grade colorida nos rostos: manter fora.
- `VICTORY_HEAP_WORKER` (`d4a528b`, +542 linhas): passe de texturas em thread própria; +40% de quadros no barco numa 4070, pelo README. Risco alto (textura um quadro atrasada).
- `VICTORY_MESH_LAUNCH` (#1381), mesh linking (`emit/link.rs` do #854), bake de pele (#1264, #1422), `VICTORY_PACKED_LEVELS` (#1401).
- **Não portar:** `VICTORY_SEGMENTS_ACROSS` (`62a8d51`): já temos a ideia em `VICTORY_INDIRECT_BATCH=2`.

## `Rubenpannen/VIctory-RTX5070` (RTX 5070, Windows 11)
- `e4822e1` (~110 linhas: cópias achadas por faixa, sem releitura depois de cada cópia), `a4930c4` `VICTORY_RELEASE_THREAD` (~65 linhas), `b906e30` `VICTORY_TEXTURE_VOLATILE` (~90 linhas: reenvia textura que o jogo reescreve entre gravação e envio; hipótese de causa comum da pele da cabeça sumindo com `VICTORY_THIN`), `18ae3bd` `VICTORY_EXACT_POOL`, `e982674` `VICTORY_EARLY_DEPTH` (conserto de imagem em vidro).
- Ele removeu as portas 1, 5, 6 e 7 por ficarem mais lentas na 5070.
- **Não portar:** `e9472c2` (`VICTORY_UI_PLANE` escreve código na memória do jogo).
- `VICTORY_SUBMIT_FRAMES=2` já existe aqui e só é lida com `VICTORY_THIN=1` ou `VICTORY_ASYNC_SUBMIT=1`.

## `Belobug/VIctory-Belobug-AMD`
Já portado em parte (`docs/ports/2026-11-26-belobug.md`). Para ray tracing em grupos de 32 faltam `06fe22a` e `431cab5` com os quatro seguintes. Lá, 53 de 53 pipelines numa RTX 5070 Ti e o jogo caiu em segundos.

## `DoubleGhosty/cool_fox`
PR #53: mesmo conserto mínimo do #30, com `Root::Uav` no harness de teste (necessário para provar a tradução de verdade). **Não portar** o PR #56. Issue #50: retângulos na água quando os dois programas de volumetria rodam.

## Outros
- `Grimkaaah/Victory-Grimkaaah`: cópia do avatarmanz com ajustes, só AMD. Ideias: `VICTORY_TEXBACK_FREE`, `VICTORY_FILL_FORWARD`, `VICTORY_UP_QUEUE`.
- `stellarr-studio/kelvaroq`: não está à frente (7 a 10 FPS em placas maiores). Ideias: lançamento de malha por estágio de amplificação; contador por tipo de limite indireto. Parte dos patches deriva de XCL: só ideia. Traz um `launch.dxil` pronto: nunca usar.
- `Marvisak/VIctory`: parado desde 25/11; PR #189 (grupos de 64, outro desenho).
- Ninguém, em nenhum fork lido, tem ray tracing com imagem iluminada em NVIDIA, nem conserto limpo para tatuagem, braço cinza e rosto sem superfície.
