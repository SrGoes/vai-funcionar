---
project: vai-funcionar
type: research
updated: 2026-11-27
---

# Kit NVIDIA do repo e os testes de chaves desta noite

## O erro de rota
`scripts/play.ps1` (rota AMD) foi o que usamos em todas as medições até 02:10. `Play-VIctory.cmd` é a rota NVIDIA do grupo CGBJ. Diferenças do .cmd: `VICTORY_AMBIENT=constant:0.8` (o .ps1 usa 0.8 estimado), `VICTORY_AMBIENT_AIR=1.5`, `VICTORY_TEXTURE_GUARD=0`, `VICTORY_THIN=0`, `VICTORY_JOIN_PARTS=0`, `VICTORY_UPLOAD_AHEAD=1`, `VICTORY_LEVELS_ON_DEMAND=1`, `VICTORY_PIPELINE_WAIT_MS=0`, `VICTORY_LIBRARY_STORE_LATER=1`, `VICTORY_CARD_BUFFERS_MIN_KB=64`, `VICTORY_PORT_CONSTANTS=0`, `VICTORY_PORT_CBV=0`, `VICTORY_LOG=warn`, cache próprio `%LOCALAPPDATA%\VIctory\cache-pareto` e `--console series-x`. O .ps1 põe ainda `VICTORY_TITLE_PRIORITIES=0`, `VICTORY_D3D12_DRED=light`, `VICTORY_SKIP_PIPELINES=none` e, com `-Safe`, `VICTORY_UNDER_WIDE=0` e `VICTORY_LAND_ECHO=0`.
Comentários do próprio .cmd: constante porque a estimativa "lavava" o quadro ao entrar numa porta clara; guarda de textura desligada porque a pele do braço de um guarda lia ruído; THIN tira a pele da cabeça na NVIDIA e JOIN_PARTS repete quadros do vídeo; Series X porque em placa de 8 GB a Series S perde a malha da cabeça.

## Medidas com o kit do play.ps1 -Safe (build de diagnóstico, casa, 5,9 mil draws)
80,0 ms por quadro (12,5 FPS): preparo 50,2 ms (vigiar escritas 4,9; bytes sob voos 3,8; passos das listas 24,4, dos quais atualizar cópias 14,7; snapshots 4,8; cópias na memória da placa 8,0; envio 2,2), cópia de volta 16,8 ms, descritores 11,8 ms (798 mil slots olhados), placa 92% ocupada, 874 esperas de grupos + 200 comandos indiretos limitados por quadro, 45.861 chamadas de lista em 277 ms somando as threads, 289 draws descartados sem pipeline e 912 indiretos. `card_buffers_min_kb=4096` efetivo.

## PresentMon (play-melhor release, kit play.ps1, 60 s, 659 quadros)
11,08 FPS; MsBetweenPresents mediana 90, p95 145, p99 181, máx 229; MsCPUBusy 89,6; MsGPUBusy 88,3; MsGPUWait 0; 30 quadros acima de 150 ms; SyncInterval 0; modo "Composed: Flip" em 446 quadros. Arquivo: `%TEMP%\victory-perfil-20261127-020515\quadros.csv` (cópia em `arquivo`).

## Testes T1-T3 (play-melhor diag + captura, Series S, uma rodada, o Bruno às vezes andando: cenas diferentes)
| Teste | Chaves | Amostra | Do log |
|---|---|---|---|
| T1 | kit NVIDIA | 8,4 FPS, 119 ms, 9.522 draws | 94,0 ms com 7.766 draws: preparo 51,6 (cópias 20,1 ms/126 MB, texturas 11,0 ms/270 MB), volta 12,3; placa 43-54%; memória 25,6 GB privados, RAM livre 0,8-3,6 GB; cache hits 3.753, misses 0 |
| T2 | kit + `VICTORY_ASYNC_SUBMIT=1` | 7,73 FPS, 129 ms, 11.060 draws | thread nossa trabalhou 75,2 ms; 71.721 chamadas de lista em 403 ms; placa 46%; sem ganho |
| T3 | kit + `VICTORY_THIN=1` | 12,17 FPS, 82 ms, 4.394 draws | 13,8 FPS com 3.782 draws; thread nossa 65,1 ms; threads do jogo esperaram 39,9 ms nos presents; draws descartados subiram a 1.223-1.399 em alguns intervalos; cabeça com pele nos quadros vistos; mais tarde, em cena cheia (11.206 draws), caiu a 4 FPS / 271 ms |
Pastas: `%TEMP%\victory-run-20261127-021359-T1-nvkit-diag`, `…-022045-T2-nvkit-async-diag`, `…-022432-T3-nvkit-thin-diag` (com `cap\`).
**Leitura honesta:** o custo por draw fica em 11-19 µs em todos e varia com a cena; sem ponto fixo não dá para dizer que alguma chave ganhou FPS. O que está provado: o kit NVIDIA melhora a imagem (roxo e névoa somem; o Bruno confirmou) e tira a placa do teto; o limite passa a ser o processador.

## O que o Bruno viu com o kit NVIDIA (02:17-02:22)
Sem textura roxa em persiana e luz; reflexo do sol na água; nuvens melhores; quadrados na água reduzidos mas presentes; FPS 8-10, "bem travado"; um braço e a escada de dentro ficaram pretos ao mover; um flash vermelho na tela; artefatos e falta de nitidez em vegetação, barba e pelo, piores ao mexer a câmera ("parece artefato de DLSS"); sombras bugando (print 02:22: metade direita escura).
Suspeitas ainda não testadas: `VICTORY_UPLOAD_AHEAD=1` para o preto (o Goes tirou do play.ps1 por deixar decals pretas na NVIDIA, commit 8d50b6b); o reconstrutor temporal do jogo a 8-10 FPS para o borrão (o FSR4 do jogo não carrega, issue #12).

## Plano de FPS da imagem que o Bruno trouxe (outro grupo, mesma base cool_fox Series S)
Números deles batem com os nossos (cenas cheias 12-15 FPS, placa 85-92%). Itens: Thin LTO + codegen-units=1 (JÁ temos no `Cargo.toml`); `VICTORY_SEGMENTS_ACROSS` do fxceless (não existe na nossa árvore); `VICTORY_EARLY_DEPTH` e `VICTORY_RELEASE_THREAD` do Rubenpannen/VIctory-RTX5070 (não existem aqui); `VICTORY_SUBMIT_FRAMES=2` (existe: `queue/behind.rs:720`, padrão 1); fixar threads em P-cores (não se aplica ao 9800X3D).
