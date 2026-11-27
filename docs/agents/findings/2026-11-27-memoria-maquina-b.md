# Memória na máquina B: quem ocupa os 21 GB e o que dá para cortar (27/11/2026)

**Máquina B:** RTX 5060 8 GB, 16 GB de RAM, pagefile de 40 GB.

**Fontes:**
- o mapa de memória do log (`copystat.rs` `log_memory_map`: varredura do espaço de endereços; "na RAM" é amostrado em 1 página a cada 16);
- as linhas de memória do log;
- a rodada longa `-SmallRam` (`texmax`, 33 min, 17:23-17:56 UTC);
- as amostras de paginação da rodada da linha do tempo;
- a leitura do código.

Nada aqui foi medido em ponto fixo.

## Onde está a memória

Na rodada longa:

| Fatia (nome no mapa) | Comprometido | Na RAM | O que é |
|---|---|---|---|
| Views das páginas do jogo (mapeadas, endereços de console) | 4,0-7,6 GB (oscila: o jogo mapeia e desmapeia) | ~0,6 GB | A memória física do próprio jogo (`xmem`: uma seção de 6,1 GB pedida de uma vez) |
| Heaps do jogo (privados, endereços de console) | 3,1 GB | ~0,45 GB | Os `XMemVirtualAlloc` do jogo |
| Write-combined (buffers compartilhados com a placa) | 7,4 → 8,2 GB | 0,4-1,5 GB | Heaps de upload. Em grupos de 64 MB: 71 a 80 deles (4,4-5 GB). **Dono não identificado** (ver abaixo) |
| Heaps do processo (segmentos de 15,8 MB) | 2,6 → 3,4 GB | ~0,7 GB | Blocos pequenos nossos, do runtime e do driver. **Crescem ~25 MB por minuto** |
| Alocações de 4 MB | 3,7 GB (rodada `guard720`) | 73 MB | Pela leitura, os buffers `before` das cópias que os shaders escrevem: reservam a cópia inteira e só tocam o que é escrito. Pesam no commit, não na RAM |
| Outras alocações privadas de 1 MB ou mais | 3,7 GB | 0,6 GB | Vários donos (39 de 32 MB, 130 de 8 MB...) |

**A conta que importa:** o jogo tem ~10 GB de memória própria e só ~1 GB dela fica na RAM. O resto foi para o disco, e cada vez que o jogo encosta nela é uma falta de página. Na rodada da linha do tempo a mediana foi de 9.644 páginas lidas por segundo, com a RAM livre em 911 MB e chegando a 14 MB. O que disputa a RAM com o jogo é o port: memória compartilhada com a placa (5,5-6,5 GB em uso pelo DXGI, fixada) e os heaps do processo.

## O que se sabe de cada alavanca

### 1. As janelas: uma segunda cópia de memória que o jogo já tem
- **O que são:** cópias de 4 MB (`shadow.rs` `WINDOW`) da memória do jogo fora de buffers, em `shared_buffer`: heap custom write-back no L0, que conta no grupo compartilhado do DXGI.
- **Quanto:** 3,1 a 5 GB presas por listas ou pela passada do heap, e 2,3 a 3,7 GB copiados alguma vez ("memory, what holds it").
- **A grade fina** (`VICTORY_WINDOW_KB=256` com `TAILS=1`) não reduziu o total (4,95 GB em 8.653 janelas): os bytes são de fato lidos.
- **Na VRAM** (`VICTORY_WINDOWS_BAR_MB`): a placa aceita (`supported=true`), mas o código só põe janelas lá enquanto a placa usa menos de 80% do orçamento próprio, e com `-SmallRam` ela fica em ~95%.

### 2. Ler a memória do jogo no lugar, sem cópia
- **A porta 1** (`port.rs`, `VICTORY_PORT_CONSTANTS`) já faz isso para os 185 MB do heap gráfico baixo do jogo. Ela mapeia uma segunda view das mesmas páginas físicas, num endereço do port, e a abre como heap da placa (`OpenExistingHeapFromAddress1`). Nada é enviado, buscado de volta ou comparado. No kit nvidia ela está desligada por causa do #15 (exceção de C++ com `PORT_CBV`).
- **O `VICTORY_INPLACE=1`** (`inplace.rs`) faz o mesmo para as janelas em memória privada comprometida inteira, que vem dos heaps do jogo.
- **O pool grande fica de fora.** O pool de páginas é `SEC_RESERVE`, porque o jogo pede 6,1 GB de uma vez e comprometer isso falhava em PCs ocupados (erro 1450, `xmem.rs:660`). O host recusa abrir uma seção reservada.
- **Onde está a maioria das janelas:** não se sabe. O censo (`VICTORY_INPLACE_PROBE=1`) classifica cada janela em uma de cinco categorias:
  - privada inteira;
  - uma tira contígua de uma seção (um buffer posto);
  - páginas espalhadas;
  - com páginas desmapeadas;
  - outra.

  Ele nunca rodou aqui. Rodava aos 30 e 150 s, que nesta máquina ainda são menus.
- **As regras que limitam** (comentário do `inplace.rs`):
  - abrir por endereço trava as páginas na RAM, e o jogo não consegue desmapear nem proteger o que está aberto. A porta 1 contorna isso abrindo uma view própria;
  - mapear blocos de páginas (tile mappings) num heap desses derruba o dispositivo. Então só serve uma janela cujas páginas formam uma tira contígua da seção;
  - `OpenExistingHeapFromFileMapping` numa seção `SEC_COMMIT` não trava nada.
- **Por que mesmo uma janela travada economiza:** as páginas travadas são as que o jogo já usa, enquanto a cópia é memória a mais. Cada janela posto no lugar tira 4 MB do grupo compartilhado e as atualizações e a cópia de volta dela.

### 3. Os 8 GB write-combined: dono a identificar
Os blocos de 64 MB no heap de upload não batem com os pools conhecidos:
- o staging de texturas (`pool.rs`) tem 128 a 896 MB de blocos;
- os blocos de texturas (3,3 GB) ficam na VRAM;
- o bloco de 64 MB do `indirect.rs` só existe em teste;
- a arena do `memory.rs` é do ray tracing, que está desligado.

Os candidatos que sobram são os envios das submissões (`host/send.rs`, guarda até 768 MB), os buffers de upload e as cópias na memória da placa. O mapa de memória não diz o dono.

### 4. Heaps do processo: ~25 MB por minuto
Na rodada longa os heaps do processo foram de 2,6 para 3,4 GB, em linha reta. Um vazamento de 30-50 MB por minuto já tinha sido visto numa sessão de 61 min do build antigo (`gaps.md`). A linha "our own allocations" do build de diagnóstico separa o que é nosso do que é do runtime e do driver, mas ninguém a comparou ao longo do tempo.

### 5. A memória do próprio jogo
~10 GB que o jogo usa, como no console. Não dá para encolher daqui. Só dá para deixar mais RAM para ela, cortando o que é do port.

## O que entrou para medir (sem mudar o comportamento padrão)
- **`VICTORY_INPLACE_CENSUS_AT=300,600,900`:** escolhe quando o censo do `VICTORY_INPLACE_PROBE=1` roda. Sem ela continua 30 e 150 s.
- **`VICTORY_BOOK_SUMMARY=1`:** liga o livro de recursos (`book.rs`) e, a cada 30 s, escreve os 20 grupos de recursos vivos com mais bytes, pelo nome que o nosso código dá a cada um, com os números tirados do nome. Responde de quem são os 8 GB write-combined. Custa um pouco de CPU por recurso criado, então é só para rodadas de diagnóstico.

## Plano, por custo e ganho

1. **Uma abertura de diagnóstico**, quando liberada, com `-SmallRam` e:
   - `VICTORY_INPLACE_PROBE=1`;
   - `VICTORY_INPLACE_CENSUS_AT=300,600,900`;
   - `VICTORY_BOOK_SUMMARY=1`;
   - o amostrador de paginação.

   Ela responde três perguntas: quanto das janelas é privado (`VICTORY_INPLACE=1` já serve), quanto é tira contígua do pool (vale comprometer o pool), e de quem são os 8 GB write-combined. Não muda nada para quem joga.
2. **Se boa parte das janelas for privada inteira:** medir `VICTORY_INPLACE=1` com a linha do tempo, observando o grupo compartilhado e as páginas lidas por segundo. É só uma chave.
3. **Se for tira contígua do pool:** criar o pool com `SEC_COMMIT` atrás de uma chave, quando o commit sobra (aqui sobram 30 GB), e abrir as janelas no lugar como a porta 1 faz (`OpenExistingHeapFromFileMapping`, sem travar páginas). É a maior mudança. As regras de submissões em voo do `inplace.rs` valem aqui; ler antes.
4. **O dono dos 8 GB write-combined**, conforme o item 1 mostrar: se forem os envios (`send.rs`), baixar o que fica guardado em PC de 16 GB. Se for outro pool, ver por que retém.
5. **Heaps do processo:** comparar "our own allocations" ao longo de uma sessão para saber se o crescimento é nosso.

**Expectativa, não medida:** cada GB tirado do grupo compartilhado é ~1 GB a mais de RAM para o jogo, que hoje tem ~1 GB dos seus ~10 GB na memória. Tirar as janelas (3-5 GB) muda a ordem de grandeza da paginação. É o que mais pode render nesta máquina, mais do que qualquer trava.

## Resultado da abertura de diagnóstico (27/11, 16:09-16:29, `-SmallRam`)

Uma rodada, quase toda em menus e cutscenes (o jogador pausou). Os números valem para esta máquina e este trecho.

**A sonda (`VICTORY_INPLACE_PROBE=1`, driver 621.40):** ler a memória do jogo no lugar **funciona** na RTX 5060.
- Memória privada aberta inteira ou em parte: leitura e escrita pela placa ok.
- Seção comprometida aberta por file mapping: leitura e escrita ok, mesmo com a view desmapeada ou protegida.
- **Seção reservada: recusada** (`0x80070057`), que é o caso do pool de páginas hoje.
- **Custo de leitura pela placa: igual ao de hoje.** A placa copia 64 MB da memória no lugar em 5,3-5,5 ms, contra 5,4 ms de um buffer compartilhado do tipo atual.
- Abrir memória privada trava as páginas na RAM (64 MB entraram no conjunto de trabalho). A seção comprometida só entra quando a placa lê.

**O censo (aos 5, 10 e 15 min):** as janelas, em quase toda a sua maior parte, **não podem** ir para o lugar sem cópia.

| Janelas fora de buffers | 5 min | 15 min |
|---|---|---|
| Total | 583 janelas, 2.404 MB | 821 janelas, 3.424 MB |
| Privada inteira (o `VICTORY_INPLACE=1` cobre) | 76, 364 MB | 78, 368 MB |
| Memória de view com páginas desmapeadas | 249, 1.008 MB | 576, 2.376 MB |
| Privada não inteira, ou guardada | 255, 1.020 MB | 142, 568 MB |
| Uma tira contígua de uma seção | 0 | 0 |

**Por que:** o jogo mapeia a memória física **uma página de 64 KB por vez**. São 57 a 85 mil views das páginas do jogo, todas de uma página, exceto 6. Uma janela de 4 MB cobre dezenas de views espalhadas, com buracos no meio. Comprometer o pool não muda isso. Abrir cada view como heap daria um heap por página, sem um buffer contínuo, e mapear blocos de páginas (tile mappings) num heap desses derruba o dispositivo.

Os buffers do jogo, para comparação, somam 756 MB: 252 privados inteiros, 230 numa tira contígua (abríveis com o pool comprometido) e 273 espalhados.

**Conclusão:** ler no lugar sem cópia tiraria no máximo ~0,4 GB de janelas (`VICTORY_INPLACE=1`). Com buffers (`=2`) e pool comprometido, até ~0,9 GB. Não os 3-5 GB esperados. O item 3 do plano cai.

**Os 8 GB write-combined: a maior parte é VRAM, não RAM.** Em 37 mapas de memória, o número de grupos write-combined de 64 MB acompanha os blocos do pool:
- é sempre a soma dos blocos de texturas (45-46), mais os blocos de staging (3 a 26), mais ~16 constantes;
- exemplos: 89 = 46 + 26 + 17, e 64 = 45 + 3 + 16;
- só ~0,65 GB dos 7,3 GB aparece na RAM do processo.

**Inferência:** com o Resizable BAR (a placa aceita `GPU_UPLOAD`), o driver mapeia os heaps de textura da VRAM no espaço de endereços do processo como write-combined. Eles contam no mapa de memória, mas não ocupam RAM. Esses 8 GB não são o que procurar.

**Quem segura os recursos D3D12 (`VICTORY_BOOK_SUMMARY`, 19:28):** 6,0 GB em 17 mil recursos vivos.

| Grupo | Tamanho | Onde fica |
|---|---|---|
| Janelas (`title memory`) | 2,5 GB (610) | RAM compartilhada |
| Buffers do jogo | 0,9 GB (3.488) | — |
| Janelas na VRAM pelo BAR (`mapped on the card`) | 0,74 GB (175) | VRAM |
| Buffers do jogo na placa | 0,5 GB (1.193) | — |
| Envios das submissões (`send.rs`) | 0,48 GB (22) | — |
| Staging | 0,26 GB (4) | — |
| Lotes indiretos | 0,16 GB (2.636) | — |

**Paginação:** em 211 amostras, mediana de 2.947 páginas lidas por segundo e pico de 137 mil. A RAM livre ficou em 830 MB de mediana, chegando a 15 MB. É menos que na rodada da linha do tempo (9.644), porque esta ficou mais em menu.

## Plano revisto
1. **`VICTORY_INPLACE=1`** (só chave): ~0,4 GB de janelas sem cópia, sem custo de leitura medido. Medir o grupo compartilhado e a paginação no mundo.
2. **Janelas na VRAM:** é para onde ainda dá para mover os ~2,5 GB de janelas que ficam na RAM. Hoje a porta (`VICTORY_WINDOWS_BAR_MB`) só aceita janelas com a placa abaixo de 80% do orçamento, e com `-SmallRam` ela fica perto de 95%. Uma troca a medir: `VICTORY_TEXTURE_MAX=512` libera VRAM para as janelas, ao preço de textura menor. É troca de imagem por memória, e a decisão vai ao Bruno.
3. **Heaps do processo:** 3,3 GB e crescendo ~25 MB/min. Achar de quem é com "our own allocations" ao longo de uma sessão longa.
4. **O que o jogo vê de si mesmo:** o `XMemGetWorkingSetStatistics` dá ao jogo o commit do processo inteiro (aqui ~19-21 GB, contra o limite de 7,8 GB do Series S). Registrar o que ele faz depois dessa chamada.
