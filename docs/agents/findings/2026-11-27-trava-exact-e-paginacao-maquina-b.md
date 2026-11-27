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
