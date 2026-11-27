# Playbook técnico: vai-funcionar na RTX 4080 (Windows 10)

Alvo: **RTX 4080 16 GB (Ada, AD103) · Ryzen 7 9800X3D · 32 GB DDR5 · Windows 10**.
Base de código: `SrGoes/vai-funcionar` @ `b2184b1` (lido em 26/11/2026, 23:07 BRT). Caminhos abaixo são relativos a `crates/`.

Como ler este documento:
- **[medido]**: número tirado do repo, de uma issue ou de um relatório público, com a fonte.
- **[estimativa]**: palpite meu de engenharia. Só vale depois de medir.
- **[não verificado]**: não consegui confirmar.

Escopo: engenharia de GPU e de runtime. Este playbook não cobre como obter ou rodar o executável, nem decriptação ou DRM.

---

## 0. Resumo em 1 minuto

1. **No Windows 10 o runtime nem chega a rodar sem o Agility SDK.** O D3D12 que vem no Windows 10 para em SM 6.5, e o VIctory exige SM 6.6 (`victory-shader/src/emit/host.rs`). Isso foi confirmado numa RTX 4080 com Windows 10 22H2 ([Marvisak/VIctory#84](https://github.com/Marvisak/VIctory/issues/84)). O vai-funcionar ainda não exporta `D3D12SDKVersion`/`D3D12SDKPath` (conferi com grep: nenhuma ocorrência). Esse é o passo zero.
2. **No Windows 10 não existe GPU upload heap.** O lançamento definitivo exige Windows 11 build 26080 ou mais nova ([blog do Agility 1.613](https://devblogs.microsoft.com/directx/agility-sdk-1-613-0/)). Por isso `VICTORY_WINDOWS_BAR_MB` e `Host::mapped_card_buffer` não fazem nada aí. O caminho de ReBAR no Win10 é o NVAPI (`NV_D3D12_RESOURCE_FLAG_CPUVISIBLE_VIDMEM`), ver §4.3.
3. **A parede é a CPU que prepara as submissões, não os shaders.** Na máquina mais parecida com a sua já medida (RX 7900 XTX 24 GB, 64 GB, i9-12900K), a mediana é 46–65 ms. Desses, 48 ms de um quadro de 55 ms são a thread de preparo, e o card fica ocupado uns 41 ms ([medido], `docs/findings.md`). No mesmo documento, o ganho que resta no código de shader foi estimado em menos de 1 ms.
4. **Metas realistas na 4080:**
   - mais de 20 fps (≤ 50 ms) **[estimativa: alcançável]**, porque a 7900 XTX já está nessa faixa;
   - 30 fps (33,3 ms), que é o que o título precisa no console ("The title needs 33 ms", `docs/findings.md`);
   - 60 fps (16,7 ms), que pede reduzir o preparo para menos de um terço **e** destravar o intervalo de quadro do jogo (`VICTORY_FRAME_INTERVAL_MS`, `victory-gpu/src/frame.rs`). Pode quebrar a lógica de um título feito para 30 Hz **[não verificado]**.

### As 5 ações de maior impacto (para a 4080)
1. **Agility SDK 1.620.2 no `victory.exe`** (exports mais `D3D12\D3D12Core.dll`). Sem isso não roda no Win10.
2. **Janelas (2/3 da memória compartilhada) na VRAM.**
   - No Win10: `VICTORY_CARD_WINDOWS_MB` mais `VICTORY_CARD_BUFFERS_MB` alto, aproveitando os 16 GB.
   - Depois: protótipo com NVAPI CPU-visible vidmem para ter uma cópia única.
3. **Atacar a thread de preparo:**
   - write watch uma vez por reserva por submissão (Kelvaroq 0045: p50 de 129 para 110 ms numa RTX 5080 [medido lá]);
   - port 2 de descritores (15–16 ms/quadro [medido]);
   - constantes lidas direto da cópia da GPU (Kelvaroq 0038–0040).
4. **Orçamento não local mais commit:**
   - checar `video_memory()[2..3]` antes de `shared_buffer`/upload;
   - pagefile grande no NVMe, porque com 64 GB o processo já chegou a 40–46 GB privados [medido].
5. **Cache de pipelines quente** (`victory warm`) e salvar a biblioteca fora dos quadros críticos (#15; travadas de mais de 200 ms entre 270 e 660 s [medido]).

---

## 1. Windows 10 na 4080: o que existe e o que não existe

| Recurso | No Win10 19045 (22H2) | Impacto no vai-funcionar | Fonte |
|---|---|---|---|
| WDDM | 2.7 (Win10 2004+). WDDM 3.x é só Win11 | Recursos que dependem de WDDM 3.x não existem | relato em [VIctory#84](https://github.com/Marvisak/VIctory/issues/84) ("Windows 10 Pro build 19045 (WDDM 2.7)"); [spec GPU upload heaps](https://microsoft.github.io/DirectX-Specs/d3d/D3D12GPUUploadHeaps.html) |
| SM 6.6 (heaps indexados direto, `ResourceDescriptorHeap[]`) | **Só com Agility SDK.** O runtime de fábrica reporta SM 0x65 | **Bloqueador.** Root signatures com `*_HEAP_DIRECTLY_INDEXED` são recusadas (flags `c01`) | [D3D_SHADER_MODEL](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/ne-d3d12-d3d_shader_model) ("6.6 … added in Windows 11 and the Agility SDK"); VIctory#84 |
| Agility SDK | Win10 1909+ (em 2004/20H2, revisão ≥ .789). Basta existir `C:\Windows\System32\D3D12Core.dll` | Exportar `D3D12SDKVersion` e `D3D12SDKPath` do exe | [Getting started Agility](https://devblogs.microsoft.com/directx/gettingstarted-dx12agility/) |
| GPU upload heaps (`D3D12_HEAP_TYPE_GPU_UPLOAD`, ReBAR via D3D12) | **Não.** O lançamento definitivo pede Win11 build 26080+ | `OPTIONS16.GPUUploadHeapSupported` fica falso. `Host::mapped_card_memory()` devolve false, e `VICTORY_WINDOWS_BAR_MB` e `VICTORY_INPLACE` (no que usar GPU_UPLOAD) caem no fallback | [Agility 1.613](https://devblogs.microsoft.com/directx/agility-sdk-1-613-0/), `victory-gpu/src/host.rs` `mapped_card_memory` |
| ReBAR | Funciona no Win10 com BIOS (Above 4G Decoding + Resizable BAR, CSM desligado) e driver recente | Só dá para usar via NVAPI (§4.3). A NVIDIA ativa ReBAR por perfil de aplicativo **[não verificado para exe desconhecido]**: meça com `NvAPI_D3D12_QueryCpuVisibleVidmem` | [nvapi.h](https://github.com/NVIDIA/nvapi/blob/main/nvapi.h) |
| HAGS (agendamento de GPU acelerado por hardware) | Sim, Win10 2004+, opt-in | A NVIDIA avisa que, sem HAGS, mais de duas filas (fora a de cópia) podem ser serializadas. O repo usa direct + copy + compute (ver §2). Teste com e sem | [MS HAGS](https://devblogs.microsoft.com/directx/hardware-accelerated-gpu-scheduling/), [NVIDIA Async Compute](https://developer.nvidia.com/blog/advanced-api-performance-async-compute-and-overlap/) |
| Enhanced barriers | É recurso do Agility SDK e depende do driver (`OPTIONS12.EnhancedBarriersSupported`) | `victory-gpu/src/host/place.rs` já consulta OPTIONS12. Olhe o log | **[não verificado no Win10 mais driver 6xx]** |
| `OpenExistingHeapFromAddress1` / `ID3D12Device13` (ports 1/5, `inplace.rs`) | Depende do Agility SDK | `port.rs` checa o cast para `ID3D12Device13` e desiste sem ele | `victory-gpu/src/port.rs:342` |
| Debug layer | `d3d12SDKLayers.dll` do mesmo NuGet. A feature opcional "Graphics Tools" precisa estar instalada | Necessário para GPU-based validation e DRED | VIctory#84 (comentário de colehammond65) |
| Suporte do SO | O Win10 saiu de suporte em 14/10/2025 | Novas versões do Agility podem deixar de testar no Win10. **Fixe a versão 1.620.2, que foi medida funcionando** | [não verificado se 1.7xx roda no Win10] |

### 1.1 Passo zero: Agility SDK (o mesmo patch medido em 4080, 5060 Ti, 2070 SUPER e 6800 XT)

Implementação descrita em VIctory#84; os PRs upstream são #2460 (exports em `apps/victory/build.rs`) e #175 (`cargo xtask agility`):

```rust
// apps/victory/src/main.rs
#[unsafe(no_mangle)]
pub static D3D12SDKVersion: u32 = 620;           // Microsoft.Direct3D.D3D12 1.620.2
#[repr(transparent)]
pub struct SdkPath(*const std::ffi::c_char);
unsafe impl Sync for SdkPath {}
#[unsafe(no_mangle)]
pub static D3D12SDKPath: SdkPath = SdkPath(c".\\D3D12\\".as_ptr());
```
```rust
// apps/victory/build.rs (hoje NÃO existe no vai-funcionar)
fn main() {
    println!("cargo:rustc-link-arg-bins=/EXPORT:D3D12SDKVersion,DATA");
    println!("cargo:rustc-link-arg-bins=/EXPORT:D3D12SDKPath,DATA");
}
```

- `D3D12Core.dll` e `d3d12SDKLayers.dll` do NuGet `Microsoft.Direct3D.D3D12` 1.620.2 vão em `target\<perfil>\D3D12\`.
- Confira com `dumpbin /exports victory.exe`.
- **Armadilha [medida]:** com os exports mas sem a pasta `D3D12\`, `D3D12CreateDevice` falha de vez ("the D3D12 SDK version configuration of the host exe is invalid"). Não há fallback.
- Isso vale também para o runner pré-compilado `victory-3d-rtx-pareto-residency.exe` que `Play-VIctory.cmd` procura.
- Os testes de GPU do `victory-shader` precisam do mesmo tratamento para o binário de teste (upstream #486).
- Critério de pronto:
  - o log mostra SM 6.6;
  - `victory translate <amostra> --validate --pipelines` cria todos os pipelines (40/40 na 4080 em #84);
  - o mundo carrega.
- Aviso [medido em #84 / #71]: numa 4080 SUPER no Win10, depois do Agility, o mundo rodou a ~0,3 fps e o device caiu com `0x887A0006` (DEVICE_HUNG). Isso está rastreado em VIctory#71/#597.
  - O vai-funcionar tem `VICTORY_GROUP_SYNC` e as correções do wave64. Teste se o hang aparece nele.
  - Se aparecer: `VICTORY_D3D12_DRED=1` e o procedimento de #71, que exclui programas um a um. A chave `VICTORY_SKIP_SHADERS` era um patch local lá e **não existe** no vai-funcionar.

### 1.2 Hardware: 4080 contra 5060, e o que isso muda

| | RTX 5060 (relatório) | RTX 4080 (sua) |
|---|---|---|
| Arquitetura | Blackwell GB206, 30 SMs | Ada AD103, 76 SMs |
| VRAM | 8 GB GDDR7, 448 GB/s | 16 GB GDDR6X, 256 bits (~717 GB/s) |
| Barramento | PCIe 5.0 x8 | PCIe 4.0 x16 (mesma ordem de banda teórica, ~32 GB/s por sentido) |
| L2 | 32 MB | 64 MB |
| Orçamento local D3D12 | 7.127 MB [medido] | ~15 GB **[estimativa: meça no log "video memory as the system counts it"]** |
| Orçamento não local (compartilhado) | 7.446 MB com 16 GB de RAM [medido] | ~metade dos 32 GB, ou seja ~16 GB **[estimativa]** |
| CPU | i5-13400F | 9800X3D: 8 núcleos/16 threads num único CCD, 96 MB de L3. Sem problema de latência entre CCDs para a thread de preparo |

Consequências diretas no código:
- `shadow.rs` `default_card_limits_mb`: com orçamento acima de 12 GiB, o default passa de 16 MiB/512 MiB para **4 MiB/2048 MiB** de cópias em VRAM. Com `VICTORY_CARD_BUFFERS_MIN_KB=64` (já no `Play-VIctory.cmd`), todos os buffers do título são candidatos.
- **Use o perfil Series X.** Com 32 GB deve caber [estimativa]:
  - `Play-VIctory.cmd` já usa `--console series-x`, porque Series S perde a malha da cabeça em 8 GB;
  - o fork fxceless roda Series X numa 4070 com 32 GB;
  - na 5060 com 16 GB, Series X caía (#3).
- A queda em ~9 min da 5060 (memória compartilhada esgotada, #2) deve demorar mais com 16 GB de orçamento não local, mas **não some**: com 64 GB, o processo chegou a 40–46 GB privados (`docs/findings.md`: alocadores de listas 9 GB, janelas 7, uploads 4).

---

## 2. O motor gráfico como o runtime vê (do `ExecuteCommandLists` do título ao Present)

O console tem memória unificada: o endereço de GPU é o endereço de CPU (`victory-gpu/src/lib.rs`). No PC tudo que um lado escreve e o outro lê precisa ser **notado** e **movido**. `docs/findings.md` resume: "Every large cost is one of four: noticing writes of the title, noticing writes of shaders, moving them, and waiting."

```
título (x64 nativo)
 └─ D3D12.X inline: escreve pacotes enquanto put < limit; senão chama o método
     (o runtime zera os limites → TODO comando vira chamada)        lib.rs, list.rs
 ├─ Command lists: cada lista do título grava numa host command list   list.rs, draw.rs
 │   · draw.rs: por draw, atualiza pipeline host, root signature nossa,
 │     user data (b0), cópias host dos buffers/descritores lidos, estados
 │   · descritores: heaps do título são memória zerada; handles = endereços;
 │     views de 32 B e samplers de 16 B (tamanho RDNA2)          descriptor.rs
 │     espelho host na mesma ordem; slot refeito quando o título escreve ou o
 │     draw acha mudança                                        descriptor/host.rs
 │   · ExecuteIndirect: argumentos copiados e limitados na GPU antes (clamp)  indirect.rs
 ├─ Submissão (ExecuteCommandLists do título)                         queue.rs
 │   · antes: refresh das páginas de 64 KiB que o título escreveu      shadow.rs, watch.rs
 │     (write watch por página de 4 KiB; vistas físicas: read-only + VEH)
 │   · cópias "card": ranges mudados sobem p/ VRAM (Host::card_before)
 │   · depois: compare na GPU dos ranges declarados como escritos e merge
 │     de volta na memória do título (Host::card_back, Shadow::card_merge)
 │   · land.rs: o que os shaders escreveram volta à memória do título em threads
 │   · por padrão TODA submissão é esperada; VICTORY_LATE_WAIT / VICTORY_RELAY
 │     mudam isso (host.rs late()/late_stage(); relay_worker)
 │   · VICTORY_ASYNC_SUBMIT: uma thread nossa prepara/envia em ordem  queue/behind.rs
 ├─ Pipelines: tradução GFX10→DXIL, cache de shaders, ID3D12PipelineLibrary
 │   (1,3–1,5 GB, mapeada), workers = 2/3 dos núcleos lógicos (2–16)
 │                                       pipeline/library.rs, pipeline/make.rs
 ├─ Streaming de texturas
 │   · texturas que o título escreve: memória tiled do console → detile num
 │     staging → cópia na textura host quando mudou              texture.rs
 │   · primeiros uploads numa fila de cópia própria (VICTORY_UPLOAD_AHEAD)   texture/ahead.rs
 │   · níveis sob demanda: só os mips que o título mapeou ocupam VRAM   tiles.rs
 │   · blocos de memória reaproveitados (alocação ~310 µs cada no AMD)   pool.rs
 │   · admissão por headroom de commit; E_OUTOFMEMORY → upload_refused   residency.rs
 └─ Frame e Present
     · relógio próprio ticando no intervalo do título; VICTORY_FRAME_INTERVAL_MS
       força outro (16.667 = 60/s)                                     frame.rs
     · Series S compõe duas camadas (UI sobre cena) escaladas             compose.rs
     · DXGI: um adaptador, uma saída                                     dxgi.rs
```

Para onde vai o quadro [medido, 7900 XTX, `docs/findings.md`], em 55 ms na água:
- **Thread de preparo: 48 ms.** Inclui:
  - passos das listas: 18;
  - refresh das cópias: 10;
  - passos no card: 7;
  - espera de registros de descritor: 6;
  - consulta ao write watch: 5;
  - cópias em VRAM: 4;
  - bytes do título sob escrita de shader: 4.
- **Card: ~41 ms ocupado.**
- **Chamadas de lista:** 36.000 por quadro, somando 230 ms de trabalho em 8 threads (67.000 numa perseguição).
- **Descritores:** 400 mil slots examinados e 40 mil espelhados por quadro.

Na 5060 [medido, relatório de 26/11]:
- GPU a 99% de "uso" mas só 49 W de 145 W;
- na loja, 536 ms copiando 290 MB por quadro e 3,3 milhões de slots varridos.

A explicação mais provável **[estimativa]** é que a GPU estava esperando leituras via PCIe de buffers em memória do sistema, e não calculando.

---

## 3. Traduzir a arquitetura de shaders do Xbox (RDNA2) para NVIDIA (Ada)

### 3.1 O que chega no runtime

Um shader do Series é um contêiner DXBC com parte `DXIL` (`victory-shader/src/dxbc.rs`, `dxil.rs`). Em volta do bitcode existe uma tabela `XDAT` com blobs, e entre eles está **o código GFX10 nativo comprimido com zlib** (`native.rs`).

O bitcode DXIL do console usa intrínsecos exclusivos do console. Por isso **o runtime traduz o ISA nativo, não o DXIL** (`lib.rs`). O registro nativo traz, entre outras coisas:
- o bit de wave32/wave64;
- as contagens de VGPR e SGPR;
- a tabela de layout de root (como cada parâmetro chega em user SGPRs);
- a tabela de recursos usados;
- para pixel shader, as entradas interpoladas e os "export patches" por MRT.

### 3.2 Conceitos de RDNA2 que importam, e o equivalente no host

| Conceito RDNA2/GCN | O que é | No DXIL/NVIDIA | Onde o runtime trata |
|---|---|---|---|
| **Wave64 / wave32** | O compilador do console escolhe por programa (bit no registro nativo) | Warp de 32 na NVIDIA. `WaveSize(64)` é **recusado** em host com `WaveLaneCountMax` 32 | `emit/lower.rs` `set_host_max_wave` (~l.1378), divisão em metades (~l.1800–1925) |
| **SGPR** (escalar, 1 valor por wave) | user data, descritores, constantes, máscaras | DXIL não tem registrador escalar. Tudo é valor por lane, e o driver descobre a uniformidade | `ir.rs`: cada dword de registrador vira valor SSA. `lane_uniform()` em `lower.rs` prova valores uniformes guardados em VGPR |
| **VGPR** (vetorial, 1 por lane) | dados por thread | `i32` em DXIL; floats via bitcast | `emit/lower.rs` (cabeçalho) |
| **EXEC / VCC / SCC / M0** | máscara de lanes ativas, carry/compare, flag escalar, registrador auxiliar | Não existem. Divergência vira fluxo de controle estruturado por lane | `ir.rs` torna os implícitos explícitos. `structure.rs` remove EXEC: máscaras viram booleanos por lane, e cada instrução que não dá para provar ativa ganha um `if` de guarda. O CFG precisa ser redutível |
| **SALU / SMEM** (`s_load_dword*`) | carga escalar de descritores e constantes | Carga de CB ou raw buffer; uniformidade a critério do driver | `resource.rs` rastreia cada operando de descritor até o parâmetro de root. `spill.rs` cuida da tabela de user data em memória (40 bits) |
| **LDS** (64 KB por workgroup no RDNA) | memória compartilhada do grupo, `ds_*` | `groupshared`, **máx. 32 KiB em D3D12** | `HOST_LDS_DWORDS = 8192` (32 KiB) em `lower.rs`. A troca entre metades de wave64 também consome LDS |
| **DPP / `v_permlane*` / `v_readlane` / `ds_swizzle` / `ds_(b)permute`** | troca entre lanes | `WaveReadLaneAt`, `QuadReadLaneAt`, `WavePrefix*`, `WaveActive*` | `emit/wave.rs` simula seções de wave inteira de forma simbólica e emite a redução ou prefixo equivalente. `emit/permlane_tests.rs` |
| **Máscara como número** (`s_bcnt1`, `s_ff1`, `v_mbcnt`) | lanes usadas como inteiro | `WaveActiveBallot` devolve `uint4` (128 bits) | `MaskData::{Console,Host,None}` em `lower.rs`: em compute é exato (wave do console), em pixel/vertex só usos independentes do tamanho da wave, em mesh é recusado |
| **Exports** (`exp pos/param/mrt`) | saída do estágio | `SV_Position`, saídas, `SV_Target[n]` | Export patches por MRT (`native.rs`). Retângulos (topologia 17) viram triângulo mais um GS nosso (`emit/rect.rs`) |
| **Interpolação** (`v_interp_p1/p2`, `_mov`) | atributos no PS | Entradas interpoladas e `GetAttributeAtVertex` para flat | `lower.rs` `interp` (~l.11216–11380) |
| **MUBUF/MTBUF/MIMG** (formato no descritor/instrução) | carga tipada, amostragem | Carga raw e conversão manual; `Sample*`/`Load` | Vertex fetch `tbuffer_load_format_*` vira bytes crus mais conversão (`emit/mod.rs`). Samplers imediatos (4 palavras `SQ_IMG_SAMP_WORD0-3`) viram static samplers (`emit/sampler.rs`) |
| **Descritores D3D12.X** | V#/T#/S# crus de 32/16 B em memória; o título copia bytes | SM 6.6 `ResourceDescriptorHeap[i]`, com o índice tirado do endereço do console | `emit/mod.rs` (user data em `b0`, root CBV/SRV em `ROOT_CBV_SPACE`). Os heaps host mantêm a ordem do título |

### 3.3 O pipeline do tradutor (decisão 0003)

```
DXBC → native.rs (acha o GFX10) → gfx10/ (decodifica por fluxo de controle, não varredura linear)
 → ir.rs (SSA 1:1, implícitos explícitos, VGPR escrita lê o valor antigo)
 → structure.rs (sem EXEC; guardas; CFG redutível; helper lanes contam como vivas)
 → resource.rs + spill.rs (cada descritor ligado a um parâmetro de root)
 → emit/wave.rs, own_bits.rs, threading.rs (formas especiais provadas, sem casar padrões)
 → emit/lower.rs (instrução a instrução → LLVM; valor inexpressável = Missing(motivo))
 → bitcode/opt (VICTORY_SHADER_OPT: passes exatos; 63 por padrão, 127 com o de cargas x4)
 → emit/bitcode.rs (bitcode LLVM 3.7 escrito direto, SEM dxc) → emit/validator.rs (IDxcValidator do dxil.dll)
 → cache de shaders: chave = TRANSLATOR_VERSION + fonte do tradutor + cada VICTORY_* lido + fatos do host
```

Implicações:
- **O driver NVIDIA é o único otimizador de verdade** (DXIL → SASS). O DXIL emitido é "um dword, um i32, com bitcasts", e o driver faz a alocação de registradores.
- Mudar qualquer chave que entra na tradução **invalida o cache** (`emit/mod.rs` `host_settings`). Em A/B, aqueça os dois lados.

### 3.4 Armadilhas na NVIDIA (o que o código já sabe e o que falta)

1. **Wave64 em warps de 32** (#11, 4 programas pendentes: `vfx_triangleClip` CS_DefragHeap_ComputeRelocations e CS_CollectTriangles_Meshlets, `water_shading` CS_TransmissionVolumetrics_Tiled e a variante LocalLights).
   - Programas cuja troca entre lanes fica dentro de metades de 32 rodam como duas waves.
   - Os que cruzam metades usam troca por dwords em LDS mais encontros de grupo (`exchange`, `halves`, `with_half_meets`).
   - **A numeração não é consecutiva na NVIDIA**: "NVIDIA's 8 x 8 groups". `wave_base()` (l.~4560) pega a base da wave com um contador em LDS.
   - Motivos de recusa em #11: grupo de 128 threads, LDS em grupo acima de 32, leitura e escrita de LDS na mesma região de branch por lane, máscara como dado, `v_mbcnt`, `v_readlane`.
   - Diagnóstico: `VICTORY_WAVE_WHY=1`.
   - Para resolver:
     - `v_mbcnt` sobre a wave do console = `WavePrefixCountBits` da metade mais `countbits` da metade de baixo, trocado via LDS;
     - `v_readlane` em lane fixa de outra metade = escrita em LDS por aquela lane mais barreira;
     - a barreira só pode ser emitida onde todas as threads do grupo chegam (ver item 3).
2. **Largura do ballot.** `WaveActiveBallot` é `uint4`. No console a máscara é de 64 bits (wave64) ou 32.
   - Em pixel/vertex, o host pode ter qualquer tamanho de wave, por isso só são aceitos usos invariantes (contagem, primeira lane, contagem abaixo, igualdade).
   - Não "conserte" isso assumindo 32, porque o driver pode mudar.
3. **Barreira de grupo divergente = TDR na NVIDIA.**
   - `emit/mod.rs` `group_sync`: "NVIDIA's wait is for every thread: the group stands until Windows resets the card."
   - `VICTORY_GROUP_SYNC=1` só espera em blocos que todas as lanes executam; `=0` é diagnóstico (resultado errado).
   - `VICTORY_WAVE_WIDE` está desligado porque derrubou o card (runs 121–124).
4. **Derivadas e helper lanes.**
   - Helper lanes contam como vivas na estruturação (`structure.rs`).
   - "Alguma lane do quad ativa" é assumido verdadeiro (`Atom::Quad`, atalho documentado).
   - Em waterfall loops, a helper lane lê o próprio valor (`VICTORY_HELPER_OWN`, padrão ligado). Isso evita loops infinitos de helpers e quads sem amostra nas bordas.
   - Bug real já visto: leituras de vizinho do quad com wave reads deram buracos em sombras no AMD (`docs/findings.md`).
   - Na NVIDIA use `QuadReadLaneAt` (`quad_pick`) e nunca wave reads para vizinhos do quad.
5. **Precisão e semântica de float (medido na NVIDIA):**
   - `f32tof16` **arredonda para zero** na NVIDIA, enquanto o console arredonda para o mais próximo. `VICTORY_F16_NEAREST=1` deixa exato, a uma dúzia de instruções por resultado, e está desligado por padrão.
   - Sob fast-math o driver dobra `fcmp uno`: o NaN é testado em aritmética inteira (`is_nan_f32`).
   - `r > k ? k : r` vira `FMNMX` e engole NaN (`fract`).
   - `fcmp one`: o AMD trata como unordered; NVIDIA e WARP não.
   - Leitura raw por uma view estruturada dá **zeros na NVIDIA**, por isso todas as views de buffer são raw.
   - `VICTORY_RDNA_FLOAT` (atalhos de float nativos) **só liga em RDNA**: na NVIDIA tudo é soletrado de forma exata. Ganho na 7900 XTX: 0,8 ms.
   - Oportunidade na 4080: rodar `emit/probe_tests.rs` (`VICTORY_PROBE=<pasta>`, programas `sem*` com 0, inf, -0, NaN) e ver quais das 4 regras (MAX_MIN, LEGACY, MED3, FRACT) a NVIDIA também cumpre. Ganho **[estimativa] < 1 ms**.
6. **Limite de descritores.**
   - A NVIDIA limita o heap de views a 1.000.000; o heap de um jogo é maior.
   - Ler além do fim **pendura a GPU** (validação: "descriptor heap index out of bounds").
   - O runtime reserva um slot nulo no fim e prende índices nele (`emit/shader.rs` `VIEW_LIMIT`).
   - A orientação da NVIDIA é ≤ 1M CBV/SRV/UAV e ≤ 2K samplers ([Command Buffers](https://developer.nvidia.com/blog/advanced-api-performance-command-buffers/)).
7. **Flag `AllResourcesBound` não é emitida.** Os bits de ShaderFlags em `emit/shader.rs` `flags` não incluem o bit 8 (`D3D12_SB_GLOBAL_FLAG_ALL_RESOURCES_BOUND`, ver [DxilShaderFlags.h](https://github.com/microsoft/DirectXShaderCompiler/blob/main/include/dxc/DXIL/DxilShaderFlags.h)).
   - A NVIDIA recomenda ligar ([Shaders](https://developer.nvidia.com/blog/advanced-api-performance-shaders/)).
   - Risco: os heaps são indexados direto, e o significado com slots nulos e vazios precisa ser conferido.
   - É experimento barato, porque muda a chave do cache. Ganho **[estimativa] 0–1 ms de GPU**.
8. **Pressão de registradores e ocupação no Ada (SM 8.9).**
   - Por SM: 64K registradores de 32 bits, máximo de 48 warps (1.536 threads), 100 KB de memória compartilhada ([CUDA Programming Guide, tabela de compute capability](https://docs.nvidia.com/cuda/cuda-c-programming-guide/index.html#features-and-technical-specifications)).
   - Exemplo: um shader traduzido com 128 registradores por thread → 512 threads/SM → 16 warps (33% de ocupação) **[conta, não medido]**.
   - Fontes de pressão no código traduzido:
     - guardas por instrução;
     - simulação de seções de wave inteira;
     - LDS de troca do wave64, que também reduz grupos por SM;
     - `f16_nearest`.
   - Como ver: Nsight Graphics → GPU Trace → Shader Profiler (registradores, ocupação, motivos de stall por instrução SASS) **[nome exato das telas não verificado nesta versão]**.
9. **Formas não suportadas** (de VIctory#84 e #597, para saber o que ainda cai):
   - baricêntricas computadas;
   - atomic 64 bits em imagem;
   - máscara de lane como dado;
   - leitura escalar de memória como dado;
   - descritor não rastreado;
   - "an address not from a root parameter";
   - mesh shader acima de 32.768 bytes de saída;
   - "a structured buffer of unknown stride".

### 3.5 Como validar uma tradução

- **Testes do próprio crate:**
  - `cargo test -p victory-shader -- --test-threads=1`. O estado global indexado por endereço deixa os testes instáveis em paralelo (#26).
  - Suítes por tema: `wave64_tests.rs`, `valu_tests.rs`, `permlane_tests.rs`, `waterfall_tests.rs`, `cube_lod_tests.rs`, `intersect_tests.rs`, `size_tests.rs`.
- **Diferencial:** `emit/diff_tests.rs` compila o mesmo HLSL no dxc do console e no dxc do PC e compara a saída na GPU.
  - Precisa de `VICTORY_CONSOLE_DXC`, que é o SDK do console. Se você não tem licença, não use.
  - Use os outros testes mais o `probe`.
- **Validação em lote:** `victory translate <pasta> --validate --pipelines` traduz, valida e cria PSO. Use depois do Agility SDK.
- **Retradução:** `victory shaders` retraduz o que um build anterior deixou em cache e informa quantos saíram diferentes. Se algum mudou, era obrigatório incrementar `TRANSLATOR_VERSION`.
- **Dumps:** `VICTORY_EMIT_DUMP`, `VICTORY_WAVE_DUMP`, `VICTORY_WAVE_WHY=1`. `VICTORY_SHADER_OPT=0` separa um erro do tradutor de um erro dos passes de limpeza.
- **No GPU:**
  - D3D12 debug layer mais GPU-based validation (índices fora do heap, estados);
  - `VICTORY_D3D12_DRED=1` para hang ou remoção (breadcrumbs mostram o comando);
  - Nsight Graphics **Shader Profiler** para hot spots e ocupação;
  - **Nsight Aftermath** para crash dumps de GPU [ferramenta existe; integração no repo não verificada].
- **Visual:** capturas a cada N quadros do `scripts/test-session.ps1` com diff de imagem entre A e B, no mesmo save e no mesmo cache.

---

## 4. Gargalos por ordem de impacto (4080)

Para cada item: mecanismo, técnica no código, ganho esperado, risco e como medir.

### 4.1 Bloqueador: SM 6.6 no Win10
Ver §1.1. Medir: log de inicialização e `translate --validate --pipelines`.

### 4.2 Thread de preparo (CPU), a parede principal
- **Mecanismo** (`docs/findings.md` mais o código): antes de cada submissão o runtime:
  - consulta o write watch;
  - atualiza as cópias de 64 KiB;
  - espelha descritores;
  - escreve user data;
  - grava barreiras.
  Por padrão espera a GPU (header de `host.rs`).
- **Técnicas, em ordem de impacto [estimativa]:**
  1. **Port 2, descritores** (`port_descriptors.rs`, `VICTORY_PORT_DESCRIPTORS=1`): marca slots em vez de varrer as páginas mudadas. Antes: ~290 páginas, ~600 mil slots, 15–16 ms por quadro [medido]. Não foi testado no NVIDIA 8 GB. Teste na 4080.
  2. **Ports 5 e 1, CBs no lugar** (`VICTORY_PORT_CBV`, `VICTORY_PORT_CONSTANTS`):
     - tiram ~23.000 cópias por quadro (~9 MB) e 17 MB em ~600 pedaços [medido];
     - no 8 GB NVIDIA: estrada sumindo e exceção C++ (`Play-VIctory.cmd`);
     - **hipótese a testar:** o bug ligado a orçamento de 8 GB não aparece com 16 GB;
     - depende de `ID3D12Device13` (Agility).
  3. **Write watch uma vez por reserva por submissão** (Kelvaroq correção 0045, `VICTORY_WATCH_RESERVATION`/`VICTORY_COPY_SMALL`): p50 de 129,2 para 109,5 ms [medido na 5080, [documentation.md](https://github.com/stellarr-studio/kelvaroq/blob/HEAD/documentation.md)]. No vai-funcionar a chamada está em `watch.rs:~703` (`GetWriteWatch`).
  4. **Constantes de tabela lidas da cópia da GPU** em vez de copiadas pela CPU (Kelvaroq 0038, `VICTORY_DIRECT_CONSTANTS`). **Espera de fill só nos blocos de 64 KiB** em que um voo mudou constantes, em vez de uma chave global (`VICTORY_FILL_WAIT`).
     - Kelvaroq mediu 15 esperas e 170–590 ms por quadro causadas por isso.
     - O vai-funcionar não tem nenhuma das duas chaves (grep). Porte a ideia, não o patch: o código diverge.
  5. **Paralelizar a submissão em duas threads em sequência** e pedir a próxima enquanto esta é enviada ("What is next" nº 1 em `findings.md`).
  6. `VICTORY_THIN=1` (listas gravadas e traduzidas por 8 threads): 75 para 68 ms na 7900 XTX [medido]. **Desligado no NVIDIA** porque a pele da cabeça some. Ache a causa com diff de captura antes de ligar.
- **9800X3D:** 16 threads lógicas → `pipeline/make.rs` usa 2/3, ou seja ~10 workers. Os 96 MB de L3 ajudam memcpy e hash de páginas quentes [estimativa]. Rode com o plano de energia em alto desempenho.
- **Medir:**
  - Nsight Systems com `--trace=dx12,wddm,osrt,nvtx --dx12-wait-calls=true --sample=cpu` mostra em que função a thread de preparo vive e quanto tempo fica bloqueada em fence;
  - `VICTORY_PANEL`/F3 mostra a divisão por etapa;
  - PresentMon `MsCPUBusy` contra `MsGPUBusy`.

### 4.3 A GPU lendo memória do sistema via PCIe
- **Mecanismo:**
  - `Host::shared_buffer()` (`host.rs:~10759`) cria buffers em heap CUSTOM, `WRITE_BACK`, **L0 (RAM)**, para as cópias host. O vkd3d-proton classifica esse heap como tipo READBACK, ou seja, host-cached e sem VRAM (`libs/vkd3d/memory.c` `vkd3d_normalize_heap_type`).
  - Cada carga de shader nessas cópias atravessa o PCIe. Um grupo de skinning de 32 threads levou 1,8 ms, o tempo de ~170 cargas seriais (`shadow.rs`).
  - **Janelas** (vértices, índices e constantes do alocador do título) são **2/3 da memória compartilhada**: run 104, 861 janelas, 3.552 de 5.400 MB. Nenhum shader as escreve (run 91), segundo `shadow.rs`, comentário de `window_card_most`.
- **Técnicas no Win10:**
  1. `VICTORY_CARD_WINDOWS_MB=2048..4096`: janela com lado na VRAM, o lado CPU continua na RAM e o que mudou sobe antes da submissão. **Não reduz** a memória compartilhada (comentário do código), mas tira as leituras do PCIe.
  2. `VICTORY_CARD_BUFFERS_MB=4096`, com `_MIN_KB=64` já padrão. Com 16 GB cabe bem mais que os 2048 padrão. Observação: as cópias escritas por shader ocupam o tamanho duas vezes (gêmeo "before").
  3. **Código: cópia única em VRAM visível pela CPU via NVAPI** (o que `VICTORY_WINDOWS_BAR_MB` faz com GPU_UPLOAD no Win11):
     - `NvAPI_D3D12_CreateCommittedResource(..., NV_RESOURCE_PARAMS{NVResourceFlags = NV_D3D12_RESOURCE_FLAG_CPUVISIBLE_VIDMEM})` em heap **UPLOAD**. Só heaps de upload aceitam o flag, então o recurso fica `GENERIC_READ`, e isso serve porque janelas só são lidas.
     - Antes, `NvAPI_D3D12_QueryCpuVisibleVidmem` para saber total e livre. Se o total for ~256 MB, o ReBAR não está ativo.
     - "SUPPORTED OS: Windows 10" segundo o [nvapi.h](https://github.com/NVIDIA/nvapi/blob/main/nvapi.h).
     - Junte isso em `window_mapped_for()` em `shadow.rs` como alternativa quando `mapped_card_memory()` for falso.
     - Ganho: no Win11, `WINDOWS_BAR` deu listas de 29 para 26 ms e quadro de 36 para 32 [medido no repo]. Kelvaroq usa `VICTORY_REBAR_COPIES_MB=2048` na 5080.
     - Risco: o CPU escreve em memória write-combined via PCIe. **Nunca leia** esse mapeamento (Kelvaroq mediu ~30 MB/s de leitura).
- **Medir:**
  - Nsight Systems com `--gpu-metrics-devices=all`, métrica `PCIe Read Bandwidth`/`pcie__throughput` (lista de métricas do [guia](https://docs.nvidia.com/nsight-systems/UserGuide/index.html)); compare com a potência (W) da placa;
  - Nsight Graphics GPU Trace: throughput por unidade e ocupação de SM. Com a GPU "99% ocupada" e unidades baixas, a GPU está esperando memória;
  - `VICTORY_MEMORY_CATEGORIES=1` mostra quanto há em cada tipo de heap.

### 4.4 Memória: orçamento não local, commit e pagefile
- **Mecanismo:**
  - `shared_buffer()` cria recurso committed L0 e o torna residente **sem consultar o orçamento não local**. `video_memory()` lê os dois grupos (`host.rs:~10681`), mas a admissão (`resource::room`, `card_for`, `window_mapped_for`) só usa o local.
  - `residency.rs` admite por headroom de commit, não por DXGI.
  - Na 5060, a sequência até a morte foi: 18.844 "no upload buffer", 5.695 "no shared buffer", "no memory to fetch what shaders wrote" e por fim `0x887A0005` (#2).
- **Técnicas:**
  1. Em `shared_buffer`, `upload_buffer` (`host.rs:~9822`) e no pool de readback, recusar ou adiar quando `shared_used + size > 95%` do orçamento não local. Registrar `RegisterVideoMemoryBudgetChangeNotificationEvent`.
  2. Degradar em vez de morrer:
     - liberar bytes ociosos do pool de readback;
     - adiar `UPLOAD_AHEAD`;
     - reduzir `VICTORY_TEXTURE_BUDGET_MB`.
  3. Subalocar `shared_buffer` em heaps placed de 64 MiB em vez de um committed por janela (861 objetos). É a orientação da NVIDIA: "prefer placed resources in CreateHeap", evitar churn ([Memory and Resources](https://developer.nvidia.com/blog/advanced-api-performance-memory-and-resources/)).
  4. Pagefile fixo e grande no NVMe. Com 64 GB o processo foi a 40–46 GB privados [medido], então com 32 GB vai paginar. `VICTORY_MEMORY_MAP=1` mostra o mapa.
  5. Atacar os maiores consumidores: alocadores de listas (9 GB), janelas (7 GB), uploads (4 GB). A biblioteca de pipelines ocupa 1,3–1,5 GB mapeada.
- **Medir:**
  - Nsight Systems com `--trace=wddm --wddm-memory-trace=true` (eventos `VidMmProcessBudgetChange`, `VidMmProcessUsageChange`, `VidMmProcessDemotedCommitmentChange`);
  - linha "video memory as the system counts it" do log;
  - Gerenciador de Tarefas → GPU → "Memória compartilhada da GPU";
  - **Critério:** 30 min sem `0x887A0005` e com memória compartilhada estável (inclinação perto de 0 MB/min depois dos 10 min).

### 4.5 Pipelines: cache frio e salvamento da biblioteca (#15)
- **Mecanismo** [medido em #15]: com cache frio caem de 600 a 1.400 draws diretos e de 1.000 a 2.300 indiretos por quadro. A fila de pipelines foi de 924 para 431 em ~2,5 min; com cache quente, ~3 pipelines são criados por quadro.
  - Salvar a `ID3D12PipelineLibrary` travou quadros por mais de 200 ms entre 270 e 660 s.
  - Runs 72/73/82 removeram o device (`DRIVER_INTERNAL_ERROR`) depois de salvar com o handle fechado (`pipeline/library.rs`).
- **Técnicas:**
  - `victory warm --used` depois de cada build ou driver novo;
  - `VICTORY_LIBRARY_STORE_LATER=1` (já no `.cmd`);
  - `VICTORY_LIBRARY_SAVE_S` com intervalo longo;
  - estratégia da NVIDIA de "miss and update" com bibliotecas pequenas, e não serializar centenas de milhares de PSOs de uma vez porque incha a memória do sistema ([PSOs](https://developer.nvidia.com/blog/advanced-api-performance-pipeline-state-objects/));
  - aumentar o tamanho do cache de shader do driver no Painel de Controle NVIDIA ("Shader Cache Size"; não afirmo o padrão);
  - `VICTORY_PIPELINE_WORKERS` (padrão 2/3 dos núcleos lógicos): `findings.md` sugere um quarto dos workers para os lentos.
- **Medir:**
  - draws descartados por quadro na linha de frame do log;
  - contagem de quadros acima de 200 ms nos primeiros 15 min (PresentMon `MsBetweenPresents`);
  - tamanho do arquivo da biblioteca.

### 4.6 Espera e serialização CPU↔GPU
- **Mecanismo** (`host.rs` `late()`/`late_stage()`):
  - run 86: quadro de 446 ms = 134 de preparo + 194 de espera + 78 de cópia de volta, em série;
  - run 92: 172 de 200 ms esperando.
  - `VICTORY_RELAY=2` (padrão nos scripts) liga o estágio 3 mais o `relay_worker`. `VICTORY_GPU_TIMING` força o estágio 0, então não meça desempenho com ele ligado.
- **Técnicas:**
  - depois de 4.2–4.4 a espera cai sozinha, porque a GPU deixa de esperar PCIe;
  - `VICTORY_ASYNC_SUBMIT` (`queue/behind.rs`; 202 de 292 ms dentro de `queue::execute` no profile do fxceless [medido lá]);
  - duas filas de cópia com fences cruzadas (Kelvaroq `VICTORY_QUEUE2`, não existe no vai-funcionar).
- **Regra da NVIDIA:** 5–10 `ExecuteCommandLists` por quadro; cada ECL custa 50–80 µs de agendamento do SO ([Command Buffers](https://developer.nvidia.com/blog/advanced-api-performance-command-buffers/)). `VICTORY_JOIN_COPIES`/`JOIN_PARTS` juntam submissões. `JOIN_PARTS` está desligado no NVIDIA porque repete quadros do vídeo.
- **Medir:** Nsight Systems `--dx12-wait-calls=true` (tempo em `Wait`/`SetEventOnCompletion`), linhas de fila DX12/WDDM e `MsGPUWait` do PresentMon.

### 4.7 Trabalho na GPU (depois que a CPU sair da frente)
- Na 7900 XTX o card fica ~41 ms ocupado [medido]. Na 4080 é desconhecido, então meça.
- Passes que rodam à toa sem RT:
  - estágios do construtor de RT descartados (360–2.540 por quadro);
  - `CS_RaytracingResolve` deixado de fora (574–778).
  Corte o produtor deles se o consumidor nunca roda (`findings.md`, "What is next" nº 2).
- Barreiras: ~125 chamadas a `ResourceBarrier`, e barreiras melhoradas só em `host/place.rs`. Siga a NVIDIA ([Barriers](https://developer.nvidia.com/blog/advanced-api-performance-barriers/)):
  - agrupar numa chamada só;
  - nada de `GENERIC_READ`;
  - nada de `COMMON` fora do estado inicial (`shared_buffer` nasce em COMMON);
  - nada de barreira read→read;
  - usar split barriers.
  Toda troca de tipo de trabalho (draw/dispatch/cópia) ou barreira UAV na mesma fila gera um WFI ([Async Compute](https://developer.nvidia.com/blog/advanced-api-performance-async-compute-and-overlap/)).
- Async compute **só depois** de um GPU Trace mostrar unidades baixas e slots de warp sobrando. Tarefas abaixo de 0,2 ms não compensam.
- **Medir:** Nsight Graphics GPU Trace. Trave os clocks para comparar A/B; a ferramenta tem opção de travar no clock base [não verificado o nome exato].

### 4.8 Upscaler: FSR4 quebrado (#12) contra DLSS
- **Estado [medido]:**
  - os passes `FFX_FSR4` 21–26, `SPD_AUTOEXPOSURE` e `RCAS` falham em `DeserializeComputePipelineStateX` com `0x80004001` e caem para "NewBE";
  - `amd_fidelityfx_upscaler_xbox.dll` é recusada pela política do host;
  - resolução interna ~720p.
- **Opções:**
  1. Implementar `DeserializeComputePipelineStateX` no HLE. Mantém o FSR4 do jogo, mas exige traduzir shaders de ML/compute pesados.
  2. **DLSS SR via [Streamline](https://github.com/NVIDIA-RTX/Streamline/blob/main/docs/ProgrammingGuideDLSS.md).** Tags obrigatórias:
     - `kBufferTypeScalingInputColor`, `kBufferTypeScalingOutputColor`, `kBufferTypeDepth`, `kBufferTypeMvec`;
     - opcionalmente `kBufferTypeExposure`, que sem ela usa auto-exposure.
     Constantes por quadro: `jitterOffset`, `mvecScale`, `cameraViewToClip`, `clipToPrevClip`, `depthInverted`, `cameraMotionIncluded`, `motionVectors3D`, `reset` ([ProgrammingGuide.md](https://github.com/NVIDIA-RTX/Streamline/blob/main/docs/ProgrammingGuide.md)).
     - **O difícil:** identificar, nos recursos do título, depth, motion vectors e jitter do quadro. Dá para ler da entrada que o título passaria ao FSR4, que pede os mesmos insumos [estimativa].
  3. NIS (escala espacial), como paliativo sem vetores [não avaliado].

---

## 5. Conhecimento NVIDIA D3D12 condensado (verificado nos artigos)

Fontes: série [Advanced API Performance](https://developer.nvidia.com/blog/tag/advanced-api-performance/) e [DX12 Do's and Don'ts](https://developer.nvidia.com/dx12-dos-and-donts) (a série atualiza este último).

- **Memória** ([artigo](https://developer.nvidia.com/blog/advanced-api-performance-memory-and-resources/)):
  - `QueryVideoMemoryInfo` mais `RegisterVideoMemoryBudgetChangeNotificationEvent`;
  - `SetResidencyPriority` alta para RT/depth/UAV;
  - `MakeResident`/`Evict` para ficar no orçamento, com `MakeResident` em lote;
  - criar recursos fora do caminho crítico;
  - recursos placed em `CreateHeap`, atualizações alinhadas a 2 MB, sem churn de criar e destruir;
  - `UpdateTileMappings` na fila de cópia async.
  - O repo não usa `SetResidencyPriority` (grep).
- **PSOs** ([artigo](https://developer.nvidia.com/blog/advanced-api-performance-pipeline-state-objects/)): criar de forma async em workers, começar com PSOs genéricos, evitar compilação em tempo de jogo; o cache em disco do driver ajuda; "miss and update" em vez de biblioteca gigante.
- **Command buffers** ([artigo](https://developer.nvidia.com/blog/advanced-api-performance-command-buffers/)): 5–10 ECL por quadro; 50–80 µs de agendamento por ECL; não bloquear em ECL; não alternar draw/dispatch/cópia com frequência; `ExecuteIndirect` com count buffer; ≤ 1M descritores e ≤ 2K samplers.
- **CPU** ([artigo](https://developer.nvidia.com/blog/advanced-api-performance-cpus/)): gravar listas em várias threads; ECL na própria thread; thread dedicada à criação de recursos; fila de cópia async para uploads.
- **Sincronização e async compute** ([sync](https://developer.nvidia.com/blog/advanced-api-performance-synchronization/), [async compute](https://developer.nvidia.com/blog/advanced-api-performance-async-compute-and-overlap/)):
  - combinar trabalho de banda com trabalho de ALU;
  - menos de 0,2 ms não compensa;
  - fences não são barreiras;
  - prioridade de fila do D3D12 não muda prioridade de warp;
  - não sobrepor trabalho de RT core com trabalho de RT core;
  - mais de duas filas sem HAGS serializa.
- **Async copy** ([artigo](https://developer.nvidia.com/blog/advanced-api-performance-async-copy/)):
  - a cópia async usa um motor dedicado feito para saturar o PCIe;
  - não use a fila de cópia para algo consumido logo em seguida, porque fica serial e custa a troca;
  - não leve cópias local→local para a fila de cópia.
  - Isso é relevante para `texture/ahead.rs` e para a cópia de volta (`VICTORY_RELAY_COPY`).
- **Descritores** ([artigo](https://developer.nvidia.com/blog/advanced-api-performance-descriptors/)):
  - root constants > root CBV/SRV/UAV > tabelas;
  - bindless SM 6.6 aceito;
  - descritores persistentes;
  - root signature 1.1 com `DATA_STATIC_WHILE_SET_AT_EXECUTE`.
- **Shaders** ([artigo](https://developer.nvidia.com/blog/advanced-api-performance-shaders/)):
  - flag all-resources-bound;
  - evitar leitura divergente de CBV (use SRV para dados indexados por thread, como ossos);
  - intrínsecos de wave em vez de groupshared;
  - grupos múltiplos de 32, com 64 costumando ser o ponto ideal.
- **Método de análise:** [Peak-Performance-Percentage](https://developer.nvidia.com/blog/the-peak-performance-analysis-method-for-optimizing-any-gpu-workload/). Comece pelas unidades com maior throughput (SOL%), e não pela ocupação.

---

## 6. Lições de projetos comparáveis (o que ler e por quê)

| Projeto | Arquivo | O que copiar |
|---|---|---|
| **Xenia** (Xbox 360) | [`src/xenia/gpu/shared_memory.h`](https://github.com/xenia-project/xenia/blob/master/src/xenia/gpu/shared_memory.h), [`d3d12/d3d12_shared_memory.cc`](https://github.com/xenia-project/xenia/blob/master/src/xenia/gpu/d3d12/d3d12_shared_memory.cc) | **A cópia da GPU mora na VRAM**: um recurso reservado de 512 MB (`kBufferSizeLog2 = 29`, `CreateReservedResource`), com tiles de 4 MB mapeados sob demanda de heaps DEFAULT (`UpdateTileMappings`) e uploads por um pool de upload (`D3D12UploadBufferPool` mais `CopyBufferRegion`). Watches por página: `RequestRange` sobe o que mudou, `RangeWrittenByGpu` marca o que a GPU produziu e protege. É o modelo para "janelas em VRAM com upload incremental", que funciona no Win10 sem ReBAR. Atenção: tiles em heap aberto de endereço removem o device (`inplace.rs`), então use heaps DEFAULT normais |
| **shadPS4** (PS4) | [`buffer_cache/memory_tracker.h`](https://github.com/shadps4-emu/shadPS4/blob/main/src/video_core/buffer_cache/memory_tracker.h), `region_manager.h`, `fault_manager.cpp`, `video_core/page_manager.cpp` | Bitsets separados para "modificado pela CPU" e "modificado pela GPU" (`IsRegionCpuModified`/`IsRegionGpuModified`, `ForEachUploadRange`/`ForEachDownloadRange`). O download só acontece quando a CPU encosta (`InvalidateRegion` com `on_flush`). Proteção por `userfaultfd`/sinais. **Ideia para o vai-funcionar:** em vez de copiar de volta tudo que os shaders declararam escrever a cada submissão (`land.rs`: 98 MB em ~1.500 pedaços num run), marcar como GPU-modificado e baixar sob falta da CPU. `fault_manager` lê um buffer de falhas escrito pela GPU, análogo a `memory.rs` |
| **vkd3d-proton** | [`libs/vkd3d/memory.c`](https://github.com/HansKristian-Work/vkd3d-proton/blob/master/libs/vkd3d/memory.c), `cache.c`, [README](https://github.com/HansKristian-Work/vkd3d-proton/blob/master/README.md) | Mapeia CUSTOM `WRITE_BACK` → READBACK (host cached), `WRITE_COMBINE`+L1 → GPU_UPLOAD; usa ReBAR para UPLOAD, a não ser que o app já use GPU_UPLOAD (`no_upload_hvv`). `vkd3d_report_memory_budget` (flag `LOG_MEMORY_BUDGET` do `VKD3D_CONFIG`; nome exato da opção não conferido). Cache de pipelines próprio, que torna `ID3D12PipelineLibrary` um "dummy" (`VKD3D_SHADER_CACHE_PATH`, `pipeline_library_app_cache`). Isso sustenta a ideia de não salvar uma biblioteca de 1,5 GB no meio do jogo |
| **RexGlue SDK** (recomp de 360) | `src/gpu/xenos/shared_memory.cpp`, `src/gpu/xenos/d3d12/shared_memory.cpp`, `pipeline_cache.cpp` | Herda o modelo da Xenia num runtime de recompilação, mais perto do VIctory (código nativo mais HLE) |
| **Kelvaroq** (fork VIctory, RTX 5080) | [`documentation.md`](https://github.com/stellarr-studio/kelvaroq/blob/HEAD/documentation.md) | Correções 0038–0040 (`DIRECT_CONSTANTS`, `FILL_WAIT` por bloco, `REBAR_COPIES_MB`, `CARD_BUFFERS_MIN_KB=64`, `ASYNC_SUBMIT`, `QUEUE2`) e 0045 (write watch por reserva). Protocolo de medição: p50/p95/p99 em ≥ 600 quadros pesados (≥ 3.000 draws) em 3 passagens; meta de 33,3 ms p50 e 40 ms p95. PRs upstream [VIctory#2120](https://github.com/Marvisak/VIctory/pull/2120) (placement em VRAM) e [#2130](https://github.com/Marvisak/VIctory/pull/2130) (espelho CPU) |
| **fxceless/victory-fork** | `crates/victory-gpu` (base do vai-funcionar) | Origem de `ASYNC_SUBMIT` (f85101b6), `CARD_BUFFERS_MIN_KB` (c36971b2) e texture read-back (f5c03f3b). README: 4070 12 GB / 32 GB / Win11 a 240–350 ms com `RELAY=2` ou `LATE_WAIT=3 WIDE_BACK=1 CARD_BUFFERS_MIN_MB=1 CARD_BUFFERS_MB=1024` |

---

## 7. Setup de medição (o mesmo em todo experimento)

1. **Rota fixa:** mesmo save, mesma rota do `test-session.ps1` (cela → corredor → loja), 5 min, 3 passagens por variante. Cópias independentes do mesmo cache e da mesma pasta de saves (protocolo Kelvaroq).
2. **PresentMon:** `PresentMon.exe --process_name victory.exe --output_file run.csv --timed 300 --terminate_after_timed`. Colunas: `MsBetweenPresents`, `MsCPUBusy`, `MsCPUWait`, `MsGPUBusy`, `MsGPUWait`, `MsGPULatency` ([README](https://github.com/GameTechDev/PresentMon/blob/main/README-ConsoleApplication.md)).
3. **Nsight Systems** (CPU, filas, memória):
   `nsys profile --trace=dx12,wddm,osrt,nvtx --dx12-gpu-workload=true --dx12-wait-calls=true --wddm-memory-trace=true --gpu-metrics-devices=all --duration=60 -o run victory.exe ...`.
   Mantenha a janela curta (60 s), porque o trace cresce rápido.
4. **Nsight Graphics:**
   - GPU Trace num quadro pesado: throughput por unidade, ocupação de SM, PCIe;
   - Shader Profiler: hot spots, registradores e stalls dos shaders traduzidos.
5. **Logs do runtime:**
   - `VICTORY_PANEL`/F3;
   - `VICTORY_MEMORY_CATEGORIES=1` e `VICTORY_MEMORY_MAP=1`;
   - linha de frame (draws descartados);
   - `VICTORY_COPY_MODE=none` **só para limite de tempo** (a imagem sai errada).
6. **Registro:**
   - driver;
   - versão do Agility;
   - estado de HAGS e ReBAR (GPU-Z);
   - plano de energia;
   - todas as `VICTORY_*` (o `test-session.ps1` já grava o ambiente).
7. **Regra de decisão:** só conta mudança maior que 5% na mediana com p95 sem piorar (`findings.md`: abaixo de ~5% é ruído).

---

## 8. Plano de 2 semanas na 4080

Metas:
- **S1:** mediana ≤ 50 ms (mais de 20 fps) e 30 min sem OOM.
- **S2:** ≤ 33,3 ms (30 fps) na rota.
- **Trilha de 60 fps:** exploratória.

| Dia | Experimento | Hipótese | Mudança | Métrica | Manter se | Reverter se |
|---|---|---|---|---|---|---|
| D1 | E0 Agility | Sem SM 6.6 nada roda no Win10 | Exports mais `D3D12\D3D12Core.dll` 1.620.2 (§1.1) | Log SM 6.6; `translate --validate --pipelines` 100%; mundo carrega | Mundo carrega | n/a (obrigatório). Se der DEVICE_HUNG: DRED mais #71 |
| D1 | E1 Linha de base | 4080 mais 32 GB fica entre a 5060 e a 7900 XTX | `Play-VIctory.cmd` como está (Series X), depois Series S | p50/p95 PresentMon, `MsCPUBusy` × `MsGPUBusy`, memória compartilhada por minuto | Fica a melhor das duas como base | — |
| D2 | E2 Limite superior das cópias | As cópias CPU↔GPU são o teto | `VICTORY_COPY_MODE=none` (só tempo) | Δ p50 contra E1 | Informativo: mostra quanto 4.2–4.4 podem render | — |
| D2 | E3 HAGS on/off | Com 3+ filas, sem HAGS serializa | Alternar HAGS (reinicia) | p50, `MsGPUWait` | Δ ≥ 5% | Δ < 5% (fica o que for mais estável) |
| D3 | E4 VRAM grande | Menos leitura via PCIe → GPU mais rápida | `VICTORY_CARD_BUFFERS_MB=4096`, `VICTORY_CARD_WINDOWS_MB=2048` → 4096 | p50, PCIe read (nsys), W da placa, VRAM usada < 90% | p50 −10% sem perda visual | VRAM estoura, ou p50 não muda |
| D4 | E5 Port 2 | A varredura de descritores custa 15 ms | `VICTORY_PORT_DESCRIPTORS=1` | p50; tempo de "descriptor" no painel; diff de captura | p50 −5% e nenhum slot errado | Textura errada ou preta |
| D4 | E6 Ports 5/1 com 16 GB | Os bugs eram efeito do orçamento de 8 GB | `VICTORY_PORT_CBV=1`, depois `+PORT_CONSTANTS=1` | Estrada visível; sem exceção C++; p50 | Visual igual e p50 menor | Qualquer regressão visual ou crash |
| D5 | E7 Write watch por reserva | `GetWriteWatch` por bloco custa ~5 ms | Portar a ideia do Kelvaroq 0045 em `watch.rs` (uma consulta por reserva por submissão, resposta reaproveitada) | Tempo de "write watch" no painel; p50 | −3 ms ou mais e testes `watch`/`land` passando | Escrita perdida (ruído em textura, #5) |
| D6 | E8 Constantes direto da GPU mais espera por bloco | Fills de constantes esperam voos à toa | Portar a ideia do Kelvaroq 0038 (CBV na cópia host alinhada a 256 B) e `fill_ahead` por bloco de 64 KiB | Número de esperas por quadro; ms de espera | Esperas −50% | Constantes erradas (luz ou sombra estranha) |
| D7 | E9 Guarda de orçamento não local | A morte é estouro do compartilhado | Código em `shared_buffer`/`upload_buffer`: checar `video_memory()[2..3]`, liberar pool e adiar uploads; evento de mudança de orçamento | Soak de 30 min; wddm memory trace | 30 min sem `0x887A0005` | Mais travadas (> 200 ms) do que a base |
| D8 | E10 NVAPI CPU-visible vidmem | Janelas com cópia única na VRAM via ReBAR no Win10 | `QueryCpuVisibleVidmem`; se total > 1 GB, janelas via `NvAPI_D3D12_CreateCommittedResource` com `CPUVISIBLE_VIDMEM` em `window_mapped_for` | Memória compartilhada −GB; p50; listas no card | Compartilhada −1 GB ou mais e p50 igual ou melhor | Total ~256 MB (sem ReBAR) ou regressão |
| D9 | E11 Pipelines | Cache frio e salvamento travam | `victory warm --used`; `LIBRARY_SAVE_S` longo; cache do driver maior; workers 1/4 nos lentos | Draws descartados nos primeiros 5 min; quadros > 200 ms em 15 min | −80% de descartes; nenhuma trava de save > 200 ms | — |
| D9 | E12 THIN no NVIDIA | Os 8 threads de gravação valem ~7 ms | `VICTORY_THIN=1` mais diff de captura de rosto | p50; pele da cabeça presente | Visual ok e −5% | A pele some: abrir bug com captura |
| D10 | E13 GPU Trace da melhor config | Com a CPU resolvida, a GPU vira a parede | Captura; testar `AllResourcesBound` (bit 8) e cortar produtores de RT sem consumidor | SOL% por unidade; ms de GPU | −1 ms ou mais de GPU | Hang ou erro de validação |
| D10 | Soak final | — | Config combinada | 30 min mais 60 min: p50, p95, p99, OOM | Meta S1 batida → vira padrão no `.cmd` | — |

**Trilha de 60 fps** (depois de S2):
1. `VICTORY_FRAME_INTERVAL_MS=16.667` só para medir quanto a CPU aguenta.
2. Verificar se a simulação do título acompanha. Física e animação em 60 podem mudar o comportamento [não verificado].
3. DLSS SR (§4.8) para cortar o custo de GPU.
4. Paralelizar a submissão (§4.2 item 5).

Sem a thread de preparo abaixo de ~14 ms, 60 fps não acontece [estimativa].

---

## 9. O que não consegui verificar

- O orçamento local e não local exato na sua 4080 com Win10 e 32 GB. Os ~15 GB e ~16 GB são estimativas.
- Se o ReBAR está ativo e qual o tamanho de VRAM visível pela CPU para um exe sem perfil NVIDIA.
- Enhanced barriers no Win10 com driver 6xx.
- Se o DEVICE_HUNG do upstream (#71) acontece no vai-funcionar.
- Se os ports 1, 2 e 5 ficam corretos com 16 GB.
- Se o título suporta 60 Hz sem efeitos colaterais.
- Os nomes exatos de algumas telas do Nsight Graphics (Shader Profiler, trava de clock).
- O padrão do cache de shader do driver.
- Todos os ganhos marcados [estimativa]. Os números [medido] vêm de outras máquinas (5060, 7900 XTX, 5080, 4070) e não da sua.
