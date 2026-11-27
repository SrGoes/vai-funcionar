---
project: vai-funcionar
type: research
updated: 2026-11-27
---

# Forks e referências avaliados

## CGBJ (privado)
PortWatch: 25 FPS em média, "8 GB system RAM and 16 GB VRAM", imagem "lit, but major effects missing" (0,7), história 80%, raramente cai. O código ficou público por horas em 26/11 como `codec-d/cool_fox`; **a nossa base (DoubleGhosty/cool_fox -> vai-funcionar) é esse retrato**, e o `Play-VIctory.cmd` é a rota NVIDIA deles. Continuaram no privado. O Bruno viu vídeo de multiplayer a 25-30 FPS com imagem quase de console.

## Kelvaroq (stellarr-studio/kelvaroq), GPL-3.0-or-later
- Não está à nossa frente: 108,5 ms de mediana em quadros pesados (~7,6 FPS) numa RTX 5080 a 4K/Series X; uma sessão de terceiros numa 4090 a ~10 FPS. Tabelas de uma passada só; boa parte do README sem medição.
- Patches 0049-0064 derivam do XCL (GPL-2.0-only): só ideias. 0041 traz um `launch.dxil` pré-compilado: nunca usar o blob.
- Úteis: 0041/0042 lançamento de malha por estágio de amplificação (198,3 -> 119,8 ms lá; origem do fxceless #1381) e o contador por tipo de limite indireto (0042, ~40 linhas); 0033 e 0047 (grupos de 32: máscara vazia, ordem das linhas de assinatura, recusar pipeline que precisa de onda mais larga) devolvem draws descartados. Não vale: 0045 (vigia por reserva; aqui são 3,7 ms), 0038, 0039 (precisa de upload heap), 0044 (já temos), 0063.
- **Achado de licença na NOSSA árvore:** `crates/victory-loader/src/console_insn.rs` diz "Ported from XCL (github.com/ivall/xcl, GPL-2.0-only)" e o `README.md:113` afirma "no XCL code is included". Veio do upstream (commits de 25/11). O Bruno vai falar com o Goes no Discord; não abrir issue nem mexer no código.

## Grimkaaah/Victory-Grimkaaah
- Seguro para ler (nada baixa nem executa código; único gancho: `tools/play.ps1` roda `discord-presence.ps1` se existir). É um reenvio de 27/11 com 2 commits; segundo o grokbot do Bruno, cópia do avatarmanz/VIctory-7900XTX com ajustes. Só AMD (RX 7900 XTX, Windows 11, 32 GB + pagefile de 96 GB): 24,3 FPS sem ray tracing em cenas cheias, 13,9 com; processo 30-44 GB.
- Para nós: (1) kit de chaves por testar: `SUBMIT_FRAMES=2`, `SUBMIT_DEPTH=128`, `PORT_CONSTANTS=1`, `PORT_CBV=1`, `PORT_GUARD=1`, `PORT_JOURNAL=1`, `PORT_CONSTANTS_ORDER=1`, `PLACED_ORDER=1` (as PORT_* são arriscadas em NVIDIA de 8 GB, issue #15; na 4080 falta testar); (2) ports pequenos: `VICTORY_REAPER` (mgrein a2cf68b, -2,3 a -3,8 ms), `VICTORY_TEXBACK_FREE` (tira espera de 173 ms na thread de retorno), `VICTORY_FILL_FORWARD` (~190 linhas, espera da placa 19-23 -> 0 ms com ray tracing), `VICTORY_UP_QUEUE` (`host/upq.rs`, 230 linhas, -3,5 ms); `lightprobe.rs` (CSV de exposição para o roxo); `tools/report.py` e o PREPARE (shaders + warm). Ganhos só medidos em AMD.

## Rubenpannen/VIctory-RTX5070 (scout concluído)
- Fork do avatarmanz/VIctory-7900XTX (GPL-3.0), 848 commits, push 26/11. Máquina: Windows 11, Ryzen 9 5900X, 64 GB, RTX 5070 12 GB: quadro de 420 para 96-138 ms (7-9 FPS) em 1080p. Sem problema de segurança; **não portar** `e9472c2` (`VICTORY_UI_PLANE` escreve código na memória do jogo).
- `VICTORY_EARLY_DEPTH` (`e982674`, `emit/lower.rs:1950-1965`): conserto de imagem (vidro), muda a tradução e esfria o cache; sem medição. Não é FPS.
- `VICTORY_SUBMIT_FRAMES=2`: código igual ao nosso (padrão 1). **Só é lido quando o envio assíncrono está ligado** (`VICTORY_THIN=1` ou `VICTORY_ASYNC_SUBMIT=1`; `queue/behind.rs:735-745`); o nosso `Play-VIctory.cmd` põe THIN=0, então a chave não faz nada no kit NVIDIA puro. Medido em RTX 2070 SUPER (avatarmanz PR #379): mediana 179 -> 164 ms, espera nos presents 8,4 -> 0 ms.
- `VICTORY_RELEASE_THREAD` (`a4930c4`, ~65 linhas): solta objetos D3D12 aposentados numa thread própria; ~5% da thread de envio em outra máquina. Imagem não muda.
- "Um grupo por peça": já temos (`CARD_COMPARE` com `numthreads(64)` por peça de 64 KiB, `host.rs:~2442-2532`).
- Outros que não temos: `e4822e1` (~110 linhas, 6 arquivos: `send_copied` acha cópias por faixa; `copy_mapped` devolve bool e tira a releitura depois de cada cópia: mira os ~20 ms de "copies" por quadro; só micro-benchmark); `18ae3bd` `VICTORY_EXACT_POOL` (195 linhas: trava de leitura no caminho de acerto do `exact()`); `b906e30` `VICTORY_TEXTURE_VOLATILE` (~90 linhas: reenvia textura que o jogo reescreve entre gravação e envio; conserta quadros do vídeo voltando no tempo; **hipótese: pode ser a causa comum da pele da cabeça sumindo com THIN e dos quadros repetidos com JOIN_PARTS**).
- Ele REMOVEU os ports 1, 5, 6 e 7 do renderer: na 5070 ficaram mais lentos (138 -> 258/206 ms) ou iguais; `PLACED_ORDER=1` também mais lento (112 -> 132 ms).
- Launcher dele: THIN=1, JOIN_PARTS=8, SUBMIT_FRAMES=2, SPARES_BY_HEADROOM=1, UI_PLANE=1, Series X.

## fxceless `VICTORY_SEGMENTS_ACROSS` (`62a8d51`)
Adaptação da ideia do `VICTORY_INDIRECT_BATCH=2`, que já temos (`draw.rs:488`). Lado da placa, -3 ms lá. **Não portar.** Do fxceless, o único item do lado do processador é `VICTORY_HEAP_WORKER` (`d4a528b`, +542 linhas, 10 arquivos, precisa de `VICTORY_LATE_WAIT`, desligado por padrão): passa o passe de texturas para uma thread própria; README diz +40% de quadros no barco numa RTX 4070 (6,2-6,7 -> 8,2-8,6 FPS). Risco alto (texturas um quadro atrasadas; mexe na ordem do bake de pele).

## Ordem sugerida pelo scout para FPS no processador
1. `ASYNC_SUBMIT=1` + `SUBMIT_FRAMES=2` com THIN=0 (meu T2 testou só ASYNC sem SUBMIT_FRAMES: sem ganho; falta com SUBMIT_FRAMES=2 e no ponto fixo). 2. THIN=1 + SUBMIT_FRAMES=2 só depois de entender a pele da cabeça. 3. Port `e4822e1`. 4. `RELEASE_THREAD`. 5. `TEXTURE_VOLATILE`. 6. `HEAP_WORKER` por último. 7. `EXACT_POOL` só se o log mostrar custo de trava.

## mgrein/victory-nvidia (pista do grokbot, NÃO verificada por nós)
Fork focado em RTX, testado numa 4070. Branches `nv/native-rt` (DXR de verdade), `nv/wave-same`, `nv/translator-fixes`. Interessa para ray tracing e para os shaders em grupos de 64. **Falta scout.**

## fxceless/victory-fork
Fork-base ativo e voltado a NVIDIA (RTX 4070): forma paired, `MESH_LAUNCH` padrão ligado, `SEGMENTS_ACROSS`, bake de pele, `PACKED_LEVELS`. FPS deles é baixo (3-10); o nosso ramo (cool_fox) é o rápido. Ver `2026-11-27-consertos-em-outros-ports.md`.

## Playbook NVIDIA (`.agents/gta6-port-kit/PLAYBOOK-NVIDIA.md`)
Bate com o medido. Úteis: Agility (feito), dados na memória da placa (`CARD_BUFFERS_MIN_KB=64` feito; `CARD_WINDOWS_MB` e `CARD_BUFFERS_MB=4096` por testar), port de descritores (`PORT_DESCRIPTORS=1` por testar), guarda de orçamento não local, `warm --used` (feito). Chaves do Kelvaroq citadas lá (`WATCH_RESERVATION`, `DIRECT_CONSTANTS`) não existem na nossa árvore.
