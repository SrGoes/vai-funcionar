# #30 e #11: desenho da tradução real do defrag em wave32 e classificação das recusas wave64
(claude-gtavi-worker, 27/11. Escrito antes de compilar; o resultado está nos PRs #46 e #50 e na seção do worker em session-state.md.)

## 1. Tradução real do CS_DefragHeap_ComputeRelocations

### O que acontece hoje (confirmado pelo assert)
mask_dword (lower.rs:3973) lê uma máscara de lanes como número. Quando a máscara foi escrita num bloco IR anterior b != cur_ir, ele faz o ballot em at = tail[b] (lower.rs:3997), o que está certo: os lanes que contam são os que rodavam b. Mas ele também faz a troca entre as metades em at (exchange_halves(at, ...), lower.rs:4024). Essa troca chama order_waves(at) -> half_meet_at(at), que reescreve o terminador de um bloco já fechado e move self.cur. O mesmo vale para scc_some (lower.rs:4924/4938) com waveAnyTrue.

### Desenho
Separar o que precisa dos lanes do bloco de origem do que precisa das duas metades juntas:
1. Ballot (mask_dword) ou waveAnyTrue (scc_some) continua em at = tail[b] (ou s.entry). Isso é SSA comum. O bitcode aceita instruções num bloco já terminado ("Terminators are kept apart, so a block can still grow at its end", bitcode.rs:~455), e o push segue os redirects de continues().
2. A troca inteira vai para o bloco corrente: exchange_halves(self.cur, Some(self.cur_ir), half_value). Ou seja, store do dword da metade, order_waves, os dois loads e order_waves de novo, tudo em self.cur.
3. Quem chama usa os valores devolvidos e continua emitindo em self.cur. order_waves já faz self.cur = done quando block == self.cur.

Por que está certo:
- Dominância: b domina cur_ir no IR, porque o valor é usado lá (senão haveria phi). tail[b] tem o terminador de b, então todo caminho de b até cur passa por tail[b], e o valor da metade está disponível em self.cur. O caso s.entry domina tudo.
- Valor uniforme por metade: waveActiveBallot e waveAnyTrue dão o mesmo valor em todos os lanes ativos da onda host. Guardar e trocar depois não muda o resultado; o que importa é quais lanes estavam ativos no bloco do ballot, e isso fica fixado em at.
- Quem precisa estar presente: a troca em self.cur exige as duas metades da onda console em cur_ir. order_waves já exige both_halves(cur_ir) no ramo do half-meet e reached_by_group no da barreira de grupo; exchange_halves exige all_lanes(cur_ir). Isso é mais estrito que hoje (all_lanes(b)), e é a condição certa, porque é em cur que as metades se encontram.

Risco (a), o par de dwords: some sem par extra. O problema só existia se o store e o encontro ficassem em blocos diferentes, com outra troca no meio regravando o par. Com a sequência inteira em self.cur, cada troca vira S = [store par[metade]; encontro; load par[0], par[1]; encontro]. Duas trocas da mesma onda console seguem a ordem do programa. O primeiro encontro de S2 só acontece depois que as duas metades passaram o segundo encontro de S1, e esse segundo encontro vem depois dos loads de S1. Logo o store de S2 não pega leitura pendente de S1. Os half-meets contam gerações por site (half_meet_at, um par de dwords por site e por onda em meet_base) e não reaproveitam a contagem entre visitas de loop. Portanto um par por onda (exchange_pair) basta, como para toda troca que já acontece em self.cur.

Risco (b), a segunda order_waves: fica em self.cur, logo depois dos loads. O _at de exchange_halves:4881 já é order_waves(at), e com at == self.cur o self.cur avança. Nenhuma mudança além de passar self.cur.

Mudança necessária no cache: hoje self.ballots.insert(lo, xy) guarda o par JÁ TROCADO. No desenho novo o par é definido em self.cur (depois do primeiro encontro), que não domina usos em blocos irmãos. Reaproveitar o par ali seria SSA inválido e poderia trazer de volta o "use of a void value". Proposta: ballots: HashMap<Value, V> guarda só o valor da metade (definido em at, que domina); o par trocado vira cache por (lo, cur_ir), ou não tem cache. Custo: um encontro a mais por uso em bloco diferente. Esses encontros contam no meet_cap ("more half-meets than the group memory holds"); medir quantos sites o defrag usa (MEET_SITES já é publicado em lower.rs:2876).

Arquivos e pontos: lower.rs mask_dword 3993-4028 e scc_some 4920-4942 (at para o ballot, self.cur para a troca, cache novo). A correção mínima em order_waves (HALF_MEET_CLOSED) FICA como rede de segurança: qualquer chamada futura com block != self.cur continua recusada em vez de corromper o CFG.

O que eu não sei, a verificar antes de implementar:
- Se all_lanes(cur_ir) é verdadeiro na região wide do defrag (exchange.all vem de all_lane_blocks, lower.rs:1867; wave_all também, 1940). Se não for, exchange_halves recusa com GROUP_BARRIER_DIVERGENT e a correção não destrava o programa: é recusa limpa, mas sem ganho.
- Quantos usos de máscara em blocos diferentes o defrag tem. Isso define o custo em encontros e se cabe no meet_cap.

### Plano de teste de equivalência (não só "não entra em pânico")
Não existe hoje um harness que execute DXIL traduzido na GPU nos testes. O único interpretador é o do otimizador de bitcode (bitcode/opt.rs:1749), sobre IR do próprio bitcode, não sobre ISA do console. Proposta em três níveis:
1. Estrutura (victory-shader, sem GPU): o teste sintético do #30 passa a esperar Ok, wave 32, half_meets >= 2, nenhum group barrier dentro do bloco divergente (como em half_meet_orders_a_128_thread_lds_region), mais uma checagem do Module: todo bloco alcançável tem terminador e nenhum uso é de valor void. Hoje isso só é afirmado em write(), com assert; extrair um validate() que devolva Err serve aos dois casos.
2. Execução na GPU (victory-gpu, teste que só roda com dispositivo e Agility SM 6.6, imprimindo "skipped" sem eles): traduzir 3 programas sintéticos wave64 (group 128, 2 ondas console):
   a. v_cmp_gt_u32 vcc, s0, v0; saveexec; branch wide; dentro, v_mov v1, vcc_lo / vcc_hi; buffer_store por thread. Esperado na CPU: para a onda w, bits dos lanes l com 64w+l < s0. Rodar com s0 = 0, 20, 40, 64, 100, 128: metade baixa cheia e alta parcial, onda 1 fora da região (é o caso que exige half-meet), e as duas dentro.
   b. Igual, mas com o SCC de s_and_b64 lido como dado (caminho scc_some). Esperado: qualquer bit != 0 na onda console.
   c. O mesmo uso dentro de um loop de N visitas (s1 = N), checando que cada visita reencontra (gerações) e que o valor é o mesmo em todas.
   Comparar o UAV lido de volta com a expectativa da CPU, bit a bit. Também ler os três dwords de tally (meet_tally) e conferir que meetings == visitas esperadas e que nenhuma espera estourou.
3. O defrag real: não temos referência de execução no console. O que dá para fazer: (i) passar pelo nível 1 com o programa real (o teste carrega o contêiner 02384a60fda6f2d0 de shaders.rpf se VICTORY_GAME_DIR existir, e senão imprime "skipped"); (ii) no jogo, o resultado do ComputeRelocations é a tabela de realocação do heap: comparar uma captura (VICTORY_PIPELINE_BOOK / dumps do buffer de saída) com VICTORY_DEFRAG=0 (dispatch descartado). Isso só mostra que ele passa a escrever algo coerente, não que é idêntico ao console. Vou deixar isso explícito no PR.

## 2. #11 e os 37 "no wave32 lowering for this wave64 program"

### Onde a recusa nasce
A mensagem é montada só em emit/mod.rs:771. Ela aparece quando a lowering TERMINA com sucesso, mas com wave 64 (s.wave_size == Some(64)), num host de no máximo 32 lanes, e todas as novas tentativas falharam. Não é um meet_refuse. A causa real é a decisão de onda em lower.rs:1818-1871, e o VICTORY_WAVE_WHY=1 imprime "WAVE64-WHY group=.. lds=.. [motivos]" (lower.rs:1874-1909) com estes motivos possíveis:
- M1 v_mbcnt, M2 v_readlane, M3 v_writelane, M4 ds_(b)permute: cruzam as metades.
- M5 "mask as data" (reads_mask_as_data, lower.rs:1560) e M6 "lane-constant mask" (lower.rs:1346).
- M7 "LDS in a group over 32", mais o subcaso: "an indirect exit" (all_lane_blocks devolve None) ou "an LDS read and write in one lane-branch region" (lds_meetings devolve None, lower.rs:1006). Esse subcaso é a família do defrag.
- Também st.wide não vazio força crosses (lower.rs:1827).

Fora isso, um crosses com all_lane_blocks válido e LDS ok vira wave 32 com exchange; só fica em 64 quando lds_ok é falso ou all_lane_blocks é None.

Novas tentativas em ordem (mod.rs:532-750), cada uma visível com VICTORY_WAVE_WHY=1 como "WAVE64-RETRY <forma>: <resultado>":
R1 without early-outs, R2 without skips (mod.rs:603-615); R3 without skips, scalars followed (só com VICTORY_WAVE_DEEP_SKIPS=1); R4 widen, os ramos por lane tomados pela onda inteira (só com VICTORY_WAVE_WIDE=1, ou para os digests de scene_kernel em mod.rs:625-640, o defrag ou driver_loop); R5 local waves, cada onda host sozinha (exato só para o idioma de alocação, lower::wave_local); R6 local waves APPROXIMATE (any mask), desligado com VICTORY_WAVE_LOCAL=idiom. Depois disso, threading::constant_edges (mod.rs:783) tenta de novo com arestas constantes.

Em disco não achei a lista dos 37 nem nenhum log WAVE64-WHY: os logs de victory shaders em %TEMP% são do pânico. A classificação por programa PRECISA de uma rodada de victory shaders com VICTORY_WAVE_WHY=1 (e VICTORY_DEFRAG=0, até a correção do #30 entrar) e o log salvo fora do repo. Sem isso, qualquer contagem por motivo seria inventada.

### Ordem de ataque proposta (impacto visual provável; confirmar com WHY)
1. CS_TransmissionVolumetrics_Tiled e _LocalLights (água e volumetria). Já estão em scene_kernel (mod.rs:635-638), então o R4 widen roda para eles sem a chave. Se ainda estão entre os 37, o widen falhou: ler o WAVE64-RETRY "wave-wide branches". Hipótese: M7 com LDS read+write na mesma região, a mesma família do defrag, e aí o desenho da seção 1 (half-meets para scene_kernel) seria o caminho. Confirmar com WHY.
2. vfx_triangleClip (não achei esse nome no código; precisa do hash ou nome no log WHY). Partículas: impacto visível, mas não bloqueia a cena.
3. CS_CollisionRaytraced (também em scene_kernel): afeta RT e colisão visual, não a física da simulação (a física roda na CPU do título; a confirmar).
4. Os de M1-M4 isolados (mbcnt/readlane/permute sem LDS divergente): esses têm conversão exata via exchange quando all_lane_blocks é válido. Os que sobram têm "an indirect exit" (all_lane_blocks None) e pedem trabalho em structure, não em lower.
5. Os R6 (approximate) devem ficar por último e marcados: o norte do projeto é imagem igual à do console, e "approximate" quebra isso por definição.

Para executar quando liberar: target\diag\victory.exe shaders <shaders.rpf> --jobs 4 com VICTORY_WAVE_WHY=1, guardando um log por programa, e depois agrupar por "WAVE64-WHY [...]" e pelo último WAVE64-RETRY. Uns 2 minutos, sem abrir o jogo.
