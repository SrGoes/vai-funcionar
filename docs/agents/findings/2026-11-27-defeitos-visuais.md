# Defeitos visuais: o que se sabe por defeito

## Roxo em persiana, luminária, cortina (e vegetação avermelhada)
- Causa confirmada: a luz substituta em modo estimado. `crates/victory-gpu/src/ambient.rs`, no modo estimado, mantém o lixo que o programa de resolução das sondas deixa sem os raios ("in blocks of tiles what the probes' memory held", `docs/fork-ray-tracing.md:140-146`).
- `VICTORY_AMBIENT=constant:0.8` tira o roxo (confirmado no jogo). É o valor do launcher NVIDIA.
- O filtro de magenta do Belobug (`if (was.g < 0.5*min(was.r,was.b) && max(r,b) > 0.02) was = 0`, commit e942896; portado em 562e4a5) só trocou o magenta por azul-arroxeado no modo estimado: não resolveu.
- Outras pistas não testadas: constant buffer fora de 256 bytes descartado (`descriptor/host.rs:4584-4590`, log só mostra a primeira vez); joins de máscara zerados (`lower.rs:13478-13495`, `VICTORY_STRICT_JOINS=1`); formatos substitutos na NVIDIA (R9G9B9E5 -> RGBA16F, `resource.rs:268-310`, `VICTORY_ALIAS_COPY=0`).

## Retângulos na água (e um no céu)
- Continuam com luz fixa, com e sem o conserto das cópias, com e sem o filtro. Menos visíveis com o kit NVIDIA (relato do Bruno), mas presentes.
- Suspeita principal: os dois programas de volumetria da água na forma wide aproximada (lista de programas que liga `scene_kernel`). Teste: `VICTORY_WAVE_WIDE=0`. Conserto de verdade: forma paired (ver consertos em outros ports).
- Segunda suspeita: passes `CS_RaytracingResolve` deixados de fora (~912-930 por quadro, "left out: shades ray-tracing hits without BVH"); teste `VICTORY_RT_RESOLVE=1`. No dump de operações, a volumetria da água tem um UAV "heap 332576: null".

## Imagem lavada / névoa
- Estimativa a 0.8 soma luz em tudo; `VICTORY_AMBIENT_AIR` desligada no play.ps1 deixa o ar sem limite ("brown veil"). Com o kit NVIDIA (constante + `AMBIENT_AIR=1.5`) o interior fica limpo.
- Iluminação pobre sem ray tracing (pele pálida, sem reflexo): o Bruno classifica como "degrada".

## Vegetação piscando / árvore sumindo
- Não reproduzida com câmera parada. O Bruno vê ao andar. Suspeita: culls de árvore na forma wide; 3 computes sem tradução descartados 1x por quadro; "skipped shader 39f0e9aeb156a61c".
- Registro no código: `draw.rs:412-419` (palmeiras e árvores sumindo ao limitar em lote execuções com argumentos no lugar, runs pop-14/15).

## Vistos pelo Bruno com o kit NVIDIA e ainda sem investigação
Braço e escada pretos ao mover (suspeita `VICTORY_UPLOAD_AHEAD=1`); sombras bugando; flash vermelho; borrão/"artefato de DLSS" em vegetação, barba e pelo ao mexer a câmera (suspeita: reconstrução temporal do jogo a 8-10 FPS; o FSR4 do jogo não carrega, #12).

## Personagens (scout, sessão de 62 min do build antigo)
- Rosto sem superfície (olhos e boca flutuando): visto nesta 4080 em Series S com -Safe. Hipóteses: textura de pele lida como zeros com a guarda ligada (hash a cada 8 quadros, `texture.rs:1352`, releitura parcial a cada 30, `:1407`); mapeamento de blendshapes perdido (`CS_ComputeBlendshapeMappings` `61e54e90adccaf6a`; 700fc78 já força o pipeline antes do primeiro dispatch); perfil Series S. Provável conserto já no build novo (700fc78, 2a1965f), não verificado.
- Braço cinza (#4): bake/compress descartado no primeiro uso (compute BC3 `95988cc1e16f1d89`; 2a1965f corrige em parte) ou bake com entradas neutras. Teste `VICTORY_DUMP_BAKES=1`.
- Tatuagem em ruído (#5): corrida entre chegada da textura e o bake; 200 avisos "changed after upload" (teto em `draw.rs:2906`). Teste `VICTORY_TEXTURE_GUARD=0` (já no kit NVIDIA; custa processador: texturas 11-12 ms por quadro contra 3,6). No kit NVIDIA a tatuagem do protagonista apareceu certa (já aparecia certa com o kit antigo).
- Triângulos esticados (#6): views estruturadas fora de 16 bytes (8977bb0, agora ligado com o reaproveitamento) ou a cadeia do defrag; teste: cena do carro com e sem `VICTORY_VIEW_BYTES=0`.
- Personagens brancos no flashback (#8): realimentação da estimativa; teste `constant:0.8` (já no kit NVIDIA).
- Os ~260 draws sem pipeline por quadro NÃO são de personagem: 258-283 estágios do construtor de ray tracing, 1 defrag, 1 ativação de raios; os ~930 "indirect" são os `CS_RaytracingResolve` deixados de fora.
- Build antigo no bar: clientes sem roupa, um rosto coberto de ruído, luminária roxa.
