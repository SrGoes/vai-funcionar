# PortWatch VI — snapshot

Fonte: https://portwatch-vi.xyz/data/tracker.json, `updated` = 2026-11-27T01:34Z (26/11/2026 22:34 BRT). O scout atualiza a cada 3 h.

- 44 esforços rastreados (tracked + watchlist), 22 ferramentas, 30 candidatos, 6 grupos marcados como golpe/malware.
- Histórico do líder (pontuação do site): 25/11 AvatarManz 83,1%; 26/11 e 27/11 (UTC) Belobug 91,3%. Degrau mais alto alcançado: **Playable**. Ninguém terminou a história.
- Escala do site: Loads → Boots → Menu → World → Playable (metade da nota); depois Story, Render, FPS, Stability. Passos parciais valem 0,5.
- Executável: publicado por Cyberleek em 23/11 11:36 BRT (não confirmado como genuíno).
- DMCA: Take-Two 24/11 (Leonida + 3.007 forks).

## Principais esforços (valores do tracker; fps = estimativa do PortWatch)

| Projeto | Repo | Abordagem | GPU testada | ★ | último push (UTC) | fps | story | render | estab. | repo sumiu? |
|---|---|---|---|---|---|---|---|---|---|---|
| Belobug | Belobug/VIctory-Belobug-AMD | Compatibility layer | AMD | 3011 | 2026-11-26 | 22 | 0.8 | 0.8 | 1 | não |
| CGBJ | privado | Compatibility layer | N/D | — | — | 25 | 0.8 | 0.7 | 0.85 | — |
| AvatarManz | avatarmanz/VIctory-7900XTX | Compatibility layer | AMD | 1849 | 2026-11-27 | 16 | 0.5 | 0.7 | 0.9 | não |
| NotNozy | privado | N/D | AMD | — | — | 16 | 0.8 | 0.7 | 0.9 | — |
| DexNom | DexNom/VIctory | Compatibility layer | AMD | 1291 | 2026-11-26 | 3 | 0.5 | 0.85 | 0.7 | não |
| darkworldhm | privado | N/D | NVIDIA | — | — | 15 | 0.8 | 0.7 | 0.7 | — |
| Kelvaroq | stellarr-studio/kelvaroq | Compatibility layer | NVIDIA | 297 | 2026-11-26 | 10 | 0.25 | 0.25 | 0.8 | não |
| hyphennn | fxceless/victory-fork | Compatibility layer | NVIDIA | 3282 | 2026-11-27 | 3 | 0 | 0.7 | unknown | não |
| VIctory | Marvisak/VIctory | Compatibility layer | AMD | 13379 | 2026-11-25 | 1 | 0 | 0 | unknown | não |
| Uilian | uiliannn/VItoriaBr | Compatibility layer | NVIDIA | 230 | 2026-11-26 | 4 | 0.1 | 0.2 | 0.4 | não |
| ZETTROX | Zettrox/ztrx | Compatibility layer | NVIDIA | 1499 | 2026-11-25 | 2 | 0.1 | 0.3 | 0.8 | sim |
| XCL | xcl-dev/xcl | Compatibility layer | AMD/NVIDIA | 8615 | 2026-11-26 | 0 | 0 | 0 | unknown | não |
| TuxBox | brcly/TuxBox | Compatibility layer | NVIDIA | 391 | 2026-11-27 | 0 | 0 | 0 | unknown | não |
| Xbox2Windows | R0Y00/Xbox2Windows | Static recompilation | NVIDIA/AMD/Intel | 1966 | 2026-11-27 | 0 | 0 | 0 | unknown | não |
| Leonida | ItsNotRexed/Leonida | Static recompilation | N/D | 32312 | 2026-11-23 | 0 | 0 | 0 | unknown | sim |
| americas-recomp | realdody/americas-recomp | Relinker + shader recompiler | N/D | 586 | 2026-11-23 | 0 | 0 | 0 | unknown | sim |
| GTAVI-Recomp | JamesCicada/GTAVI-Recomp | Relinker (PS5 route) | N/D | 77 | 2026-11-25 | 0 | 0 | 0 | unknown | não |

Stars/push são os que o tracker gravou; a API do GitHub às 23:40 BRT dava valores um pouco maiores (ex.: Marvisak/VIctory 13.411, Belobug 3.130, cool_fox 699).

## Resumos (texto original do tracker, em inglês)

**Belobug** — Belobug's VIctory fork runs on AMD Radeon and is now very playable: about 22 fps on average with no crashes, using about 30 GB of RAM and 12 GB of video memory. Its lit open world plays through missions, with story progress rated 80%, level with CGBJ, and its picture is rated just below DexNom's. Its README hasn't caught up yet: it still reports 5-7 fps.
  - Bloqueios: Its README still reports 5-7 fps, calls the build "not playable yet" and lists more video memory than a 16 GB card has; Ray tracing is reported working on AMD; the README says enabling its ray-tracing path still ends in device removal; Tested on one AMD PC only; nothing on NVIDIA or Intel yet

**CGBJ** — Private VIctory/XCL build, PortWatch-verified. Owner rates story progress at 80%, rendering 0.7 and stability 0.85. It averages 25 fps with 8 GB system RAM and 16 GB VRAM. Its code was public briefly and is offline now.
  - Bloqueios: Minor visual glitches remain; Gameplay does not sustain 30 fps; 25 fps is the reported average; Code is private, so nobody outside the group can test or reproduce it

**AvatarManz** — AvatarManz's fork of VIctory, tuned for speed on an AMD Radeon RX 7900 XTX. It brings victory-fork's, DexNom's and Belobug's work together on VIctory and adds its own speed-ups: the game's draw calls translated on eight threads of its own, frames prepared on a separate thread, and textures uploaded on their own queue. PortWatch has verified it: GTA VI runs at about 16 fps on average in game, with sound, as far into the story as the leading builds. Its README measures 13-22 fps indoors and on the first boat mission, and a 41-minute session without a crash.
  - Bloqueios: Busy street chases drop to 6-10 fps, with single frames far longer; outdoors on foot runs at about 11-17 fps (its README); Trees, plants and far buildings flicker outdoors in places, dim rooms are too dark without ray tracing, and new textures appear late; Uses 40-46 GB of memory (64 GB of RAM recommended); tested on one PC only, not yet reproduced by others

**NotNozy** — A private build shown on NotNozy's "GTA6 Modding Stream" on Rumble. It looks about the same as AvatarManz's build; the owner rates its story progress at 80%. It runs on an AMD card. Code not public, and what it builds on isn't known.
  - Bloqueios: No measured frame rate, crash record or test PC given: the values follow AvatarManz's build, which it looks about the same as; Code is private, so nobody outside can test or reproduce it

**DexNom** — DexNom Runtime, a fork of VIctory for AMD Radeon, tested on a Radeon RX 7900 XTX. It builds on ZSlayerHQ's victory-world-runtime with its own fixes for shaders, texture streaming and GPU memory. PortWatch has seen it running with ray-traced lighting working, the first AMD build on our board to get it. Its picture is the best-rated on our board, though lighting and materials still have problems, and it plays through missions at about 3-4 fps (about 1 fps with ray tracing on), rarely crashing.
  - Bloqueios: About 1 fps with ray-traced lighting on; 3.5-4 fps indoors in Chapter 1's city without it (its test report); Its README still calls ray-traced lighting experimental, and its test report finds interior lighting wrong (overexposed doorways, dark objects); Tested on one AMD card only; not yet reproduced by others

**darkworldhm** — A build shown on darkworldhm's Rumble stream on 25 Nov, running GTA VI's free roam at about 12-15 fps, swinging between 5 and 24: the highest frame rate on our board. The owner rates its story progress at 80%, and it hasn't crashed so far. The picture looks alright, with some texture or ray-tracing glitches. Reportedly on an NVIDIA RTX 50-series card. It's built on VIctory, but which fork isn't known. Code not public.
  - Bloqueios: Frame rate swings between about 5 and 24 fps; Some texture or ray-tracing glitches; Code is private, so nobody outside can test or reproduce it

**Kelvaroq** — Kelvaroq is a public Rust runtime built on VIctory and victory-fork. It adds rendering patches and measured submission improvements, and runs GTA VI through the prologue and first mission.
  - Bloqueios: CPU submission and synchronization remain the main bottleneck; Missing shader pipelines, 7% dropped draws and VRAM pressure remain; One-session stability evidence and two-platform performance measurements are limited

**hyphennn** — hyphennn's fork of VIctory (formerly victory-gta6), with his own rendering and speed fixes. On his Rumble streams the prologue plays with ray-traced lighting, detailed textures and hair at about 3 fps; the README still lists shadows as black. Self-reported.
  - Bloqueios: About 3 fps on the streams; its README now measures 8-10 fps in the first rooms, with optional switches; The README hasn't caught up with the streams: it says shadows are black and ray tracing lights nothing yet; Tested on one PC (RTX 4070); not reproduced by others yet

**VIctory** — Marvis's Rust layer runs GTA VI natively. The owner reports controlled gameplay is now reachable, though the build remains unstable; the README still says it is not playable and reports about 1 fps in the world.
  - Bloqueios: About 1 fps; ray-traced shadows and indirect light are missing; Tested on one machine (AMD RX 6700 XT); not reproduced by others yet; README still says not playable; the new gameplay report has not yet been reproduced

**Uilian** — Uilian's VItoriaBr, built on hyphennn's victory-fork with his own NVIDIA work and streamed live on YouTube. On stream it reaches Jason's first mission, "Gone Fishing", with poor rendering, up to 4 fps and few crashes.
  - Bloqueios: Rendering is poor; shadowed areas are dark; Up to 4 fps; Its code now matches victory-fork's except for two small changes

**ZETTROX** — ZETTROX's build of VIctory and hyphennn's victory-fork for NVIDIA cards. It plays from Lucia's prologue into Jason's first mission, "Gone Fishing", where PortWatch verified the first kill with a gun in GTA VI on PC. Rendering is still poor and it runs at about 2 fps, but it's playable and almost never crashes.
  - Bloqueios: Rendering is still poor: anything in shadow is black and bright scenes overexpose; Runs at about 2 fps (1-3 in its README); Code published on 25 Nov; not yet reproduced by others

**XCL** — Wine-style layer running game code natively. On its main branch GTA VI gets past its menus into the world for several testers, but the screen stays on a loading spinner and a GPU watchdog ends the run. Unverified.
  - Bloqueios: README: an unmerged NVIDIA branch reaches the world at about 5 fps, with parts of the picture missing; The game's GPU watchdog ends each run 3 to 14 minutes in, on AMD and NVIDIA; Pipeline builds fall behind at world load, and BCPack-compressed textures aren't decoded yet

**TuxBox** — Native-first layer for unpacked Xbox Series games, tested mostly on Linux through Proton. GTA VI reported its own state as "Game Running" in the story intro before crashing after about ten minutes, but no picture past the player-selection prompt is shown. Self-reported.
  - Bloqueios: Images, input and frame times of the story-intro run aren't verified; the crash cause is unknown; 12,087 of 16,868 captured GTA VI shader requests compile; large compute shaders take minutes; Windows support still in development; results not yet reproduced

**Xbox2Windows** — Static recompilation research turning Xbox Series code and shaders into Windows C++ and DXIL, with over 50 contributors. Footage the project shared shows GTA VI's menus, unstable. Its README now reports a modified diagnostic exe playing the intro and setup text; the original exe and the menus remain unverified.
  - Bloqueios: Intro and setup text come from a modified diagnostic exe; the original exe is unverified; Only 865 of 26,503 shaders have output verified by its own tests so far; Its tests cover about a quarter of shader instruction forms

**Leonida** — Made just for GTA VI, by Hookshot's author. Taken offline on Nov 23: its Discord vanished, the GitHub repo was deleted and the creator said he quit. Last result: the exe ran, frames stayed black.
  - Bloqueios: Taken offline: GitHub repo deleted, Discord gone, creator stepped away; Take-Two DMCA (Nov 24) disabled all 3,007 GitHub forks; copies and a frozen mirror remain; Last state: every presented frame was black

**americas-recomp** — Fork of AnyPS5 that tested whether the PS5 port tool's RDNA2 shader recompiler can handle the Xbox game. Taken offline on Nov 23: the repo answers 404, its owner's account is still active, and no DMCA notice is published.
  - Bloqueios: Repository gone from GitHub since Nov 23 (HTTP 404); its owner's account is still active; No frame rendered yet; Vertex layout, blend and descriptor state come from the executable

**GTAVI-Recomp** — Targets the PS5 build through AnyPS5 and will not work from leaked files, so game research is blocked. Work continues on AnyPS5 itself.
  - Bloqueios: 15 tasks blocked until GTA VI files can be researched lawfully

## Ferramentas listadas no Toolkit

| Ferramenta | Repo | Tipo | O que faz | ★ |
|---|---|---|---|---|
| Rage Tools VI | lynxnathan/rage-tools-vi | Archive toolkit | Python RPF9 toolkit: rebuilds archive entries from verified payloads, decodes audio and the script schema. | 19 |
| GTA VI PC port hub | fishingpvalues/gta6-pc-port-hub | Coordination hub | Shared notes for port projects: package inventory, shader census format, common blockers. | 6 |
| SixNG | holdmysocks/SixNG | Asset extractor | CLI and desktop app: models to glTF, textures and map data from your own install. | 118 |
| PercyStar | rpatty-dev/PercyStar | Archive explorer | Fast Rust RPF9 explorer with a Bink2 player and ADAT Opus audio decoder. | 106 |
| rpf9-explorer | breadbbbb/rpf9-explorer | Archive explorer | Windows RPF explorer with model, texture, audio and video previews. Leonida builds on its decoders. | 108 |
| bcpack-rs | Ahmet-Dedeler/bcpack-rs | Texture codec | Clean-room decoder for Xbox BCPack textures. BC1 works; BC7 in progress. | 19 |
| RageForge | nullbrik/RageForge | RE toolkit | Explore and analyse RPF9 / TTPF archives. | 2 (sumiu) |
| ttpf-studio | woreksami/ttpf-studio | Texture tool | View, export and replace .ttpf textures; Bink video playback. | 16 |
| GTA6-Asset-Info | walkthestars/GTA6-Asset-Info | Format notes | Format notes for RPF9, textures, BCPack, models and map data. | 10 |
| awesome-gta6-RE | Ahmet-Dedeler/awesome-gta6-reverse-engineering | Curated list | Running list of open-source GTA VI reverse-engineering tools and layers. | 795 |
| gta6-explorer | HolyRoses/gta6-explorer | Asset explorer | Read-only Python toolkit: browse your packages and extract textures, audio, video and shaders. | 15 |
| 6stars | meetox80/6stars | World viewer | From-scratch macOS engine (Swift, Metal) that renders the map, vegetation and weather from your own archives. | 30 |
| rage9-re | Ahmet-Dedeler/rage9-re | Data decoder | Decodes the script and animation archives (PSO data, animations) with glTF export. | 7 |
| AWC from RPF9 | Albotterin/AWC_from_RPF9_Extractor | Audio extractor | Pulls music streams (AWC) out of RPF9 archives. | 6 |
| 0x52504639 | SentinelInteractive/0x52504639 | Format research | RPF9 format notes, catalogues and tooling. | 11 |
| rpf9-research | JoshRob297/rpf9-research | Format research | Spanish-language reverse engineering of the RPF9 header, table of contents and name table. | 7 |
| Vice Lens | Librett0/vice-lens | Asset explorer | Desktop explorer for the map, models, animations, movies and sound in your own copy. | 14 |
| xseriesrecomp | batatinhafcbatatinha-pixel/xseriesrecomp | Front end | One launcher over bundled snapshots of four tracked projects; untested with any game. | 0 (sumiu) |
| xbox-hle-lab | sub-north/xbox-hle-lab | Test tools | GPU probes, VM scripts and patches for XCL, for testing in a sandboxed VM. | 7 |
| XBOX-Inspector | kry-vex-gta/XBOX-Inspector | Build inspector | Rust desktop tool that inventories an unpacked Xbox Series build read-only and checks RPF9 headers and open data ranges. Russian-language docs. | 8 |
| forge9 | nineforge0/forge9 | Archive toolkit | Rust RPF9 toolkit: inspect, extract and repack archives using keys from your own copy. | 8 |
| OpenVI | TheoBIET/OpenVI | Archive explorer | OpenIV-style desktop explorer for the RPF9 archives of your own install; views models and catalogs data. | 12 |

CSV completos: `portwatch-efforts-2026-11-26.csv`, `portwatch-tools-2026-11-26.csv`.