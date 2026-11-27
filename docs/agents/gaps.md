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
| 15 FPS (cena fixa, máquina A, kit novo) | Depois do `EXACT_LAZY` a trava `EXACT` deixa de dominar; as gravadoras ainda gastam a maior parte do quadro nas chamadas de lista (~14 µs por draw em `bind`/`tables_with_extents`/`readable`, 4 travas globais por textura) | Perfil com `VICTORY_TIMELINE_RECORDERS=1` depois do kit novo; travas de `resource.rs`/`texture.rs` | |
| RAM livre ~120-320 MB em 32 GB | `EXACT_LAZY` mantém até 64 cópias ociosas a mais; o pico subiu ~0,9 GB em 15 min | Contar os bytes guardados pela `EXACT`; `VICTORY_TEXTURE_BUDGET_MB` | #2 |
| 8-12 FPS | O processador limita: preparo 51-57 ms por quadro | Passos 3 e 4 do `handoff.md` | #10, #13 |
| 24-25 GB de RAM | Conjunto de trabalho grande; vazamento de 30-50 MB por minuto em "our own allocations" e "distinct views kept", medido só numa sessão longa do build antigo | Três chaves do `handoff.md`; linha de log com a memória do jogo; contadores por tipo | #2, #3 |
| Sem perfil do processador | O `wpr` da máquina grava e falha ao salvar (0x80010106) | Instalar o Windows Performance Toolkit ou outro caminho | |
| Memória compartilhada com a placa em PC de 16 GB | Orçamento do DXGI de 7,4 GB; janelas de memória do jogo ~5 GB, presas por listas; a queda vinha daí (ver `handoff.md`, máquina B). `-SmallRam` passa; o freio do grupo compartilhado segura as texturas, mas não as janelas | Medir `-SmallRam` em ponto fixo; contar quanto das janelas vai para a VRAM com `VICTORY_WINDOWS_BAR_MB`; achar o que pede as janelas sem tamanho (3,8 GB) | #2, #3 |
| Fila de texturas (13-25 mil esperando) | Com `VICTORY_TEXTURE_NO_WAIT=1` o que não subiu sai preto; o teto de 1024 baixou a fila para 6-8 mil no mundo | Captura com giro, com e sem `VICTORY_TEXTURE_MAX`, para pesar o detalhe perdido | #7, #38 |
| Cache frio | Mudar chave do tradutor esfria tudo. Com os pipelines frios a rota nvidia cai no primeiro quadro do mundo (R16) | `victory shaders` e `warm --used` antes de abrir; achar o dispatch que o jogo lê de volta | #15, #53 |

| Quarentena do reaproveitamento (#32) | Hipótese de revisão, não confirmada: um buffer solto volta à reserva quando a submissão que o marcou termina, enquanto `retire_resource` espera mais submissões (`bury_lag`). Se uma submissão na fila ainda ler o buffer, ele pode ser zerado antes | Ler se a lista não enviada segura o `Arc`; `VICTORY_VIEW_BYTES_REUSE=0` na cena dos relatos R03 e R04. A leitura do código não confirma a hipótese pelo caminho das listas | #41 |

| Ports do upstream ainda não trazidos | 26 commits revisados; os de maior valor: tirar o par de volumetria da água da forma wide, a luz do ar no modo constante, a lista que cresce sem limite (memória) | Seguir a ordem da issue; não usar "Sync fork" | #43 |

## Processo
| Lacuna | Próximo passo | Issue |
|---|---|---|
| Medidor e comparadores de imagem fora do repositório | Trazidos para `scripts/measure/` (PR do medidor). Falta uma rodada nova por eles, pelo `play.ps1`, para provar a cadeia inteira | #44 |
| Series X nunca medido na máquina de 32 GB | Uma rodada vigiada; os consertos do mgrein só valem lá | |
| Testes de GPU intermitentes | Retomar `fix/26-gpu-test-isolation`. O teste de GPU novo do reaproveitamento em `shadow.rs` pede `--test-threads=1` e não tem `#[ignore]` | #26 |
| Limpezas apontadas na revisão do PR #34 | Três `agility.rs` quase iguais; testes repetidos no `play.ps1`; números de vendor soltos em `indirect.rs`; `xtask/src/agility.rs` cita uma issue de outro repositório; dispatch largo ligado por vendor com base numa única RTX 4070 | #42 |
