# Sub-prompt 02 — Tradução de shaders RDNA2 → DXIL (NVIDIA)

Objetivo: reduzir shaders não traduzidos ou incorretos, com prova de equivalência, sem
placeholder que "faz renderizar".

## Contexto do repo
- Pipeline: `victory_shader::ir::build` (SSA, 1 instrução por instrução de máquina) →
  `structure` (remove EXEC; máscaras viram booleanos por lane) → `resource` (rastreia
  descritores até o root parameter) → DXIL.
- `victory translate <arquivos>` lista o que não traduz; `victory shaders` preenche o cache.
- CLAUDE.md registra: 25.756 ok, 229 sem tradução, 26 programas wave64 sem conversão para
  wave32. Issue #11: 4 compute wave64 sem tradução na NVIDIA (`vfx_triangleClip`,
  `water_shading`).

## Regras
- GPUs NVIDIA executam warps de 32 lanes. Um programa escrito para wave64 precisa de
  lowering que preserve a semântica de operações cross-lane (ballot, readlane, DPP,
  reduções, `v_readfirstlane`), máscaras de 64 bits e LDS. Não force `[WaveSize(64)]` sem
  checar `WaveLaneCountMin/Max` do device.
- Valide todo DXIL gerado com o validador do DXC (`dxv`/`IDxcValidator`) e, se houver
  backend Vulkan, SPIR-V com `spirv-val`.
- Equivalência: para cada correção, um teste com entrada sintética gerada por nós que
  compare o resultado do shader traduzido com um interpretador de referência na CPU (o
  americas-recomp e o Xbox2Windows relatam esse método). Nenhum dado do jogo no teste.
- Siga a convenção de censo do gta6-pc-port-hub (`docs/shader-census.md`): diga o
  denominador (quantos programas, de onde) antes de dizer "X% traduzido".
- Shader essencial não pode virar placeholder. Use `VICTORY_SKIP_PIPELINES` só para
  diagnóstico, nunca como correção.

## Passos
1. Rode `victory translate` e classifique as falhas por causa (instrução não suportada,
   wave64, recurso não rastreado, root signature ausente — ver C-3 do port-hub).
2. Escolha a classe com mais impacto em tempo de GPU ou correção visível.
3. Escreva o teste de equivalência primeiro (vermelho), implemente, deixe verde.
4. Meça custo: tempo de compilação/PSO, tempo do pass (Nsight Graphics ou replay).
5. Relatório: classe, contagem antes/depois (mesmo denominador), testes, custo.

## Referências
RDNA2 ISA (AMD), DXIL.rst e HLSL SM 6.6 (Microsoft), dxil-spirv e vkd3d-proton (exemplos de
tradução real), Mesa ACO/RADV (semântica de instruções AMD), `docs/conformance.md` do port-hub.
