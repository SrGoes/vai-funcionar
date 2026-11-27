# Para onde vai o quadro na máquina B, e como otimizar (27/11/2026)

**Máquina B:** RTX 5060 8 GB, i5-13400F, 16 GB, driver 621.40.
**Build:** diag da `main` em `5487f38` mais o freio do PR #57.
**Execução:** rota nvidia com `-SmallRam` (720p, upscaler do jogo, texturas até 1024, cabelo), Series S, cache aquecido.

**Fontes:**
- uma linha do tempo de 3 quadros aos 300 s (`VICTORY_TIMELINE`, 167 ms por quadro, cena da prisão);
- amostras de paginação a cada 5 s (WMI: `PagesInputPersec`, `AvailableMBytes`, conjunto de trabalho do `victory.exe`);
- as linhas de frame do log;
- o trecho jogável da rodada `texmax` (depois das 17:45, mais de 1.500 draws por quadro, 124 linhas de frame).

Tudo isso vem de uma rodada por configuração, em cenas que andam sozinhas. Não é medição em ponto fixo.

## O que a placa espera

Trecho jogável, medianas:

| Item | Valor |
|---|---|
| Tempo de quadro | 219 ms (4,6 FPS) |
| Preparo | 113 ms (cópias 40, texturas 26) |
| Espera pela placa | 0,2 ms |
| Cópia de volta | 34 ms |
| Draws por quadro | ~7.300 |

O relógio da placa diz que **ela fica parada 131 ms por quadro** (44% ocupada):
- em 63 ms a nossa thread de envio está trabalhando;
- nos outros 68 ms o jogo ainda não enviou nada, porque as threads dele estão gravando listas dentro dos nossos ganchos.

A CPU limita nas duas pontas.

## Achado 1: a trava `EXACT` domina a gravação das listas

Na cadeia crítica dos 3 quadros, a maior causa é "a long list call": **74 ms por quadro** (81 e 66 ms em dois deles).

Por local de chamada, somando as threads, os maiores são:
- `list::execute_indirect` chamado de `+0x555e97c`: **297 ms por quadro em 417 chamadas** (~0,7 ms cada);
- outros dois `execute_indirect`: 64 ms;
- `dispatch_mesh`: 27 ms.

Dentro dessas chamadas longas, 1.287 dos 2.183 ms são espera. Dessa espera, **1.213 ms (94%) são na trava de `shadow::exact`**:
- vinda de `descriptor::host::buffer_copy`: 632 ms;
- vinda de `descriptor::host::buffer_srv`: 582 ms.

Isso acontece ao montar as tabelas de views de cada draw ou dispatch. Mais 22 ms são `WaitForSingleObject` dentro de `Host::shared_buffer`, chamado de dentro de `exact`.

**Por que:** na NVIDIA o heap de views visível aos shaders tem no máximo 1.000.000 de slots, e o jogo já ocupa ~1 milhão. Por isso os gêmeos raw da AMD não cabem (`raw_twins=false`; eles pedem 2^22 slots). Toda view estruturada que não começa num elemento da cópia do seu buffer ganha uma cópia própria dos seus bytes (`exact`). No mundo:

| Medida | Valor |
|---|---|
| Pedidos à `exact` por quadro | 1.500 a 2.200 |
| Misses por quadro | 300 a 400 (20-200 buffers novos, 100-365 reaproveitados) |
| Cópias guardadas | 3.200 a 4.400 |

Cada miss, com 256 cópias ou mais, **varre o mapa inteiro segurando a trava** (`map.retain`). Para cada cópia faz `Arc::strong_count` e `host::references`, que é um AddRef e um Release COM. São ~350 × 4.000 ≈ 1,4 milhão de visitas por quadro, enquanto 6 a 8 threads do jogo esperam. Depois, ainda dentro da trava, o miss cria o buffer (`CreateCommittedResource` + `Map`) ou zera um buffer reaproveitado.

## Achado 2: a RAM vai para o disco o tempo todo

Durante os 6 minutos da rodada da linha do tempo (70 amostras):
- **9.644 páginas lidas do disco por segundo** na mediana (~38 MB/s), 69 mil no p90 e 222 mil no pico;
- RAM livre de 911 MB na mediana, chegando a 14 MB;
- o `victory.exe` com 5,5 GB na memória física para ~21 GB privados.

Na rodada `texmax`, o mapa de memória mostra para onde vão os 21 GB:

| Item | Tamanho |
|---|---|
| Memória do próprio jogo (views das páginas do jogo, mapeadas) | 7,5 GB |
| Heaps do jogo | 3,1 GB |
| Cópias de janelas da memória do jogo, em memória compartilhada com a placa (fixada na RAM) | ~5 GB |
| Pool de páginas do jogo, reservado | 4 GB |
| Heaps do processo | 2,9 GB |
| Outras alocações | 3,7 GB |

As janelas são uma segunda cópia de memória que o jogo já tem. Elas ficam fixadas na RAM e empurram o resto para o disco. Cada trava fica pior quando quem a segura para esperando uma página.

## O que já existe para isso

| Peça | Onde | Ataca | Estado |
|---|---|---|---|
| `VICTORY_EXACT_LAZY` | PR #56 (Lago, `703b167`, `3d67c68`) | Varredura a cada 64 misses e teste barato primeiro | Na 4080: 9,3 para 12-13 FPS com o resto do kit, e foi o maior ganho. Na máquina B, uma rodada em cutscene não mostrou ganho, mas aquela rodada estourava a memória compartilhada. Precisa ser medido de novo com `-SmallRam` |
| Pedaços de buffers de 32 MB para as cópias | mgrein `982b9f0` (+686 linhas, `arena.rs`) | Tira a criação de recurso do miss; menos memória (hoje cada cópia é um recurso de 64 KB ou mais) | Na 4070: criar e destruir era ~23% do tempo das threads de gravação. Aqui já reaproveitamos buffers soltos, então o ganho deve ser menor |
| Ociosas por 2 s, procuradas no máximo uma vez por quadro | mgrein `54ef67f` (+58 linhas) | O mesmo do `EXACT_LAZY`, por tempo | Concorre com o PR #56; escolher um |
| Slot reescrito com o mesmo recurso não pega nova referência | mgrein `f9bcd8a`, `b1737be` (~110 linhas) | AddRef e Release COM nas tabelas | Não medido aqui |
| Cada thread de gravação guarda as views que fez | mgrein `c263560`, `fc02789`, `6381d40` (~1.100 linhas) | Menos pedidos à `exact` e às travas globais | Risco médio a alto, mesma área do #29 |
| Memória do jogo lida pela placa sem cópia | `VICTORY_INPLACE` (`inplace.rs`), já na árvore | As janelas de ~5 GB e as atualizações delas | Estágio 1: só janelas em memória privada inteira comprometida. O pool de páginas é `SEC_RESERVE` e o host recusa. Abrir por endereço trava as páginas na RAM. Nunca rodou na NVIDIA aqui |

## Plano, por custo e ganho

1. **Medir o PR #56 com `-SmallRam` na máquina B, por linha do tempo** (o build de teste `test/night-smallram` já está compilado: PR #56 mais o PR #57). O critério é a espera na trava `EXACT` dentro das chamadas longas, que hoje é 1.213 ms em 3 quadros, mais o tempo de quadro na mesma cena. Uma abertura, sem matriz.
2. **Tirar a criação e o zeramento do buffer de dentro da trava** (~40 linhas em `shadow::exact`):
   - no miss, soltar a trava;
   - pegar um buffer solto ou criar um, e zerar;
   - pegar a trava de novo;
   - se outra thread já criou a mesma chave, devolver a dela e guardar o nosso como solto.

   Vale com ou sem o `EXACT_LAZY`, e ataca os `WaitForSingleObject` vistos dentro da trava.
3. **Varredura fora do caminho do miss:** levá-la para o passe que já roda uma vez por segundo (`host.rs`, `let_go_idle`), ou no máximo uma vez por quadro, como no `54ef67f`. O miss deixa de visitar o mapa.
4. **Slot reescrito sem nova referência** (`f9bcd8a`, `b1737be`): pequeno, independente, mede-se pela mesma linha do tempo.
5. **Memória (o que destrava de vez a máquina B): janelas sem cópia.**
   - Rodar `VICTORY_INPLACE_PROBE=1` uma vez, para saber o que o driver 621.40 aceita e quanto das janelas está em memória privada.
   - Se o pool de páginas for o que segura (`SEC_RESERVE`), estudar comprometer o pool (`SEC_COMMIT`): custa commit, que aqui sobra (pagefile de 40 GB), e permite `OpenExistingHeapFromFileMapping` sem travar páginas.

   É a maior mudança e a de maior risco (semântica das submissões em voo, `inplace.rs` explica as regras), mas tira 5 GB da RAM e boa parte das atualizações e da cópia de volta.
6. **Por último, a pilha de views por thread do mgrein:** só se, depois de 1 a 3, a `exact` e as travas de `resource.rs` ainda aparecerem na cadeia crítica.

**Expectativa, não medida:**
- tirar a espera na `EXACT` encurta a cadeia crítica em até ~74 ms por quadro naquela cena (de ~167 para ~100 ms, de ~6 para ~10 FPS);
- 15 FPS na máquina B pede também cortar a paginação (item 5).

**Como medir cada passo:** `VICTORY_TIMELINE` nos mesmos horários, mais o amostrador de paginação (`paging.ps1`, WMI; os contadores do `typeperf` têm nomes traduzidos neste Windows em português), comparando a seção "list calls of 50 us or more" e as esperas por local.

## Com o PR #56 (`EXACT_LAZY` e afins): branch `test/smallram-night`, 27/11 à noite
Branch `test/smallram-night` (`7df2b25`): o PR #56 com os commits do PR #57. Rodada com `-SmallRam` e a linha do tempo aos 300, 600, 900 e 1200 s. Uma rodada, cenas que andam sozinhas e parte jogada.

| | Sem o PR #56 (rodada longa `texmax`, 33 min) | Com o PR #56 (20 min) |
|---|---|---|
| FPS no mundo, mediana (linhas com mais de 1.500 draws) | 4,9 | **7,2** |
| Quadro, mediana | 204 ms | 139 ms |
| Draws por quadro, mediana | 5.846 | 6.261 |
| Placa ocupada (relógio do port), mediana | ~44% | **81%** |
| Quedas e falhas de upload | 0 e 296 | 0 e 0 |

**Espera na trava `shadow::exact`** dentro das chamadas de lista longas, somando as threads:

| Momento | Espera | O que mais pesa na cadeia do quadro |
|---|---|---|
| Sem o PR #56, 300 s | 1.213 ms em 501 ms de gravação (2,42 por ms) | as chamadas de lista longas |
| Com o PR #56, 300 s | 975 ms em 641 ms (1,52 por ms) | as chamadas de lista longas |
| Com o PR #56, 600 s | 776 ms | **a placa executando**, 55 ms por quadro |
| Com o PR #56, 900 s (cena pesada, 310 ms por quadro) | 911 ms, mais 242 ms em `resource::texture_at_global` | as chamadas de lista longas, 91 ms por quadro |
| Com o PR #56, 1200 s | 386 ms | **a placa executando**, 120 de 181 ms por quadro |

**Leitura:**
- **O `EXACT_LAZY` corta ~37% da espera na trava, mas não a elimina.** O que sobra é o que ele não toca: a criação e o zeramento do buffer dentro da trava, e a disputa entre as ~2 mil buscas por quadro.
- **Na máquina B, com a trava aliviada, a placa (RTX 5060) vira o limite em boa parte das cenas**, mesmo com render interno de ~853x480. Placa ocupada numa resolução tão baixa aponta para trabalho que não depende de pixels. Os suspeitos:
  - leituras pela PCIe das cópias que ficam na RAM: a 5060 tem PCIe 5.0 x8, metade das lanes da 4080;
  - a fila de cópia;
  - os passes que limitam argumentos de `ExecuteIndirect`;
  - o custo dos shaders traduzidos.

  Próxima medida: `VICTORY_GPU_TIMING`, que cronometra cada lista na placa.
- **Nas cenas pesadas, as travas das threads de gravação voltam a mandar.** O próximo código continua sendo a `EXACT` (criação fora da trava, varredura fora do miss), com a `texture_at_global` em seguida.
- **Na máquina A, os ~70 ms por quadro de "o jogo ainda não enviou"** com a branch sem o PR #56 (comentário do Lago no PR #57) são essas mesmas travas. Lá a placa sobra, então elas são o limite inteiro.

## `VICTORY_EXACT_SPLIT` (acertos com trava compartilhada, buffer criado fora da trava), 27/11 à noite
Commit `cecc179` na `test/smallram-night`. Uma rodada com `-SmallRam` e `VICTORY_AB=30,VICTORY_EXACT_SPLIT`: a chave alterna a cada 30 s no mesmo jogo, começando no mundo aberto. Contam só as fases com mais de 1.500 draws.

| | Ligada (10 fases, 2.172 quadros) | Desligada (11 fases, 2.188 quadros) |
|---|---|---|
| Quadro, mediana das fases | 135,0 ms | 139,2 ms |
| Quadro, média | 142,3 ms | 149,2 ms |
| p90 | 225,7 ms | 213,0 ms |
| Draws por quadro | 7.576 | 7.354 |

**Pares de fases vizinhas (19):** a chave ligada ganha em 12, por 7,5 ms de mediana e 3,8 ms de média.

**Leitura:**
- O ganho é real, mas pequeno: 3 a 5%, com o p90 um pouco pior.
- Duas threads criando a mesma cópia ao mesmo tempo quase não acontece no jogo (~0 por quadro); o teste sintético, com 8 threads e as mesmas views, teve 115 em 3.200.
- Os acertos esperando atrás da criação de buffer não eram o grosso da espera. O que resta deve ser a escrita de cada miss e a varredura, agravadas quando quem segura a trava para numa falta de página (máquina B paginando).
- A chave fica no kit nvidia desta branch. A próxima medida da trava é uma linha do tempo com ela ligada, para ver quem segura a trava escrita.

**Na mesma rodada, as texturas pretas:**
- A VRAM estava a 98% do orçamento (6.412 de ~6.530 MB), e o log disse 31 vezes que a placa não tinha espaço para texturas novas.
- Ficaram 19.375 texturas na fila, subindo a só 27 por segundo (6 MB/s) no mundo.
- Parte da VRAM é das janelas que o `-SmallRam` põe na placa (`VICTORY_WINDOWS_BAR_MB=1536`; ~0,74 GB na abertura de diagnóstico). Elas entram no carregamento, com a VRAM ainda folgada, e depois ocupam o lugar das texturas do mundo.

  **Próximo teste:** `-SmallRam` com `VICTORY_WINDOWS_BAR_MB=0`. As janelas voltam à RAM; a memória compartilhada estava em 4,5 de 7,4 GB, e o freio protege.
- A pressão de RAM do `residency.rs` ligou no quadro 45 (2,86 GB livres) e não desligou mais, porque para desligar exige 5 GB livres. Num PC de 16 GB ela fica ligada a sessão inteira, mas o envio real ficou muito abaixo do teto de 64 MB por quadro. Não é ela que segura as texturas.

## Texturas pretas no mundo: a reserva para render targets e as janelas na VRAM (27/11 à noite)
Mesma branch, rodadas começando no mundo aberto (o save avançou).

**Sintoma:** asfalto, muretas e paredes pretos chapados, enquanto carros, céu, água e árvores saíam normais. A fila de texturas ficou com 13 a 21 mil esperando, e o log dizia que a placa não tinha espaço para texturas novas.

**Causa, em duas partes:**
1. `room()` (`resource.rs`) soma ao uso da placa uma reserva para render targets que ainda não existem (`targets_room_kept`): `VICTORY_CARD_ROOM_FOR_TARGETS_MB` (3.072 MB) menos o que os targets já ocupam. O número foi medido numa resolução maior. Com a saída de 720p do `-SmallRam`, os targets ocupam ~966 MB, e ~2,1 GB ficavam guardados para targets que não vêm. A fila era recusada com ~1,4 GB da placa livres de verdade (5.101 de 6.502 MB, contra o limite de 90%, ~5.850 MB).
2. As janelas na VRAM (`VICTORY_WINDOWS_BAR_MB=1536`, ~0,74 GB) entram durante o carregamento e ocupam o lugar das texturas do mundo.

**Mudança no `-SmallRam`:** `VICTORY_WINDOWS_BAR_MB=0` e `VICTORY_CARD_ROOM_FOR_TARGETS_MB=1152` (o que os targets ocupam em 720p, mais folga).

| Uma rodada de cada, ~10-14 min | Antes (janelas na VRAM, reserva de 3 GB) | Depois |
|---|---|---|
| Capturas com mais de 20% de pixels quase pretos | 10 de 47 | 2 de 25 |
| Pixels quase pretos por captura, mediana e média | 3,3% e 14,5% | 1,4% e 6,7% |
| FPS no mundo, mediana | 6,9 | 6,5 |
| VRAM, mediana e pico | 6.299 e 6.461 MB | 6.587 e 7.019 MB |
| Memória compartilhada, mediana e pico | 4.833 e 5.828 MB | 6.146 e **7.170 MB** |
| Quedas e falhas de upload | 0 e 0 | 0 e 0 |

**Nas capturas:** o asfalto ganhou textura (cinza, faixas de pedestre e marcas), e calçadas, casas e cercas aparecem completas. Na rodada anterior, a mesma rua saía preta. A fila ainda sobe ao trocar de área (pico de 18,5 mil) e desce depois (4,5 mil no mínimo).

**Ressalvas:**
- A memória compartilhada chegou a 96% do orçamento, e a VRAM passou um pouco do orçamento que o sistema dá. O freio segurou (nenhuma falha de upload, nenhuma queda), mas a margem é fina.
- Se a queda de memória voltar, os próximos passos são um teto de textura menor (`VICTORY_TEXTURE_MAX=512`), uma reserva de janelas na VRAM só quando sobrar placa, ou soltar as janelas ociosas mais cedo.
- **Em código, a correção certa é a reserva acompanhar o tamanho dos targets da resolução em uso**, em vez de um número fixo medido noutra resolução.

**A casa escura é outra coisa:** é a luz substituta (`constant:0.8`), com o ray tracing desligado. O asfalto preto do relato R05 / #39 também pode ter uma parte de iluminação: os computes sem tradução `8b437ab6`, `477b52e7` e `17af85f1` caem a cada quadro. Esta rodada mostra que a maior parte era textura que não subia.
