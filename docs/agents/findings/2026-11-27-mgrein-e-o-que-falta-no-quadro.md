# O que falta em cada quadro e o que o fork mgrein/victory-nvidia conserta (27/11 ~02:45)

Origem: uma nota externa e a leitura direta do fork. Tudo abaixo marcado VERIFICADO foi conferido por mim no nosso código ou nos nossos logs; o resto é afirmação do fork ou da nota externa.

## O que o nosso próprio log diz que falta em cada quadro (VERIFICADO)
- Build de diagnóstico, kit NVIDIA, Series S ((arquivo local, fora do repositório)), linha `dropped N/frame without a host pipeline: <motivo>`:
  - **~900-940 por quadro: `left out: shades ray-tracing hits without BVH`**. É a iluminação do jogo (o ray tracing dele), deixada de fora de propósito porque o construtor de BVH está desligado. Uma constante (`VICTORY_AMBIENT=constant:0.8`) entra no lugar. É por isso que a imagem fica "chapada", com cara de pré-render (visto no jogo: a lâmpada da casa brilha mas não ilumina o deck).
  - 1 por quadro cada: computes sem tradução `02384a60fda6f2d0`, `8b437ab61d8008fe` (erro "a lane mask exchange", é o #30), `477b52e73ddeb6f2`, `17af85f170e66f30`. Rodam uma vez por quadro: são passes de tela inteira. Candidatos aos retângulos (água, céu) e às sombras.
  - 1 por quadro: `skipped shader 39f0e9aeb156a61c` (lista de pulados).
- Build release da mesma noite (`...023509-jogar-kit-nvidia-release`), 75 linhas no mundo: média de 5.995 draws, **301 draws sem pipeline e 901 execuções indiretas descartadas por quadro** (mínimo 178, máximo 999).
  - **Correção (03:05):** esses ~300 draws sem pipeline NÃO são objetos de cena. Pela pesquisa anterior (`2026-11-27-defeitos-visuais.md`): 258-283 são estágios do construtor de ray tracing, 1 é o defrag e 1 a ativação de raios (`39f0e9aeb156a61c` = `CS_ScreenProbes_ActivateRays`). Os ~900 "indirect" são os `CS_RaytracingResolve` deixados de fora. Ou seja: quase tudo o que falta no quadro é ray tracing.
- Tempo: 74 ms de quadro com 57 ms de preparo (cópias 14,5 ms / 95 MB; texturas 15 ms / 326 MB) e 17 ms copiando de volta. A placa espera o processador (bate com o overlay do Bruno: 99% de uso com 100 W).

## Corte dos dispatches indiretos em 65.535 (VERIFICADO no código; NÃO é a causa em Series S)
- `crates/victory-gpu/src/indirect.rs:285`: `if amd && on { 1 << 20 } else { 65_535 }`. O próprio comentário do nosso código descreve o defeito: em 2560x1440 a lista de tiles tem 67.440 grupos e perde os últimos 1.905 (a faixa de baixo da imagem).
- Tiles são de 8x4 pixels. Em Series S (1920x1080) são 64.800 no total: abaixo do corte. **Só dispara em Series X** (2560x1440), que é o perfil que o `Play-VIctory.cmd` usa (linha 57-59: "series-s drops the head/skin meshes on an 8 GB card"). Nós rodamos tudo em Series S.
- Conserto: mgrein `b2f561e` + `8f2ac66` (só `indirect.rs`; testado por ele numa RTX 4070 com o teste `card_runs_wide_indirect_dispatches`). A/B: `VICTORY_INDIRECT_WIDE=0`.

## Heap de views limitado em 1.000.000 (VERIFICADO no log; efeito aqui ainda NÃO medido)
- Nosso log: `host view heap capped at 1000000; views past it read a null slot count=1245184`. E ~34 mil tabelas de draws por quadro (`34266.9 new ones of draws' tables left to the draws`).
- mgrein `d4b8094` (porta DexNom/VIctory PR #68): na NVIDIA a reserva de slots dinâmicos fica em 2^18 e seca todo quadro (13-60 mil tabelas recusadas a cada 5 s numa 5060 Ti, draws descartados: flicker e vegetação aparecendo e sumindo). Com a divisão nova (o jogo fica com os 393.215 slots de baixo, 606.784 viram dinâmicos): zero recusas lá.
- Nós NÃO temos no log a contagem de tabelas recusadas, então não sei se seca aqui. Pedi o contador ao worker. A/B sem recompilar depois do port: `VICTORY_DYNAMIC_SLOTS=262144`.

## Tradutor (afirmações do fork, a conferir pelo worker)
- Correções: `3f029ba`, `2bbad48` (LDS fora do limite), `2056dae` (ds_bpermute), `16fc3d7` (barreira divergente, `VICTORY_GROUP_SYNC` automático), `77f07f0` (máscara de wave64), `3cf024b`. Trazem `nv_fixes_tests.rs`.
- Grupos de 64 em placa de 32: `f48bacb`, `a7be7cf`, `9c4acb6`, `db37d50` (`VICTORY_WAVE_SAME`, `VICTORY_WAVE_LIVE`); README diz 52/52 programas do construtor de BVH com `VICTORY_HOST_WAVE=32`. Pode resolver o mesmo que a tradução do #30 do worker: pedi comparação antes de continuar.

## Portas do renderer (só configuração; evidência em conflito)
- As chaves `VICTORY_PORT_CONSTANTS`, `PORT_CBV`, `PORT_DESCRIPTORS`, `PORT_GUARD`, `PORT_JOURNAL` existem no nosso código. mgrein (4070, 12 GB): 80 -> 67 ms com 1, 2, 5, 6 e 7. Rubenpannen (5070): mais lento (138 -> 258/206 ms) e removeu. Precisa do nosso número: células na matriz da pi. Não ligar `PORT_CBV_SLOT`.

## Ray tracing (afirmação do fork)
- DXR real cobrindo 47 de 64 programas; numa 4070 a área pesada caiu de 15,7 para 9,3 FPS, com 28-29 GB de RAM. Na nossa máquina (31 GB, ~24 GB privados sem RT) a RAM é o risco. É o caminho para a iluminação de verdade; decisão do Bruno com números e imagens quando existir build.

## Resizable BAR (VERIFICADO na máquina)
- `nvidia-smi -q`: `BAR1 Memory Usage Total 16384 MiB` na RTX 4080: **está ligado** (desligado seria 256 MiB).
- `VICTORY_WINDOWS_BAR_MB` existe no nosso código (`shadow.rs:482-507`) e usa `D3D12_HEAP_TYPE_GPU_UPLOAD`; padrão 0. Uma nota externa diz que isso só existe no Windows 11 e que no Windows 10 seria código novo com NVAPI: NÃO conferi. Ganho medido no comentário do código: listas 29 -> 26 ms de placa, quadro 36 -> 32 ms (em outra máquina). Não é o gargalo de hoje (processador).

## Fonte
`github.com/mgrein/victory-nvidia`, GPL-3.0, HEAD `9849f04` (26/11), ancestral comum com o nosso `0a43566`.

## Atualização 02:53: relatório do scout do mgrein (concluído) e decisões
- **Seguro para ler e portar:** os `build.rs` só fazem hash; nenhum binário além do `bcpack-tables.bin` (igual ao nosso); `Cargo.lock` sem crates novos; único script é o `play.ps1`. 126 commits de um autor só ("Grein"), todos em 26/11.
- **Os branches de tradutor NÃO resolvem os nossos 36 programas:** `nv/wave-same` e `nv/translator-fixes` miram o construtor de BVH do ray tracing (52 de 52 pipelines numa 4070). Os motivos de recusa dos nossos programas continuam recusados lá. A árvore deles não tem half-meets nem a forma "paired". A tradução do #30 do worker continua sendo o caminho.
- **Ray tracing nativo (DXR 1.1):** ~15 mil linhas de outro autor (maren-halv/VIctory-DXR). Numa 4070: 153 ms com a caminhada emulada, 108 ms com ray queries (6,5 para 9,3 FPS), pico de ~31 GB de memória privada. O README diz que ninguém conferiu a olho se os interiores ficam iluminados. Numa RX 9070 XT o FPS não mudou (11,6 contra 11,7), porque o limite é o processador.
- **O que vale para o FPS, pela ordem do scout:** (1) só configuração: portas ligadas, `VICTORY_CARD_AWAKE` desligada, `VICTORY_PIPELINE_BOOK=0`; o nosso kit está ao contrário nas três; lá, o conjunto levou uma área pesada de 80 para 64 ms numa 4070. (2) REAPER `a2cf68b` (~90 linhas, risco baixo): na 4070 as resets de lista e a soltura de objetos aposentados eram ~2/7 da thread de envio; sem número de FPS. (3) pilha de cópias de views (`982b9f0`, `54ef67f`, `f9bcd8a`, `b1737be`, `c263560`+`fc02789`, `6381d40`; ~1.900 linhas, risco médio-alto): criar views custava de 1/5 a 1/3 de cada thread de tradução; é a mesma área da regressão do #28.
- **NÃO se aplica a nós:** `cc94c34` (pool de listas em PC de 32 GB): o nosso `list.rs:256` já mantém 512 listas e não tem o trim de 4 GiB (VERIFICADO).
- **Hipótese reforçada para os retângulos na água:** a nossa lista `scene_kernel` força a forma aproximada em seis programas; a árvore do mgrein não força. Teste: `VICTORY_WAVE_WIDE=0` (esfria o cache).
- **Pergunta aberta:** por que o nosso kit força `PORT_CONSTANTS=0` e `PORT_CBV=0` na NVIDIA (defeito visto em placa de 8 GB, segundo a nota externa)?

## Captura com giro de 360 graus (exigência do Bruno, 02:50)
- Hoje não dá: `VICTORY_KEYS` só aperta botões (`gameinput.rs:66`, setas = direcional), sem eixo de stick; a captura salva potências de dois e listas (`capture.rs`), meio tamanho, só no build de diagnóstico.
- Pedido ao worker: eixo em `VICTORY_KEYS` (ex.: `40-52:RX+`) e `VICTORY_CAPTURE_EVERY_MS` (por tempo desde o primeiro quadro do mundo). Tem de entrar nos dois builds comparados.
- Roteiro pedido à pi: mundo + 20 s; rajada parada de ~10 quadros (flicker = diferença entre quadros vizinhos); giro completo em ~12 s com ~24 prints; rajada parada no fim; folha de contato lado a lado.

## Series S contra Series X (VERIFICADO em `console.rs`)
- O perfil diz ao jogo em qual console ele está; o jogo escolhe resolução e orçamentos de memória por ele. Memória do jogo: Series X 13,5 GiB (10 + 3,5), Series S 7.776 MiB. Imagem: 2560x1440 em X (comentário do `indirect.rs`), janela de 1920x1080 em S (nosso log).
- O modo automático só escolhe Series X com 48 GB de RAM e 12 GB de VRAM; o README fala em 40-46 GB privados em X num PC de 64 GB. A máquina do Bruno tem 31 GB e já usa ~24 GB em S: X provavelmente não cabe. Uma rodada vigiada só.

## Atualização 02:55: roteiro final da captura com giro (pi + worker)
- Comparador da pi pronto: (arquivo local, fora do repositório) (`run` e `compare`); casa base e candidato pelo tempo desde o primeiro quadro do mundo (linhas "frame saved" do log). Saídas: folha de contato por ângulo (base | candidato | mapa de diferença), número de pisca por rajada, draws descartados por motivo, `resumo.json`. Testado nas capturas antigas do A/B da árvore (114 blocos, igual a antes).
- Commit de diagnóstico pedido ao worker, quatro peças: eixo do stick em `VICTORY_KEYS` (`RX+` etc.); prefixo `w` para tempo desde o mundo (`w22-w34:RX+`); `VICTORY_CAPTURE_EVERY_MS` (+ `VICTORY_CAPTURE_MOST`); `VICTORY_CAPTURE_BURST_MS="20000:10,45000:10"`. Precisa estar na base e no candidato.
- Roteiro: rajada de 10 quadros aos 20 s; giro de w22 a w34 com 500 ms (~24 prints, um a cada ~15 graus); rajada aos 40 s; fecha aos 45 s.
- Ordem com o jogo livre: (i) teste real dos dois atalhos do PR #33; (ii) comparação base contra candidato; (iii) matriz N.

## Atualização 02:57: tarefa A construída, e por que ela não muda o que o Bruno vê em Series S
- `port/mgrein-nvidia-fixes` `eeba481` = `play/melhor-ate-agora` `64b9e8c` + `69c50a0` (d4b8094) + `019b8ba` (b2f561e) + `eeba481` (8f2ac66), `cherry-pick -x`. Dois conflitos pequenos resolvidos à mão (`HostHeap::new`; `make_from`). Testes novos do `victory-gpu` passam. Exe em `.worktrees\_target-shared\release` e `\diag` (02:54-02:56).
- **VERIFICADO por mim no log da T1 (diagnóstico, kit NVIDIA, Series S):** linha `dynamic slots: N tables asked, N refused` (`copystat.rs:1303`): 72 linhas, até 28.264 tabelas pedidas por quadro, **0 recusadas**. A reserva de tabelas não seca aqui em Series S: a A1 não é a causa da vegetação piscando nesta máquina. A A2 só dispara em 2560x1440. As duas valem para Series X.
- Não provado: se o teste `card_runs_wide_indirect_dispatches` rodou de fato os 2^20 grupos nesta placa (ele retorna em silêncio quando a placa não é larga); efeito das duas em Series X.
- Target compartilhado: ganho pequeno (2º build de outra worktree: 188 s, igual a um build frio de 187 s). Decisão: voltar ao target por worktree.
- **Consequência:** as suspeitas para os defeitos em Series S passam a ser chaves que já existem: `VICTORY_WAVE_WIDE=0` (retângulos), `VICTORY_TEXTURE_NO_WAIT=0` / `VICTORY_UPLOAD_AHEAD=0` (braço e escada pretos), guarda de textura no padrão. Plano em blocos passado à pi (visual antes de FPS).

## Correção 03:05: as duas causas da nota externa já estavam descartadas na nossa pesquisa
- `2026-11-27-consertos-em-outros-ports.md` já dizia: recusa de tabelas no heap de 1.000.000 "descartado para a nossa cena (0 recusadas por quadro)" e corte em 65.535 "descartado para a nossa cena: maior pedido nosso 1.368". Portei mesmo assim por não reler os relatórios depois da compactação. Vale só para Series X.
- Suspeita que sobra para retângulos na água E vegetação piscando: a lista `scene_kernel` (`emit/mod.rs:625-640`) que força a forma wide em seis programas (`CS_TransmissionVolumetrics_Tiled`, `_LocalLights`, `CS_Tree_BatchCull`, `_ImposterOnly`, `CS_Visibility`). Teste: `VICTORY_WAVE_WIDE=0`. Conserto de verdade: forma paired do fxceless (#854 parte paired + #1419).
