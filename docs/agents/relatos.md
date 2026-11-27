# Relatos de playtest

Lista corrida do que foi visto jogando, para ninguém perder um defeito entre uma sessão e outra. Cada relato tem um número fixo (`R01`...) que as issues, os PRs e os achados citam. Relato novo entra no fim; relato resolvido não sai da lista, muda de estado.

Sem imagens: capturas do jogo não vão para o repositório. Descreva a cena.

**Régua** (`GLOSSARY.md`): bloqueia, degrada, tolerável. **Estado:** aberto, em teste, resolvido em `<commit>`, não reproduz.

## Playtest de 27/11/2026
Código do PR #34 (`eeba481`, build release), kit da rota nvidia com perfil de console Series S, RTX 4080 16 GB, 32 GB de RAM, Windows 10. FPS no título da janela: 8 a 17 no mundo, com 4,5 a 7,5 mil draws por quadro.

Esse build ainda tinha o defeito do heap de samplers consertado depois em `413bbc0`. Pela leitura do código ele não tocava a amostragem de texturas (nenhuma tabela dinâmica de sampler é criada), então não explica R03 nem R04; a confirmação no jogo fica para a próxima rodada.

| Nº | O que se vê | Onde | Régua | Suspeita (hipótese, não provada) | Teste que decide | Issue | Estado |
|---|---|---|---|---|---|---|---|
| R01 | Retângulos pretos horizontais na água | Píer, à noite | bloqueia | Os dois programas de volumetria da água rodam na forma wide aproximada | `VICTORY_WAVE_WIDE=0`, com giro de câmera | #36, #11 | aberto |
| R02 | Debaixo d'água a tela fica toda preta; só a interface aparece | Mergulho junto ao píer | bloqueia | A mesma volumetria da água (transmissão), ou um dos três computes de tela inteira sem tradução | O mesmo teste do R01, mergulhando; ler os draws descartados debaixo d'água | #37 | aberto |
| R03 | NPC com a cabeça inteira preta; corpo e roupa certos | Posto de gasolina, à noite | bloqueia | Textura de pele da cabeça ainda na fila de envio (`VICTORY_TEXTURE_NO_WAIT=1`) ou bake de pele descartado | `VICTORY_TEXTURE_NO_WAIT=0`; `VICTORY_DUMP_BAKES=1` | #38, #4 | aberto |
| R04 | Rosto de NPC coberto de ruído vermelho quadriculado; cabelo e corpo certos | Lanchonete do posto | bloqueia | Bake de pele lendo as fontes erradas, ou níveis grandes da textura lendo o vizinho | `VICTORY_DUMP_BAKES=1`; portar o bake de pele ou `VICTORY_PACKED_LEVELS` do fxceless | #5 | aberto |
| R05 | Manchas pretas grandes no chão e na vegetação; metade da cena sem luz | Quintal junto à água, à noite | bloqueia | Sombras ou luz por blocos de tela vindas de um compute sem tradução ou na forma wide | `VICTORY_WAVE_WIDE=0`; ler os draws descartados na cena | #39 | aberto |
| R06 | Vegetação e árvores piscando ao andar | Várias | bloqueia | Culls de árvore na forma wide. Reserva de tabelas descartada (0 recusadas) | `VICTORY_WAVE_WIDE=0`, com giro de câmera | #40 | aberto |
| R07 | Um braço e uma escada ficam pretos ao mover a câmera | Casa inicial | bloqueia | Textura na fila de envio (`VICTORY_TEXTURE_NO_WAIT=1`, `VICTORY_UPLOAD_AHEAD=1`) | Cada chave em 0 | #7 | aberto |
| R08 | Imagem chapada, sem luz local; lâmpada acesa não ilumina o chão | Deque da casa, à noite | degrada (bloqueia no escuro) | A iluminação do jogo não roda: ~900 execuções de ray tracing por quadro ficam de fora | `VICTORY_BVH_BUILDER=1` depois de traduzir o construtor para grupos de 32 | #14 | aberto |
| R09 | Véu cinza sobre a imagem inteira em interior | Lanchonete do posto | degrada | Luz substituta constante somada ao ar (`VICTORY_AMBIENT_AIR=1.5`) | `VICTORY_AMBIENT_AIR` em outros valores, mesma cena | #14 | aberto |
| R10 | Borrão e rastro em vegetação, barba e cabelo ao mexer a câmera | Várias | degrada | Reconstrução temporal a 10 FPS; o upscaler do jogo é recusado pelo port | Rever com FPS mais alto | #12 | aberto |
| R11 | Um flash vermelho na tela inteira, uma vez | Casa inicial | ? | Nenhuma | Captura em rajada | | aberto |
| R12 | Um retângulo no céu, de dia | Casa inicial | ? | A mesma dos retângulos da água | `VICTORY_WAVE_WIDE=0` | | aberto |
| R13 | Céu todo preto à noite, sem estrelas nem nuvens | Píer | ? | Pode ser a noite do jogo sem a iluminação dele; não comparado com o console | Comparar com imagem de referência do console | | aberto |
| R14 | 8 a 12 FPS em cena cheia, com travadas (1% mais lento a 3-7 FPS) | Mundo | bloqueia | O processador limita: 51-57 ms de preparo por quadro | `handoff.md`, passos 3 e 4 | #10, #13 | aberto |
| R15 | Sobra menos de 1 GB de RAM livre com o jogo aberto em 32 GB | Mundo | bloqueia em sessão longa | Conjunto de trabalho de ~24 GB | `handoff.md`, passo 7 | #2 | aberto |

## Resolvidos nesta rodada de trabalho
| Nº | O que se via | Como saiu |
|---|---|---|
| R00a | Persiana, luminária e cortina roxas | `VICTORY_AMBIENT=constant:0.8` (valor da rota nvidia) |
| R00b | 3 FPS depois do PR #28 | Reaproveitamento das cópias de views (PR #32) |
| R00c | O port caía no primeiro quadro no Windows 10 | Agility SDK (PR #31) |
