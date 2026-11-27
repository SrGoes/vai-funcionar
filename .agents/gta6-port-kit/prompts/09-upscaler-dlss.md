# Sub-prompt 09 — Upscaling: FSR4 do jogo, NIS, DLSS SR

Estado: issue #12 — o FSR4 do jogo não carrega (`DeserializeComputePipelineStateX` devolve
`E_NOTIMPL`) e não há upscaler ativo. `VICTORY_RENDER_SCALE=0.667..1.0` já existe.

## Ordem
1. Meça se a GPU está no caminho crítico. Se o gargalo é CPU de submissão (caso comum aqui),
   upscaler não dá FPS — diga isso e pare.
2. Caminho A: implementar no HLE o que falta para o FSR4 do próprio jogo (o jogo já fornece
   motion vectors/jitter corretos para ele).
3. Caminho B: NIS (espacial, sem dados extras) como fallback universal.
4. Caminho C (RTX): DLSS Super Resolution via Streamline — exige depth, motion vectors,
   jitter, exposição e reset de histórico corretos; leia a licença do SDK antes de
   distribuir binários. Nunca obrigatório para iniciar.
5. Frame Generation desligado em toda medição dos 20 FPS reais.
6. Prova: custo do próprio upscaler (ms), ghosting/disocclusion descritos, A/B por escala.
