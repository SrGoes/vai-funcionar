# vai-funcionar: onde está (lido em 26/11/2026 ~23:40 BRT, clone só leitura)

## Identidade
- Repo: **https://github.com/SrGoes/vai-funcionar** (dono: SrGoes / Gabriel Goes). Público,
  fork do GitHub de **DoubleGhosty/cool_fox**. Não existe `Lag0/vai-funcionar`; o Bruno
  (Lag0) contribui por PRs e issues. O Bruno também tem um fork `Lag0/VIctory-Belobug-AMD-NVDIA`
  (de Belobug/VIctory-Belobug-AMD, criado 26/11 21:35 BRT, sem commits próprios até agora).
- Descrição do repo: "Unmodified mirror of the original codec-d/cool_fox (GPL-3.0, commit
  f4b48ad)". Já não é mais "unmodified": está **54 commits à frente e 0 atrás** de
  DoubleGhosty/cool_fox `main` (API compare do GitHub às ~23:40 BRT). O `codec-d/cool_fox`
  original responde 404 (o PortWatch diz que saiu do ar e moveu o CGBJ para "builds privados").
- Linhagem (README): Marvisak/VIctory → fxceless/victory-fork → forks AvatarManz, DexNom,
  Belobug, ZJOU, victory-amd… → codec-d/CGBJ cool_fox → DoubleGhosty/cool_fox → vai-funcionar.
- Licença GPL-3.0-or-later. 770 commits no histórico (vem do VIctory, primeiro commit
  23/11/2026). ~200 mil linhas de Rust em `crates/` + `apps/`, ~1.139 `#[test]`.
- 0 stars, 0 forks. Criado 26/11 09:32 BRT; último push 26/11 23:07 BRT.

## Stack e build
- Rust 1.98.1 fixado, alvo `x86_64-pc-windows-msvc`, edition 2024. Workspace: 9 crates
  (`victory-core`, `-pe`, `-loader`, `-hle`, `-gpu`, `-shader`, `-formats`, `-pipeline`,
  `-game`) + `apps/victory` (CLI) + `xtask` (política de assets e hooks).
- Requisitos: Windows 10/11 x64, AVX2, D3D12 com SM 6.6 e mesh shaders, MSVC + Windows SDK.
- CI: `.github/workflows/ci.yml` (clippy -D warnings x2 + `cargo test`) em windows-latest —
  issue #22 diz que o workflow **nunca executou**.

## Docs de agente que já existem
- `AGENTS.md` = mandato Greenline com alvo **60 FPS** (o anexo do Bruno diz 20 FPS). Não tem
  comandos de build, mapa de crates nem regras específicas do repo.
- `CLAUDE.md` = `@AGENTS.md` + contexto de uma máquina (RTX 5060 8 GB, i5-13400F, 16 GB RAM,
  driver 621.40; caminho `C:\Users\gabri\...` → parece ser a máquina do Gabriel) + tabela das
  ferramentas de arquivos clonadas e regras de código não confiável. Bom.
- `docs/`: architecture, asset-policy, findings (RX 7900 XTX), bug-journal (RTX 5060),
  replay, fork-ray-tracing, README-fxceless, ports/2026-11-26-belobug.

## Medido na RTX 5060 8 GB / 16 GB RAM (branch `test/rtx5060-session`, docs/test-reports/2026-11-26-rtx5060.md)
- 14 runs + sessão de 12 min. Series X: cai em 2–5 min por memória. Series S (auto):
  mediana 144 ms/quadro (~6,9 fps), p90 535 ms, p99 2.310 ms; 101–147 ms em cenas simples,
  360 ms na loja.
- GPU "99% de uso" mas só 49 W de 145 W → suspeita de GPU esperando/ociosa por dentro; precisa
  Nsight para confirmar (não está provado).
- Processo com 19,5 GB privados (máx. 21,7) numa máquina de 16 GB; sessão morre ~9 min de
  mundo por memória compartilhada com a GPU esgotada (`E_OUTOFMEMORY` → `0x887A0005`).
- Na loja: preparo gasta 536 ms copiando 290 MB/quadro; varredura de 3,3 milhões de slots de
  descritor/quadro → mesmo muro "cópia CPU↔GPU" que o Kelvaroq descreve.
- Shaders: 25.756 traduzidos, 229 não, 26 wave64 sem conversão para wave32.

## Issues e PRs (26/11)
Abertas: #2 Series S cai ~9 min (memória), #3 Series X inutilizável 16/8 GB, #4–#9 bugs
visuais, #10 desempenho 6–9 / 1–3 fps, #11 wave64 sem tradução, #12 FSR4 `E_NOTIMPL`, #13
chaves do play.ps1 não ajudam na RTX 5060, #14 interiores escuros sem RT, #15 cache frio
descarta ~1.000 draws/quadro, #22 CI nunca rodou, #26 testes do victory-gpu intermitentes em
paralelo. PR aberta #19 (Lag0): diagnóstico da #2.
Fechadas/mergeadas no dia: #27 (Bruno: reaproveita caixas da fila Late), #28 port do Belobug,
#25/#23 Clippy, #18/#17 testes, #1 fix/black-render.

## Leitura
O repo está bem organizado e honesto (política de assets forte, findings com números,
relatório de teste por máquina). Os maiores problemas para o hardware descrito são
**memória (16 GB RAM)** e **CPU de preparo/sincronização**, não shader nem GPU. Os builds que
o PortWatch mede como jogáveis rodam com 30–46 GB de RAM e 12–24 GB de VRAM.
