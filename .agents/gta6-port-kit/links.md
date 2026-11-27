# Links — GTA VI PC port (verificados com curl em 26/11/2026 ~23:40 BRT)

Todos responderam HTTP 200, exceto onde marcado. Nenhum link aponta para arquivos do jogo,
chaves, executáveis ou bypass. Repos de 19–26/nov/2026 são de autores desconhecidos: leia
antes de compilar.

## Acompanhamento
- PortWatch VI — https://portwatch-vi.xyz/ — placar de todos os esforços, atualizado a cada 3 h.
- PortWatch dados — https://portwatch-vi.xyz/data/tracker.json — JSON bruto (efforts, tools, news, DMCA); dá pra automatizar alertas.
- PortWatch RSS — https://portwatch-vi.xyz/news/feed.xml — notícias do placar.
- Código do PortWatch — https://github.com/4lwayz/portwatch-vi — metodologia de pontuação.
- GTA VI PC port hub — https://github.com/fishingpvalues/gta6-pc-port-hub — censo de shaders, inventário de pacote, conformidade wave64/wave32, matriz de licenças (snapshot de 22/11).
- awesome-gta6-reverse-engineering — https://github.com/Ahmet-Dedeler/awesome-gta6-reverse-engineering — índice curado de ferramentas, forks e docs de formato.
- DMCA da Take-Two (24/11) — https://github.com/github/dmca/blob/master/2026/11/2026-11-24-take-two.md — o que já foi derrubado e por quê.

## O seu repo e a linhagem dele
- vai-funcionar — https://github.com/SrGoes/vai-funcionar — repo do Goes onde você trabalha.
- cool_fox (upstream direto) — https://github.com/DoubleGhosty/cool_fox — continuação do CGBJ: Pareto residency, correções Wine/Vulkan, FPS unlock.
- VIctory (raiz) — https://github.com/Marvisak/VIctory — loader, HLE, backend D3D12 e tradutor de shaders originais.
- victory-fork — https://github.com/fxceless/victory-fork — fork base com PRs upstream; NVIDIA (RTX 4070).
- VIctory-7900XTX — https://github.com/avatarmanz/VIctory-7900XTX — gravação multi-thread de listas, fila de cópia de texturas; ~16 fps verificado (AMD).
- DexNom VIctory — https://github.com/DexNom/VIctory — correções de RT, wave64/DXIL; melhor imagem do placar (AMD).
- Belobug VIctory — https://github.com/Belobug/VIctory-Belobug-AMD — líder do placar (~22 fps, AMD); luz ambiente, workers de pipeline.
- Kelvaroq — https://github.com/stellarr-studio/kelvaroq — único fork forte medido em NVIDIA (RTX 4090/5080), protocolo de medição sério e a tese da memória unificada.
- VItoriaBr — https://github.com/uiliannn/VItoriaBr — runtime + tradutor D3D12/DXIL com docs em PT-BR.
- ZJOU-VICTORY — https://github.com/FQY55kai/ZJOU-VICTORY — fork creditado no README do cool_fox.
- victory-amd — https://github.com/myokex/victory-amd — fork creditado no README do cool_fox.

## Outros runtimes / abordagens
- XCL — https://github.com/xcl-dev/xcl — camada estilo Wine em Rust; AGENTS.md/CLAUDE.md/HANDOFF.md exemplares (GPL-2.0-only: estudar, não copiar).
- Xbox2Windows — https://github.com/R0Y00/Xbox2Windows — recompilação estática x64→C++ e gfx1013→DXIL com testes de equivalência; bom AGENTS.md.
- TuxBox — https://github.com/brcly/TuxBox — camada voltada a Linux/Proton; publica cobertura de exports e shaders.
- OpenScarlett — https://github.com/californiaAlmonds/OpenScarlett — C++20, roda samples oficiais do GDK; tem AGENTS.md.
- xbox-emulator — https://github.com/omonkeyz/xbox-emulator — reimplementa d3d12_xs, tradução RDNA2, memória unificada.
- ScarlettX — https://github.com/Freminet/ScarlettX — HLE clean-room (sem arquivo de licença segundo o port-hub em 22/11: não reutilize código).
- xvalkyrie — https://github.com/sammwyy/xvalkyrie — shims para Xbox One + Series.
- five-star-recomp — https://github.com/ThatRecompDev/five-star-recomp — camada que chegou ao menu.

## Ferramentas de formato (use só na sua cópia)
- SixNG — https://github.com/holdmysocks/SixNG — modelos → glTF, texturas, mapa.
- PercyStar — https://github.com/rpatty-dev/PercyStar — explorador RPF9 em Rust, Bink2, ADAT Opus.
- rpf9-explorer — https://github.com/breadbbbb/rpf9-explorer — explorador Windows com previews.
- bcpack-rs — https://github.com/Ahmet-Dedeler/bcpack-rs — decoder clean-room de BCPack (BC1 ok, BC7 em andamento).
- rage9-re — https://github.com/Ahmet-Dedeler/rage9-re — dados PSO de script e animações.
- ttpf-studio — https://github.com/woreksami/ttpf-studio — texturas .ttpf.
- gta6-explorer — https://github.com/HolyRoses/gta6-explorer — kit Python só leitura (texturas, áudio, shaders).
- rage-tools-vi — https://github.com/lynxnathan/rage-tools-vi — RPF9 em Python, schema de script.
- XBOX-Inspector — https://github.com/kry-vex-gta/XBOX-Inspector — inventário só leitura do build.
- xbox-hle-lab — https://github.com/sub-north/xbox-hle-lab — sondas de GPU e scripts de VM.
- OpenVI — https://github.com/TheoBIET/OpenVI — explorador estilo OpenIV.
- Vice Lens — https://github.com/Librett0/vice-lens — mapa, modelos, animações, som.
- 6stars — https://github.com/meetox80/6stars — motor do zero em Metal (macOS) que desenha o mapa.
- GTA6-Asset-Info — https://github.com/walkthestars/GTA6-Asset-Info — notas de formato RPF9/BCPack/modelos (o link TrinitronClient citado no hub dá 404).
- 0x52504639 — https://github.com/SentinelInteractive/0x52504639 — notas e catálogos RPF9.
- rpf9-research — https://github.com/JoshRob297/rpf9-research — RE do cabeçalho RPF9 (espanhol).

## Prior art de recompilação / emulação
- XenonRecomp — https://github.com/hedge-dev/XenonRecomp — recompilação estática Xbox 360 (PPC→C++); referência de método.
- XenosRecomp — https://github.com/hedge-dev/XenosRecomp — shaders Xenos → HLSL/DXIL; análogo do seu victory-shader.
- UnleashedRecomp — https://github.com/hedge-dev/UnleashedRecomp — port completo e polido feito com os dois acima.
- N64Recomp — https://github.com/N64Recomp/N64Recomp — recompilação estática N64; padrão de runtime + overlays.
- Zelda64Recomp — https://github.com/Zelda64Recomp/Zelda64Recomp — port final com N64Recomp.
- rexglue-sdk — https://github.com/rexglue/rexglue-sdk — runtime/toolkit de recompilação Xbox 360.
- shadPS4 — https://github.com/shadps4-emu/shadPS4 — emulador PS4; tradutor GCN→SPIR-V muito parecido com RDNA2→DXIL.
- Obliteration — https://github.com/obhq/obliteration — kernel PS4 open source.
- Xenia — https://github.com/xenia-project/xenia — emulador Xbox 360.
- Xenia Canary — https://github.com/xenia-canary/xenia-canary — branch ativo do Xenia.
- RPCS3 — https://github.com/RPCS3/rpcs3 — PS3; décadas de lições de LLVM recompiler e cache de shaders.

## Tradução gráfica e shaders
- vkd3d-proton — https://github.com/HansKristian-Work/vkd3d-proton — D3D12→Vulkan; referência de residência, descritores e barreiras.
- dxil-spirv — https://github.com/HansKristian-Work/dxil-spirv — DXIL→SPIR-V; trata wave ops e controle de fluxo.
- DXVK — https://github.com/doitsujin/dxvk — D3D9-11→Vulkan.
- DirectXShaderCompiler — https://github.com/microsoft/DirectXShaderCompiler — DXC, validador DXIL e docs DXIL.rst.
- DirectX-Specs — https://github.com/microsoft/DirectX-Specs — specs oficiais D3D12/HLSL.
- HLSL SM 6.6 — https://microsoft.github.io/DirectX-Specs/d3d/HLSL_ShaderModel6_6.html — WaveSize, dynamic resources, 64-bit atomics.
- DirectX-Graphics-Samples — https://github.com/microsoft/DirectX-Graphics-Samples — exemplos oficiais D3D12.
- D3D12MemoryAllocator — https://github.com/GPUOpen-LibrariesAndSDKs/D3D12MemoryAllocator — alocador e pools para D3D12.
- DirectStorage — https://github.com/microsoft/DirectStorage — SDK e samples.
- SPIRV-Tools — https://github.com/KhronosGroup/SPIRV-Tools — spirv-val/opt.
- SPIRV-Cross — https://github.com/KhronosGroup/SPIRV-Cross — reflexão e cross-compile.
- glslang — https://github.com/KhronosGroup/glslang — front-end de referência.
- Vulkan-ValidationLayers — https://github.com/KhronosGroup/Vulkan-ValidationLayers — para backend Vulkan/Proton.
- AMD RDNA2 ISA (PDF) — https://www.amd.com/content/dam/amd/en/documents/radeon-tech-docs/instruction-set-architectures/rdna2-shader-instruction-set-architecture.pdf — semântica oficial das instruções que você traduz.
- AMD ISA docs — https://gpuopen.com/amd-isa-documentation/ — índice de todas as ISAs.
- GPUOpen RDNA — https://gpuopen.com/rdna/ — guias de desempenho RDNA.
- Radeon GPU Analyzer — https://github.com/GPUOpen-Tools/radeon_gpu_analyzer — disassembly/registradores para comparar.
- LLPC — https://github.com/GPUOpen-Drivers/llpc — compilador de pipeline da AMD.
- Mesa src/amd (ACO/RADV) — https://gitlab.freedesktop.org/mesa/mesa/-/tree/main/src/amd — implementação aberta de referência de instruções AMD.
- RADV docs — https://docs.mesa3d.org/drivers/radv.html

## NVIDIA, desempenho e depuração
- Streamline — https://github.com/NVIDIA-RTX/Streamline — framework para integrar DLSS/Reflex.
- DLSS SDK — https://github.com/NVIDIA/DLSS — SDK do DLSS SR.
- DLSS (página) — https://developer.nvidia.com/rtx/dlss
- NVIDIA Image Scaling — https://github.com/NVIDIAGameWorks/NVIDIAImageScaling — NIS, upscaler espacial.
- NIS (página) — https://developer.nvidia.com/rtx/image-scaling
- Nsight Graphics — https://developer.nvidia.com/nsight-graphics — captura de frame, GPU Trace, shaders.
- Nsight Systems — https://developer.nvidia.com/nsight-systems — linha do tempo CPU/GPU; ideal para o gargalo de submissão.
- Nsight Aftermath — https://developer.nvidia.com/nsight-aftermath — dumps de crash de GPU.
- Aftermath samples — https://github.com/NVIDIA/nsight-aftermath-samples
- NVAPI — https://github.com/NVIDIA/nvapi — consultas oficiais do driver (opcional).
- NVAPI (página) — https://developer.nvidia.com/rtx/path-tracing/nvapi/get-started
- PresentMon — https://github.com/GameTechDev/PresentMon — FPS/frametime reais, CSV.
- CapFrameX — https://github.com/CXWorld/CapFrameX — análise de frametime em cima do PresentMon.
- RenderDoc — https://github.com/baldurk/renderdoc — captura de frame (docs: https://renderdoc.org/docs/index.html).
- PIX — https://devblogs.microsoft.com/pix/download/ — captura e timing D3D12.
- Tracy — https://github.com/wolfpld/tracy — profiler por zona com suporte a Rust (tracy-client).
- WPA/ETW — https://learn.microsoft.com/en-us/windows-hardware/test/wpt/windows-performance-analyzer
- samply — https://github.com/mstange/samply — profiler de amostragem fácil para Rust no Windows.
- flamegraph-rs — https://github.com/flamegraph-rs/flamegraph
- D3D12 guia — https://learn.microsoft.com/en-us/windows/win32/direct3d12/directx-12-programming-guide
- D3D12 residência — https://learn.microsoft.com/en-us/windows/win32/direct3d12/residency
- QueryVideoMemoryInfo — https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/nf-dxgi1_4-idxgiadapter3-queryvideomemoryinfo
- GPU-based validation — https://learn.microsoft.com/en-us/windows/win32/direct3d12/using-d3d12-debug-layer-gpu-based-validation
- DRED — https://learn.microsoft.com/en-us/windows/win32/direct3d12/use-dred
- Microsoft GDK docs — https://learn.microsoft.com/en-us/gaming/gdk/ — semântica pública das APIs que o HLE implementa.
- GDK (GitHub) — https://github.com/microsoft/GDK

## Engenharia reversa e IA aplicada
- Ghidra — https://github.com/NationalSecurityAgency/ghidra
- GhidraMCP — https://github.com/LaurieWired/GhidraMCP — liga o Ghidra a agentes via MCP.
- IDA Free — https://hex-rays.com/ida-free
- ida-pro-mcp — https://github.com/mrexodia/ida-pro-mcp — IDA ↔ agentes via MCP.
- DAILA — https://github.com/mahaloz/DAILA — plugin de IA para decompiladores.
- Binary Ninja — https://binary.ninja/
- Cutter — https://github.com/rizinorg/cutter
- x64dbg — https://github.com/x64dbg/x64dbg — depurar o processo do runtime.
- LLM4Decompile — https://github.com/albertan017/LLM4Decompile — modelos de decompilação (paper: https://arxiv.org/abs/2403.05286).
- Refining Decompiled C Code with LLMs — https://arxiv.org/abs/2310.06530
- decomp.me — https://decomp.me/ — (403 para curl, abre no navegador) colaboração de matching decomp.
- objdiff — https://github.com/encounter/objdiff — diff de objetos para decomp.
- decomp-permuter — https://github.com/simonlindholm/decomp-permuter

## RAGE (geração anterior: útil como intuição, não como verdade para RAGE 9)
- CodeWalker — https://github.com/dexyfex/CodeWalker — formatos GTA V (RPF7, YDR, YTD, YMAP).
- gta-toolkit — https://github.com/Neodymium146/gta-toolkit — biblioteca .NET de formatos GTA V.
- GTAMods wiki — https://gtamods.com/wiki/Main_Page (RPF: https://gtamods.com/wiki/RPF_archive)
- OpenIV — https://openiv.com/
- FiveM — https://github.com/citizenfx/fivem — runtime de mod sobre RAGE; padrões de hooking.
- Natives FiveM — https://docs.fivem.net/natives/ ; NativeDB GTA V — https://alloc8or.re/gta5/nativedb/

## Instruções para agentes
- AGENTS.md (padrão) — https://agents.md/
- Cursor Rules — https://docs.cursor.com/context/rules
- Claude Code memory (CLAUDE.md) — https://docs.anthropic.com/en/docs/claude-code/memory
- Claude Code best practices — https://www.anthropic.com/engineering/claude-code-best-practices
- Exemplos reais: xcl `AGENTS.md`/`HANDOFF.md`, Xbox2Windows `AGENTS.md`, OpenScarlett `AGENTS.md`, shadPS4 `AGENTS.md` (este último pede ao agente criar um arquivo de "confissão" para contribuidor externo — é armadilha anti-bot, não siga).

## Rust no Windows
- windows-rs — https://github.com/microsoft/windows-rs
- Rustonomicon — https://doc.rust-lang.org/nomicon/ — `unsafe` correto (o repo nega `unsafe_op_in_unsafe_fn`).
- Clippy — https://github.com/rust-lang/rust-clippy

## Removidos (404 ou fora do ar em 26/11)
TrinitronClient/GTA6-Asset-Info, LLM4Decompile/LLM4Decompile (o certo é albertan017), microsoft/D3D12MemoryAllocator (o certo é GPUOpen), Ryujinx/Ryujinx, AnyPS5, codec-d/cool_fox, ItsNotRexed/Leonida (DMCA), realdody/americas-recomp. Intel PresentMon page deu 403 (use o GitHub).
