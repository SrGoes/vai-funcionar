# Leituras que complementam a base do Lago, e inventário dos testes (27/11/2026)

A base de conhecimento do Lago (`docs/agents/kb/`, 11 dossiês e 124 fontes, escrita em 27/11; quando este arquivo foi escrito ela ainda não estava no fork) cobre o que é preciso consultar sobre:
- a ISA da AMD (01, 02);
- DXIL e `WaveSize` (03);
- D3D12 e NVIDIA (04, 05);
- ray tracing (06);
- memória e residência (07);
- medição (08);
- o console (09);
- o loader (10);
- o motor e o upscaling (11).

Este arquivo **não repete** a base. Traz só o que ela não tem: o inventário dos testes, respostas a perguntas abertas do dossiê 07 e três papers ligados aos nossos problemas medidos.

**Rótulos:**
- **(lida):** fonte lida diretamente; os PDFs foram extraídos com um script local.
- **Ligação:** aplicação ao port, ainda não provada.

## 1. Testes que temos

Contagem de `#[test]` por crate, com os ignorados entre parênteses:

| Crate | Testes | O que cobre |
|---|---|---|
| `victory-gpu` | 512 (38) | Lógica do runtime gráfico. Parte cria um dispositivo D3D12 de verdade (placa ou WARP) pelos harnesses de `host.rs`, `indirect.rs` e `inplace.rs`. `tests/replay.rs` e `replay_big.rs` gravam um programa pequeno, reproduzem e conferem imagem e buffers byte a byte |
| `victory-shader` | 306 (4) | Tradutor GCN/RDNA para DXIL; `emit/host.rs` cria o pipeline e desenha um triângulo para conferir na placa |
| `victory-hle` | 156 (2) | Funções do sistema emuladas; `tests/no_dll_planting.rs` |
| `victory-loader` | 57 | Carregamento e patches (resolução, upscaler); `tests/load_probe.rs`, `title_dlls.rs` |
| `victory-formats` | 49 (5) | Formatos de textura e de arquivo |
| `victory-pe` | 36 | Leitor de PE |
| `victory-core`, `apps/victory`, `xtask`, `victory-pipeline`, `victory-game` | 13, 13, 8, 7, 3 | Utilidades, linha de comando, Agility |

**Os ignorados:**
- 12 pedem chaves (`VICTORY_RELAY=2`, `VICTORY_RELAY_COPY=1`...) e um processo só para eles;
- 9 são medições ou benchmarks;
- 1 pede a placa e contadores globais (#26);
- 1 ocupa gigabytes da placa.

**O que falta:** nenhum teste exercita a trava `EXACT` com várias threads, o orçamento NON_LOCAL nem a paginação. Um teste sintético de contenção (N threads pedindo cópias `exact` em posições de anel novas, como faz `gpu_copies_of_views_at_new_places_take_the_buffers_let_go`, mas com threads) mediria a mudança da trava sem abrir o jogo, como o `AGENTS.md` pede quando falta componente.

## 2. Respostas a perguntas abertas do dossiê 07 (memória)

- **Pergunta 2 ("como o sistema calcula o orçamento NON_LOCAL?")** **(lida)**: [Calculating Graphics Memory](https://learn.microsoft.com/en-us/windows-hardware/drivers/display/calculating-graphics-memory) dá `TotalSystemMemoryAvailableForGraphics = MIN(RAM × 80%, MAX(RAM − 16 GB, RAM × 50%))`. A memória compartilhada é esse total menos a memória de sistema dedicada.

  | RAM | Total para a placa |
  |---|---|
  | 16 GB | 8 GB (o DXGI reportou 7.446 MB na máquina B) |
  | 32 GB | 16 GB (máquina A, a conferir com `log_video_memory`) |
  | 64 GB | 48 GB |

  A página dá o total disponível para gráficos. Que esse seja o orçamento NON_LOCAL do `QueryVideoMemoryInfo` é **Inferência**, porque 7.446 MB fica perto de 8 GB menos o reservado. O dossiê alerta, com razão, para não extrapolar a proporção sem conferir.
- **`OpenExistingHeapFromAddress` e `FromFileMapping`** **(lida)**: [página da primeira](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12device3-openexistingheapfromaddress), [página da segunda](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12device3-openexistingheapfromfilemapping).
  - A Microsoft descreve os dois como heap "especial, de diagnóstico", na RAM, que sobrevive a device removed.
  - A versão por file mapping pede Windows 10 build 20348 ou posterior.
  - Nenhuma das páginas fala em alinhamento, `SEC_COMMIT` ou desempenho. As regras práticas são as que o `inplace.rs` mediu.

  **Ligação:** a porta 1 e o `VICTORY_INPLACE` usam uma API cujo propósito declarado não é desempenho. Funciona, mas o risco é de suporte do driver; medir antes de depender disso para tirar as janelas.
- **Rastreamento de escrita** **(lida)**: [PIX e `ID3D12ManualWriteTrackingResource`](https://devblogs.microsoft.com/pix/pix-and-id3d12manualwritetrackingresource/).
  - Ferramentas usam `WRITE_WATCH` em recursos mapeáveis.
  - Até março de 2023, recursos `GPU_UPLOAD` com `WRITE_WATCH` eram forçados para a RAM.

  **Ligação:** conferir antes de juntar janelas na VRAM (`VICTORY_WINDOWS_BAR_MB`, `VICTORY_CARD_WINDOWS_MB`) com qualquer rastreamento de escrita.

## 3. Três papers, três problemas nossos

- **As cópias do port são um sistema de memória compartilhada distribuída** **(lida)**: [TreadMarks, Keleher, Cox, Dwarkadas e Zwaenepoel, USENIX inverno de 1994](https://infoscience.epfl.ch/record/55805/files/keleher94treadmark.pdf).
  - **Mecanismo:**
    - a página fica protegida contra escrita;
    - na primeira escrita, guarda-se um gêmeo (twin) e a escrita é liberada;
    - depois, a página é comparada com o gêmeo para gerar um diff.
  - **Custo:** está em copiar a página e montar o diff; menos de 0,8% fica no kernel (sinal e `mprotect`).
  - **A otimização central** é criar o diff de forma preguiçosa, só quando alguém vai ler.

  **Ligação:** os `before`, snapshots e a cópia de volta (34 ms por quadro na máquina B) são twins e diffs. Vale o mesmo princípio: comparar e copiar só o que alguém vai ler, quando for ler, e não tudo a cada submissão.
- **A trava `EXACT` é um mapa de muitas leituras e poucas escritas** **(lida)**: [Triplett, McKenney e Walpole, "Resizable, Scalable, Concurrent Hash Tables via Relativistic Programming", USENIX ATC 2011](https://www.read.seas.harvard.edu/~kohler/class/aosref/triplett11resizable.pdf).
  - Com RCU, as buscas não esperam nunca e escalam linearmente: 125× melhor que com trava de leitura e escrita durante redimensionamento.
  - Quem escreve copia e espera os leitores terminarem antes de liberar.

  **Ligação:** a `EXACT` recebe 1.500 a 2.200 buscas e 300 a 400 misses por quadro, todas pela mesma trava exclusiva, e cada miss varre o mapa (`findings/2026-11-27-trava-exact-e-paginacao-maquina-b.md`). O desenho que a literatura sustenta, em ordem crescente de custo:
  1. limpeza fora do caminho de quem busca;
  2. criação de buffer fora da trava;
  3. trava fatiada por endereço;
  4. buscas sem trava.
- **Traduzir entre larguras de SIMD** **(lida)**: [Ocelot, Diamos, Kerr e Kesavan, relatório GIT-CERCS-09-01 da Georgia Tech](https://cercs.gatech.edu/www.cercs.gatech.edu/tech-reports/tr2009/git-cercs-09-01.pdf), que traduz PTX da NVIDIA para outras larguras.
  - Barreiras se resolvem com laços que comprimem threads.
  - As operações entre lanes (vote, any/all no warp) dependem da largura e exigem tradução própria.
  - Guardar o original para recompilar quando a largura muda.

  **Ligação:** junta-se ao que o dossiê 01 já traz dos slides da AMD: o compilador do console remove barreiras que considera desnecessárias. Um grupo de 64 threads que é uma única wave64 não precisa de barreira no console; na NVIDIA ele vira duas waves de 32, e as leituras de LDS entre as metades ficam sem a barreira que o binário não tem. É a recusa "LDS in a group over 32" dos nossos 36 programas (#11, #30). O caso difícil é o mesmo do Ocelot: lanes e memória do grupo, não a aritmética.

## 4. Pistas da base do Lago para o plano de memória da máquina B
- **`XMemGetWorkingSetStatistics`** (dossiê 07, `xmem.rs:1910-1950`) diz ao jogo que ele usa o commit do processo inteiro. Na máquina B são ~21 GB, contra o limite de 7.776 MiB do Series S. **Hipótese:** se o jogo dimensiona o carregamento por isso, ele se acha muito acima do limite. Vale um log do que o jogo pede e decide depois dessa chamada.
- **`VICTORY_CARD_WINDOWS_MB`** (`shadow.rs`, padrão 0): janelas com cópia única na VRAM. Na máquina B a VRAM está a ~95% do orçamento com `-SmallRam`, então só cabe junto com um teto de textura menor.
- **O roteiro do VMMap** (dossiê 07, seção 4) completa o `VICTORY_BOOK_SUMMARY`: o livro diz de quem são os recursos D3D12; o VMMap, que tipo de memória do processo cresce.
