# Lacunas abertas (27/11/2026)

Os defeitos vistos jogando estão numerados em `relatos.md`; aqui ficam as causas e os testes. Régua: **bloqueia** = abaixo do piso de "jogável"; **degrada** = incomoda sem impedir; ver `GLOSSARY.md`. "Issue" vazia = ainda sem issue.

## Imagem
| Lacuna | Régua | O que se sabe | Teste que decide | Conserto provável | Issue |
|---|---|---|---|---|---|
| Retângulos na água e um no céu | bloqueia | Continuam com luz constante e com ou sem os consertos. A lista de programas que liga `scene_kernel` (`emit/mod.rs:625-640`) força a forma wide aproximada em seis programas, dois deles da volumetria da água. Outros forks veem a água limpa sem isso | `VICTORY_WAVE_WIDE=0` (esfria o cache) | Forma paired do fxceless | #36, #11 |
| Vegetação piscando ao andar | bloqueia | Não reproduzida com câmera parada. Reserva de tabelas descartada (0 recusadas). Os culls de árvore estão na mesma lista | O mesmo teste, com giro de câmera | O mesmo | #40 |
| Braço e escada pretos ao mover | bloqueia | ~12 mil texturas na fila de envio; a rota nvidia põe `VICTORY_TEXTURE_NO_WAIT=1` e `VICTORY_UPLOAD_AHEAD=1` | Cada chave em 0, com giro | Esperar a textura só para o que está perto | #7, #38 |
| Sombras erradas | bloqueia | Vista uma vez (metade da tela escura). Não investigada | Captura com giro e leitura dos draws descartados | | #39 |
| Imagem chapada, sem luz local | degrada (bloqueia no escuro) | A iluminação do jogo fica de fora: ~900 execuções por quadro; entra uma constante | `VICTORY_BVH_BUILDER=1` depois do passo 0 | Construtor de ray tracing em grupos de 32, ou DXR | #14 |
| Borrão em vegetação, barba e cabelo ao mexer a câmera | degrada | A 10 FPS a reconstrução temporal mistura quadros a 100 ms; o upscaler do jogo é recusado pelo port | Rever com FPS mais alto | | #12 |
| Flash vermelho | ? | Visto uma vez, sem captura | Captura em rajada | | |
| Pele, tatuagem, braço cinza, rosto sem superfície | bloqueia | Vistos de novo em 27/11: NPC com a cabeça preta (R03, #38) e rosto coberto de ruído (R04, #5). Sem conserto limpo em nenhum fork | `VICTORY_DUMP_BAKES=1` quando reaparecer | | #4, #5, #6, #8 |
| Debaixo d'água tudo preto | bloqueia | Visto em 27/11 (R02). Mesma suspeita da água | `VICTORY_WAVE_WIDE=0` mergulhando; draws descartados debaixo d'água | Forma paired do fxceless | #37 |

## Tradutor
| Lacuna | O que se sabe | Próximo passo | Issue |
|---|---|---|---|
| 37 programas em grupos de 64 sem conversão (36 com a tradução de verdade do #30, que está fora do conjunto) | Todos recusam por "LDS in a group over 32"; três deles (`8b437ab6`, `477b52e7`, `17af85f1`) caem 1 vez por quadro e traduziam antes dos commits dos half-meets | Provar `fdd93ea` na placa; decidir half-meets ou forma paired | #11 |
| Pânico do #30 | Só a recusa entrou (conserto mínimo); a issue pede também que o `victory shaders` trate um pânico como falha daquele shader e siga. Tradução de verdade fora | PR próprio quando passar na placa; a issue continua aberta | #30 |

## Desempenho e memória
| Lacuna | O que se sabe | Próximo passo | Issue |
|---|---|---|---|
| 8-12 FPS | O processador limita: preparo 51-57 ms por quadro | Passos 3 e 4 do `handoff.md` | #10, #13 |
| 24-25 GB de RAM | Conjunto de trabalho grande; vazamento de 30-50 MB por minuto em "our own allocations" e "distinct views kept", medido só numa sessão longa do build antigo | Três chaves do `handoff.md`; linha de log com a memória do jogo; contadores por tipo | #2, #3 |
| Sem perfil do processador | O `wpr` da máquina grava e falha ao salvar (0x80010106) | Instalar o Windows Performance Toolkit ou outro caminho | |
| Cache frio | Mudar chave do tradutor esfria tudo | Pré-aquecimento por cache separado | #15 |

| Quarentena do reaproveitamento (#32) | Hipótese de revisão, não confirmada: um buffer solto volta à reserva quando a submissão que o marcou termina, enquanto `retire_resource` espera mais submissões (`bury_lag`). Se uma submissão na fila ainda ler o buffer, ele pode ser zerado antes | Ler se a lista não enviada segura o `Arc`; `VICTORY_VIEW_BYTES_REUSE=0` na cena dos relatos R03 e R04 | |

## Processo
| Lacuna | Próximo passo | Issue |
|---|---|---|
| Medidor e comparadores de imagem fora do repositório | Trazer para `scripts/` sem caminho local | |
| Series X nunca medido na máquina de 32 GB | Uma rodada vigiada; os consertos do mgrein só valem lá | |
| Testes de GPU intermitentes | Retomar `fix/26-gpu-test-isolation`. O teste de GPU novo do reaproveitamento em `shadow.rs` pede `--test-threads=1` e não tem `#[ignore]` | #26 |
| Limpezas apontadas na revisão do PR #34 | Três `agility.rs` quase iguais; testes repetidos no `play.ps1`; números de vendor soltos em `indirect.rs`; `xtask/src/agility.rs` cita uma issue de outro repositório; dispatch largo ligado por vendor com base numa única RTX 4070 | |
