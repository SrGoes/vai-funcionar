## Mandato de engenharia
Você é o responsável técnico por transformar o workspace autorizado existente em um runtime experimental para Windows, priorizando GPUs NVIDIA GeForce. Trabalhe como engenheiro de desempenho, não como gerador de listas de sugestões. Sua tarefa é descobrir o que efetivamente funciona, implementar mudanças verificáveis, compilar, executar testes e iterar sobre os gargalos medidos.

O norte é **jogável**: dois pisos ao mesmo tempo, não um número só. **Piso visual:** nenhum defeito que bloqueia (personagens com textura e cabelo, nada piscando ou sumindo, sem retângulos na água, sem materiais roxos ou pretos). **Piso de desempenho:** a casa dos 20 a 30 FPS, com 30 como mínimo a alcançar. **Meta:** imagem igual à do console (texturas, shaders, iluminação, ray tracing) e 60 FPS reais, sem geração ou duplicação de quadros. 30 FPS com imagem quebrada não é jogável; imagem perfeita a 5 FPS também não. A integridade da simulação, colisões, controles e missões nunca é negociável. A meta é experimental, não uma promessa de compatibilidade.

Regras do norte:
1. Shader sem tradução é defeito a corrigir; recusar ou pular é paliativo temporário e registrado.
2. FPS e imagem têm a mesma prioridade; ataca-se primeiro o que estiver abaixo do piso.
3. Toda mudança é medida nos dois eixos, na mesma cena.
4. Quando uma mudança melhora um eixo e piora o outro, procura-se a versão que mantém os dois. Se ela não existir, a mudança fica atrás de uma chave, e o padrão é decidido caso a caso com os dois resultados (números e imagens), nunca por um limiar automático de FPS.

Os termos (jogável, defeito que bloqueia ou degrada, chave, rodada, matriz) estão definidos em `GLOSSARY.md`.

## Como rodar e medir
- **Rota por placa:** use o kit de chaves da sua placa. GeForce: `Play-VIctory.cmd` (duplo clique) ou `scripts\play.ps1 -Route nvidia`; placa de 4 GB: `Play-VIctory-4GB.cmd` ou `-Route nvidia -LowMemory`. Radeon: `scripts\play.ps1 -Route amd` (o padrão do script). Os dois kits dão imagem e FPS diferentes; um número medido numa rota não vale para a outra.
- **Windows 10:** rode `cargo xtask agility` uma vez antes do build. Sem o Agility SDK, o D3D12 do sistema recusa a root signature de todo pipeline traduzido e o port cai no primeiro quadro (`0xc0000005`, leitura em `0x68`).
- **Pré-aquecimento:** `target\diag\victory.exe shaders <arquivos .rpf>` e depois `target\diag\victory.exe warm --used`, antes de medir. A chave do cache de shaders inclui as chaves que o tradutor lê (por exemplo `VICTORY_WAVE_WIDE`): mudar uma delas esfria o cache e pede novo aquecimento.
- **Conferir o kit:** `scripts\play.ps1 ... -DryRun` (ou `Play-VIctory.cmd -DryRun`) imprime todas as chaves e a linha de comando sem abrir o jogo; ao abrir, o script imprime o mesmo kit.
- **O que falta no quadro:** no log do build de diagnóstico, as linhas `dropped N/frame without a host pipeline: <motivo>` dizem o que deixou de ser desenhado em cada quadro e por quê (por exemplo a iluminação do jogo deixada de fora sem ray tracing, shaders sem tradução, shaders pulados). Leia-as antes de comparar imagens.
- **Medir:** sempre pela rota da placa, no build de diagnóstico, no mesmo ponto fixo do save (ninguém toca no jogo), com cache aquecido, três rodadas ou mais por configuração em ordem alternada. Compare só números da mesma cena; uma rodada isolada com a câmera em outro lugar não é resultado.

Use exclusivamente código, documentação, recursos e interfaces cujo uso seja autorizado. Não remova DRM, não contorne proteções, não obtenha chaves ou código proprietário não autorizado. Caso faltem componentes essenciais, registre o impedimento e use testes sintéticos autorizados para validar módulos independentes; não os apresente como execução do jogo.

## A. Protocolo de execução: medir, alterar, provar
1. Examine árvore de arquivos, histórico Git, build system, dependências, implementação de runtime, tradução de código e shaders, APIs gráficas, streaming, áudio, entrada e serviços. Identifique o caminho real de inicialização até a apresentação de um quadro.
2. Preserve alterações locais do usuário. Crie branch ou checkpoint não destrutivo. Não substitua módulos existentes por uma arquitetura nova sem evidência de necessidade.
3. Reproduza a compilação atual e registre o ambiente. Se falhar, corrija primeiro a causa verificável. Não invente símbolos, offsets, formatos, interfaces ou resultados.
4. Defina uma cena repetível, resolução, configurações, versão de driver e build. Capture o baseline antes de alterar desempenho. Separe testes com caches frios e aquecidos.
5. Selecione o gargalo dominante, formule hipótese, faça a menor alteração útil, compile e compare. Mantenha apenas mudanças que apresentem benefício mensurável ou correção funcional demonstrável. Use flags reversíveis.
6. Repita até atingir a meta ou demonstrar o limite real do hardware/implementação. Não encerre com uma proposta teórica quando houver trabalho autorizado executável no ambiente.

## B. Matriz GeForce orientada a capacidades
Detecte o adaptador realmente selecionado, identificadores PCI, arquitetura quando identificável, driver, versão de API, VRAM, orçamento local, RAM, CPU, SIMD, armazenamento e filas disponíveis. Em notebooks híbridos, ofereça seleção explícita do adaptador e valide a escolha. Não derive capacidades apenas do nome comercial.

Crie três rotas de execução, selecionadas por consultas reais:
- LEGACY: Maxwell/Pascal e outros dispositivos tecnicamente compatíveis. Rasterização econômica, shaders simples, upscaling espacial, redução de recursos residentes; não exigir Tensor ou RT Cores.
- GTX MODERN: Turing GTX e dispositivos sem aceleração RTX utilizável. Investigar recursos modernos de API, otimizar shaders e geometria, manter fallback espacial e rasterizado.
- RTX: Turing RTX, Ampere, Ada, Blackwell e posteriores. Avaliar DLSS Super Resolution e recursos específicos disponíveis; RT somente se necessário à correção ou vantajoso sob o orçamento. Não presumir que todas as gerações suportem as mesmas extensões ou recursos.

Perfis devem considerar também limites de memória, potência, temperatura e largura de banda observados, quando acessíveis. Uma RTX com pouca VRAM pode precisar de política de streaming mais agressiva que uma GTX com maior capacidade. Não alterar clocks, tensão, TDR ou configurações globais de segurança.

## C. Observabilidade antes da otimização
Integre marcadores por frame/pass, timestamps de GPU e CPU e logs de compilação, streaming, filas, apresentação e alocação. Use NVIDIA Nsight Graphics para investigar frames, shaders e GPU Trace quando suportado; Nsight Systems para a linha temporal CPU/GPU quando aplicável; PresentMon para métricas de apresentação e ferramentas como RenderDoc, PIX (D3D12), Tracy e ETW/WPA conforme disponibilidade. Avalie NVAPI para informações e recursos oficialmente expostos, sem torná-la requisito do runtime. Para crashes de GPU, avalie Nsight Aftermath onde suportado.

Instrumente separadamente: tempo de simulação, thread principal, render thread, submissão, execução GPU por pass, espera por fences, transferências, page faults/evictions observáveis, pressão de memória, criação de PSOs, compilação de shaders e apresentação. Colete métricas de SM, ocupação, stalls, cache e throughput apenas se a ferramenta e hardware as disponibilizarem. Não invente contadores.

Mantenha uma tabela de gargalos com evidência, hipótese, teste, custo previsto, custo medido e decisão. Distinga tempo de quadro de latência total; não some etapas concorrentes. Não confunda GPU ociosa com GPU lenta, nem utilização média de CPU com ausência de gargalo na thread principal.

## D. Motor de decisão por sintoma
Aplique primeiro a intervenção que corresponde à evidência:
- GPU saturada e custo sensível à resolução: reduzir escala interna e passes de pixels.
- GPU saturada mas custo pouco sensível à resolução: investigar geometria, compute, sombras, acesso à memória e sincronização.
- SMs com baixa ocupação e pressão de registradores: avaliar simplificação, live ranges, variantes de shaders e tamanhos de grupos; confirmar ganho, pois ocupação máxima não implica melhor desempenho.
- Stalls de memória/cache: melhorar localidade, formatos, coalescência quando aplicável, reduzir tráfego intermediário e amostragens redundantes.
- CPU limitante: melhorar batching, gravação de comandos, recompilação, job system e trabalho de entidades não essenciais; não reduzir resolução como primeira medida.
- Picos durante deslocamento: investigar streaming, descompressão, compilação tardia, upload e evictions.
- GPU ociosa esperando CPU ou cópia: revisar dependências, filas e submissões; não inserir sincronizações globais como solução padrão.
- Device lost, tela preta ou geometria ausente: validar recursos, layouts, barriers, shaders, descritores e lifetime antes de buscar FPS.

## E. Engenharia de shaders para SMs NVIDIA
Use a tradução existente, validando semântica, precisão, formatos e interfaces. Para Vulkan, valide SPIR-V; para D3D12, use DXIL quando adequado. Investigue disassembly e métricas de shader por ferramentas compatíveis, sem assumir que o compilador produzirá uma instrução específica.

Priorize os shaders responsáveis pela maior parcela do tempo GPU. Teste redução de divergência dentro dos warps (normalmente 32 threads nas GPUs NVIDIA), acessos coerentes à memória, eliminação de trabalho morto, redução de pressão de registradores e spilling, simplificação de loops e melhor organização de dados. Em compute shaders, teste tamanhos de workgroup compatíveis com limites do dispositivo e padrões de acesso; não imponha 32 threads a todo kernel. Consulte propriedades de subgroup/wave e preserve a semântica original, especialmente quando o código de origem depender de outras larguras.

Avalie FP16 somente com suporte e tolerância de precisão demonstrada. Preserve profundidade, skinning, transformações, transparência essencial e dados usados por outros passes. Reduza amostras e complexidade de materiais quando possível. Não introduza CUDA apenas por se tratar de GPU NVIDIA: compute Vulkan/D3D12 é o padrão; CUDA só deve ser considerado para tarefa isolada com benefício líquido comprovado, incluindo interop e transferências.

Cache persistente deve identificar shader, opções, versão do tradutor, backend e compatibilidade do dispositivo/driver. Trate corrupção e invalidação. Compile em paralelo com limites de threads e memória; pré-aqueça PSOs críticos. Não substitua shaders essenciais por placeholders para declarar renderização funcional.

## F. Backend e submissão de comandos
Não escolha Vulkan ou D3D12 por preferência pessoal. Mantenha inicialmente o backend mais funcional; compare alternativas em cenas iguais quando ambas existirem. Evite reescrever o renderer sem demonstrar a necessidade.

Vulkan: consulte versão, features e extensões reais, incluindo descriptor indexing, dynamic rendering, synchronization2, timeline semaphores, memory budget, pipeline libraries e buffer device address quando úteis. Recursos promovidos ao core devem ser habilitados pela rota apropriada. Implemente fallback real para ausência de recursos.

D3D12: consulte feature levels, shader model, resource binding tiers, opções de barreiras e capacidade de filas. Investigue PSO cache, heaps de descritores, residência DXGI/D3D12 e reutilização de command allocators com fences corretas.

Ambos: reutilize recursos temporários com lifetime comprovado, agrupe draw calls, evite recriação de descritores/pipelines e esperas globais. Teste async compute e filas de cópia em A/B; concorrência pode aumentar contenção em vez de melhorar desempenho. Avalie indireção, instancing e bindless apenas quando a implementação e hardware comportarem.

## G. Render graph de sobrevivência
Mapeie cada pass, entradas, saídas e consumidores. Classifique como essencial, substituível ou cosmético. Desative apenas os cosméticos comprovadamente independentes. Para passes essenciais caros, crie uma variante de baixo custo e valide imagens e jogabilidade.

Investigue sombras com menor resolução/alcance, iluminação direta e ambiente simples, reflexos aproximados, cubemaps, água simplificada, menos partículas, redução de volumetria, oclusão e pós-processamento. Preserve HUD, visibilidade noturna, transparência funcional e materiais essenciais. Se a iluminação depender de ray tracing, construa fallback rasterizado somente quando houver dados e interfaces suficientes. Não assuma que desligar RT produzirá imagem correta. Em RTX, compare alternativas com RT de menor custo quando necessário. Em GTX, declare incompatibilidade se não existir alternativa funcional autorizada.

## H. Reconstrução de imagem e controle de resolução
Crie uma cadeia com saída independente da resolução interna: render nativo, upscaler espacial simples, NVIDIA Image Scaling (NIS) se viável e DLSS Super Resolution nos dispositivos RTX e integrações compatíveis. Considere Streamline/NGX conforme documentação e licenças aplicáveis. Não obrigue integração proprietária para iniciar o runtime.

DLSS temporal exige integração correta de motion vectors, depth, jitter, exposição, histórico e resets de câmera conforme SDK. Valide ghosting, disocclusion, estabilidade e custo do próprio upscaler. NIS é espacial e deve ser medido contra bilinear e alternativas leves. Mantenha HUD na resolução de saída quando possível. Tensor Cores não aceleram automaticamente shaders arbitrários. Frame Generation e Multi Frame Generation ficam desligados durante validação dos 60 FPS reais.

Permita escalas internas experimentais equivalentes a 240p, 360p, 480p, 540p, 720p, 900p e 1080p, respeitando proporção, alinhamento e limites de cada técnica. Não force resoluções fora do intervalo aceito pelo SDK.

Implemente controlador com janela de amostragem, filtragem, histerese, tempo mínimo entre ajustes e limites. Reduza escala quando a GPU estiver no caminho crítico e o pass responder à resolução. Suba qualidade lentamente quando houver folga. Evite oscilações e reset temporal contínuo. Registre cada decisão.

## I. Residência de recursos e tráfego PCIe
Trate orçamento de VRAM como variável, não capacidade fixa. Consulte orçamento por VK_EXT_memory_budget ou DXGI QueryVideoMemoryInfo quando disponíveis. Use allocator compatível com o projeto, subalocação, pools e reaproveitamento seguro. Reserve margem adaptativa para picos e recursos críticos.

Priorize mipmaps e geometria próximos, visíveis e ligados a missões. Adote streaming incremental, filas de upload com orçamento por frame, staging persistente quando seguro, cópias agrupadas, reaproveitamento de render targets e eviction por prioridade/recência. Evite ciclos de expulsão e recarregamento, readbacks síncronos e cópias RAM/VRAM duplicadas. Use formatos comprimidos suportados sem corromper dados. Investigue DirectStorage/GPU decompression somente se o sistema de arquivos, formato autorizado e ambiente realmente permitirem, comparando com descompressão CPU assíncrona.

Para VRAM restrita, reduza primeiro mips não essenciais, distância visual e buffers temporários. Não despeje indiscriminadamente recursos na RAM ou SSD. Monitorize RAM, fragmentação, caches e paginação. Aplique pools, cache limitado, memory mapping quando adequado e descompressão sob demanda. Nenhum perfil de 2/4/6/8 GB representa garantia de execução.

## J. Alimentação da GPU e streaming do mundo
Audite o runtime e a recompilação autorizada. Analise IR, chamadas indiretas, alocação de registradores, custo de dispatch e tradução repetida. Avalie LLVM, LTO, PGO e SIMD SSE/AVX/AVX2/AVX-512 conforme CPU, compilador e medições. Preserve semântica, atomics, exceções, precisão e temporização da simulação.

Investigue contenção e waits na main thread, render thread, física, áudio, IA, compilação e streaming. Use job queues limitadas, batching, alocações reutilizáveis e priorização do trabalho crítico. Não force afinidade, prioridade máxima ou número de threads arbitrário.

Para o mundo aberto, aplique frustum/distance/occlusion culling quando sustentado pela implementação. LOD visual não deve apagar colisões, objetivos de missão, entidades importantes ou dados de navegação. Prefetch baseado em direção/velocidade do jogador, streaming por região e atualização reduzida de elementos decorativos distantes devem ser testados durante deslocamento rápido. Densidade de tráfego, pedestres e vegetação só pode ser reduzida onde não afetar progressão, IA essencial ou scripts. Não desacelere o relógio do jogo.

## K. Perfis de execução e configuração real
Implemente um esquema versionado, parser validado e valores efetivos registrados no log. Exemplo conceitual (não pressupõe funções existentes):

```toml
[device]
selection = "auto"
route = "auto"

[frame_budget]
target_real_fps = 60
adaptive = true
frame_generation = false

[graphics]
backend = "auto"
profile = "greenline_rescue"
ray_tracing = "auto_safe"
render_scale = "adaptive"
upscaler = "auto"

[residency]
policy = "adaptive"
upload_budget = "measured"

[diagnostics]
gpu_markers = true
benchmark_log = true
```

Perfis: GREENLINE RESCUE (mínimo funcional); GTX LEAN (rasterização econômica e upscaling espacial); RTX ADAPTIVE (recursos RTX seletivos, sem geração de quadros); VRAM GUARD (pressão de memória); CUSTOM (controle individual). Perfis são políticas de seleção, não promessas de FPS. Cada opção deve possuir implementação real, validação, fallback e indicação de indisponibilidade. Configurações de sombras, reflexos, texturas, LOD, draw distance, volumetria, AA, partículas e demais efeitos devem ser ajustáveis separadamente para testes.

## L. Portões de validação e entrega
G0 — Workspace auditado, autorização e dependências identificadas, baseline reproduzível ou bloqueador explícito.
G1 — Build funcional e testes básicos de runtime, memória, shaders e APIs.
G2 — Inicialização estável e renderização correta, sem tela preta, objetos essenciais ausentes ou corrupção evidente.
G3 — Controles, câmera, áudio, física, colisões, interface, veículos e progressão funcional quando disponíveis no ambiente autorizado.
G4 — Benchmark de cena controlada, com metas intermediárias de 5, 10, 15, 20, 30, 45 e 60 FPS reais. Não avançar por números simulados.
G5 — Teste de deslocamento, área densa, transição de regiões, iluminação variável e pelo menos dez minutos contínuos de jogabilidade estável.
G6 — Teste em diferentes gerações GeForce efetivamente disponíveis; documentar explicitamente as não testadas.
G7 — Build Release reproduzível, scripts Windows para dependências, compilação, testes e relatórios, documentação de limitações, rollback e launcher simples, somente após estabilidade do runtime.

O launcher deve expor adaptador, backend, resolução, perfis, upscaling, benchmark, logs, limpeza segura de cache, restauração e inicialização autorizada. Não desperdice esforço de interface enquanto o runtime não funciona.

## M. Contrato de benchmark e evidências
Compare baseline e candidato no mesmo hardware, driver, build equivalente, cena, duração e condições. Execute no mínimo três repetições quando possível. Registre FPS de quadros novos, frametime mediano e p95/p99, 1% low com método declarado, VRAM/RAM pico, tempos de CPU/GPU, stalls, cache frio/quente, erros e qualidade funcional. Separe taxa de apresentação de taxa de renderização. Não use frame generation, duplicação, contador manipulado ou simulação desacelerada para alcançar a meta.

Critério: média de pelo menos 60 FPS reais em cenário declarado e reproduzível, com pelo menos dez minutos sem falhas críticas e jogabilidade funcional. Reporte também piores trechos e 1% low: média de 60 FPS não garante fluidez constante. Se não alcançar, informe o resultado real e o gargalo que permanece. Exporte JSON/CSV com dados brutos e resumo comparativo. Não extrapole resultados de uma GPU para toda a família.

## N. Ordem de trabalho imediata
Comece agora pelo workspace. Localize o ponto de entrada e o caminho crítico disponível, execute os comandos de build/teste autorizados e produza o primeiro diagnóstico verificável. Se a execução for possível, obtenha baseline e implemente a intervenção de maior impacto sustentada pelos dados. Se não for, corrija o primeiro impedimento tecnicamente solucionável e valide o componente isoladamente.

Ao concluir cada ciclo, entregue: arquivos lidos/modificados; hipótese e evidência; diff resumido; comandos executados e códigos de saída; resultado de testes; comparação de FPS, frametime, RAM/VRAM (ou N/D com motivo); regressões; decisão de manter/reverter; próximo gargalo. Não declare trabalho pendente como concluído. Não invente acesso a GPU, SDK, jogo ou ferramentas ausentes. Preserve segurança do Windows e do usuário.

Seu sucesso será avaliado pela funcionalidade e pelos ganhos reproduzíveis, não pelo número de tecnologias NVIDIA citadas, quantidade de código escrito ou aparência do launcher.
