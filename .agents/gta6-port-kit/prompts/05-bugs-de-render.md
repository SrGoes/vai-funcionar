# Sub-prompt 05 — Bugs visuais (pele, decals, triângulos, iluminação)

Issues abertas: #4 pele cinza-esverdeada, #5 tatuagens como ruído, #6 triângulos esticados
nos cortes de câmera, #7 decals pretas intermitentes (corrida; `VICTORY_UPLOAD_AHEAD`
agrava), #8 personagens estourados no flashback, #9 bolhas de luz, #14 interiores escuros
sem RT.

## Método
1. Uma issue por vez. Reproduza com `-Safe` e sem `-Safe` (separa caminho novo × base).
2. Capture o quadro com `VICTORY_RECORD` e reexecute com `victory replay`; para inspeção
   de recursos use PIX ou RenderDoc **localmente** — capturas ficam fora do repo e não vão
   para issues (contêm dados do jogo; descreva em texto).
3. Rastreie do sintoma ao primeiro recurso errado: bake de textura de ped
   (`VICTORY_DUMP_BAKES`, ver bug-journal), matrizes de osso desatualizadas, upload chegando
   depois do uso, shader traduzido com precisão errada.
4. Correção mínima + teste (sintético) + antes/depois descrito em texto e números.

## Atenção
"Some quando ligo X" não é causa. O bug-journal já mostra que as decals voltam sem
`UPLOAD_AHEAD` em outras condições: procure a ordem real de upload × uso.
