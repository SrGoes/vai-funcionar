# Lacunas abertas (27/11/2026)

Régua: **bloqueia** = abaixo do piso de "jogável"; **degrada** = incomoda sem impedir; ver `GLOSSARY.md`. "Issue" vazia = ainda sem issue.

## Imagem
| Lacuna | Régua | O que se sabe | Teste que decide | Conserto provável | Issue |
|---|---|---|---|---|---|
| Retângulos na água e um no céu | bloqueia | Continuam com luz constante e com ou sem os consertos. A lista `scene_kernel` (`emit/mod.rs:625-640`) força a forma wide aproximada em seis programas, dois deles da volumetria da água. Outros forks veem a água limpa sem isso | `VICTORY_WAVE_WIDE=0` (esfria o cache) | Forma paired do fxceless | #11 |
| Vegetação piscando ao andar | bloqueia | Não reproduzida com câmera parada. Reserva de tabelas descartada (0 recusadas). Os culls de árvore estão na mesma lista `scene_kernel` | O mesmo teste, com giro de câmera | O mesmo | |
| Braço e escada pretos ao mover | bloqueia | ~12 mil texturas na fila de envio; a rota nvidia põe `VICTORY_TEXTURE_NO_WAIT=1` e `VICTORY_UPLOAD_AHEAD=1` | Cada chave em 0, com giro | Esperar a textura só para o que está perto | #7 |
| Sombras erradas | bloqueia | Vista uma vez (metade da tela escura). Não investigada | Captura com giro e leitura dos draws descartados | | |
| Imagem chapada, sem luz local | degrada (bloqueia no escuro) | A iluminação do jogo fica de fora: ~900 execuções por quadro; entra uma constante | `VICTORY_BVH_BUILDER=1` depois do passo 0 | Construtor de ray tracing em grupos de 32, ou DXR | #14 |
| Borrão em vegetação, barba e cabelo ao mexer a câmera | degrada | A 10 FPS a reconstrução temporal mistura quadros a 100 ms; o upscaler do jogo é recusado pelo port | Rever com FPS mais alto | | #12 |
| Flash vermelho | ? | Visto uma vez, sem captura | Captura em rajada | | |
| Pele, tatuagem, braço cinza, rosto sem superfície | bloqueia | Não vistos na rota nvidia em Series S nas últimas sessões; sem conserto limpo em nenhum fork | `VICTORY_DUMP_BAKES=1` quando reaparecer | | #4, #5, #6, #8 |

## Tradutor
| Lacuna | O que se sabe | Próximo passo | Issue |
|---|---|---|---|
| 36 programas em grupos de 64 sem conversão | Todos recusam por "LDS in a group over 32"; três deles (`8b437ab6`, `477b52e7`, `17af85f1`) caem 1 vez por quadro e traduziam antes dos commits dos half-meets | Provar `fdd93ea` na placa; decidir half-meets ou forma paired | #11 |
| Pânico do #30 | Conserto mínimo neste conjunto; tradução de verdade fora | PR próprio quando passar na placa | #30 |

## Desempenho e memória
| Lacuna | O que se sabe | Próximo passo | Issue |
|---|---|---|---|
| 8-12 FPS | O processador limita: preparo 51-57 ms por quadro | Passos 3 e 4 do `handoff.md` | #10, #13 |
| 24-25 GB de RAM | Conjunto de trabalho grande e vazamento de 30-50 MB por minuto em "our own allocations" e "distinct views kept" | Três chaves do `handoff.md`; linha de log com a memória do jogo; contadores por tipo | #2, #3 |
| Sem perfil do processador | O `wpr` da máquina grava e falha ao salvar (0x80010106) | Instalar o Windows Performance Toolkit ou outro caminho | |
| Cache frio | Mudar chave do tradutor esfria tudo | Pré-aquecimento por cache separado | #15 |

## Processo
| Lacuna | Próximo passo | Issue |
|---|---|---|
| Medidor e comparadores de imagem fora do repositório | Trazer para `scripts/` sem caminho local | |
| Series X nunca medido na máquina de 32 GB | Uma rodada vigiada; os consertos do mgrein só valem lá | |
| Testes de GPU intermitentes | Retomar `fix/26-gpu-test-isolation` | #26 |
