# Consertos que já existem em outros ports da mesma família (pesquisador, só gh/git de leitura)

Licença: cool_fox, fxceless, avatarmanz, DexNom, flamingo, Dathinel, Belobug, Marvisak são GPL-3.0; Kelvaroq GPL-3.0-or-later; **XCL é GPL-2.0-only: só ideias, nunca código**. Relatos são de terceiros, cada um na sua placa.
- **cool_fox não traz nada novo:** `main` dele é 69718c0, já ancestral da nossa árvore. As pistas estão nos PRs e issues abertos.
- **fxceless/victory-fork** se moveu em 27/11 e tem a **forma "paired"** para grupos de 64 (arquivos `emit/paired.rs`, `paired_fp32.rs`, `wave64_after_loop.rs`, que não temos).
- **Marvisak/VIctory** parado desde 25/11; PR #189 (wave64) aberto.

## 1. Retângulos na água
- cool_fox issue #50 (RTX 5080): aparecem quando `5dc3b25986622949` e `6f225b641c5b0708` (`CS_TransmissionVolumetrics_Tiled` e `_LocalLights`) passam a rodar; sem eles a água fica limpa.
- Na nossa árvore eles estão na lista de programas que liga `scene_kernel` (`crates/victory-shader/src/emit/mod.rs:625-640`; outra lista em `:509`), que força a forma "wave-wide" aproximada (commit 3c1f8bd). Na mesma lista: `CS_Tree_BatchCull`, `_ImposterOnly`, `CS_Visibility`.
- fxceless README (RTX 4070, 27/11): sem manchas quando esses dois não são forçados para a forma wide; com paired + reduções FP32 exatas a água ganha o verde (PR #1419, commits e3307c77, 3ab5c12b). fxceless issue #1401 (RTX 3060 Ti): paired em todos os compute de grupos de 64 tira os tiles pretos, mesmo FPS.
- fxceless PR #834 (+368): dispatch indireto pedindo 85-115 mil grupos com corte em 65.535 deixa tiles velhos. **Descartado para a nossa cena:** maior pedido nosso 1.368.
- Teste por chave: `VICTORY_WAVE_WIDE=0` (tira os seis programas da forma wide; esfria o cache; confunde com as árvores).
- Aviso de custo: numa RTX 4080 SUPER o quadro foi de 1,2 s para 1,8 s quando os programas antes descartados passaram a rodar.

## 2. Vegetação piscando
- Recusa de tabelas de descritores no heap de 1.000.000 (avatarmanz #35; PRs #47, #48, #237): **descartado para a nossa cena** (log: ~14.400 tabelas pedidas, 0 recusadas por quadro).
- Culls de árvore na forma wide (avatarmanz #424, RTX 2070 SUPER: com paired as árvores aparecem; cool_fox #51: `89d08e56` entra em pânico na forma wide). É a suspeita que sobra.

## 3. Os 37 programas em grupos de 64 e o #30
- **cool_fox PR #53** (+188/-12): mesmo conserto mínimo do #30, verificado em RTX 5080 e WARP (640/640). Decide pelo bloco onde a troca é emitida e recusa bloco com terminador. **Não portar o PR #56** (esconde o defeito em `bitcode.rs:872`).
- **fxceless #854** (+5.123; só a parte paired) + **#1419** (+1.084): falhas de compute 39 -> 5 numa RTX 5090; todos os 8 recusados rodam numa 2070 SUPER; 7 -> 0 numa 4080 SUPER. O defrag `02384a60` na forma paired dá grade colorida nos rostos: manter fora.
- Alternativas: Belobug #128 (+1.790, só testes), #366 (+573), #345 (+127); avatarmanz #204 (+115); Marvisak #189 (+5.067, outro desenho: pular).

## 4. Tatuagem, braço cinza, decals pretas
- fxceless #1401: `VICTORY_PACKED_LEVELS=1` (~110 linhas em `draw.rs`/`texture.rs`) para ruído por níveis grandes lendo o vizinho. fxceless #821: texturas enviadas antes da leitura terminar. fxceless #1264 (+488) e #1422 (+42): "bake sources sampled wrong" 30 -> 0; port do Dathinel `5d5dfcb` ("necessário, não suficiente").
- Lista de pulados por padrão na nossa árvore (`pipeline.rs:124-145`, de f4164f8): `CS_ProjDecalCut`, `CS_CullMaskPass`, `CS_LodBlend`, `CS_CalculateFireVolumes`, `PS_Smh`. `VICTORY_HAIR=1` religa todos.
- **Ninguém tem conserto limpo** para tatuagem nem braço cinza.

## 5. Triângulos esticados: ninguém tem. Pistas: Marvisak #1319 (cargas escalares por handle errado), #1567 (`CS_PedStandardSkinning_7` lendo zeros), `CS_LodBlend` pulado.
## 6. Rosto sem superfície: "não resolvido em lugar nenhum" (Dathinel #3). fxceless #923: `PS_PedTex*` falha com "a hardware SGPR input is not supported".
## 7. Ray tracing em NVIDIA: ninguém tem imagem iluminada. avatarmanz #66 (+188) e #142, fxceless 54f3815/3e36c41/1320dfd7 dão os 53 pipelines; o jogo cai 30-40 s depois (fxceless) ou 67 s (Flamingo).
## 8. Draws sem pipeline: fxceless #854 traz `emit/link.rs` (+204): 71 programas de malha acima do limite de 32 KiB relinkados (ped, veículo, objeto).

## FPS
- fxceless #1381 `VICTORY_MESH_LAUNCH` (+973): 397/382 -> 334/293 ms numa RTX 5080 com chaves padrão; neutro com o kit que já tem BATCH. fxceless 3221ac42 `VICTORY_MASKED_PATCH`: não se aplica (nosso log já mostra 0 ms esperando a placa).

## Lista de ports por ordem
1. cool_fox PR #53 (#30) — feito por conta própria (36b998b) e ideia incorporada na tradução de verdade.
2. Forma paired do fxceless (#854 paired + #1419 + e3307c77 + 3ab5c12b), atrás de chave, medindo custo; defrag fora.
3. A/B por chave: `VICTORY_WAVE_WIDE=0`; `VICTORY_HAIR=1` (já usamos).
4. Mesh linking do #854 (`emit/link.rs`). 5. `MESH_LAUNCH`. 6. Bake de pele (#1264 + #1422 ou Dathinel 5d5dfcb) e `PACKED_LEVELS`. 7. Marvisak #1319. 8. Ray tracing por último.
