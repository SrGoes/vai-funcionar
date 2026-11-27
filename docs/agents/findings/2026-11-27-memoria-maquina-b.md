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
