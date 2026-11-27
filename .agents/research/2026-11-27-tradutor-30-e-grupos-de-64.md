---
project: vai-funcionar
type: research
updated: 2026-11-27
---

# Tradutor: o pânico do #30 e os 37 programas em grupos de 64

## #30
- Programa: compute `02384a60fda6f2d0` = `DEFRAG_HASH`, `CS_DefragHeap_ComputeRelocations` (`crates/victory-gpu/src/draw.rs:19`). No build antigo era recusado sem barulho.
- Commits cool_fox c36f9f4 e ca7e21a (entraram com o #28) fazem `exchange_halves` chamar `order_waves(at, …)` -> `half_meet_at(block)`, que reescreve o terminador e move `self.cur`: só vale se `block == self.cur`. O chamador `mask_dword` (`lower.rs` ~3997-4024) passa `tail[def_block]`, bloco já fechado. Confirmado por execução com assert temporário ("half-meet at a block that is not the current one"). `scc_some` (~4924/4938) tem a mesma forma, não exercitado.
- O pânico aparece em `emit/bitcode.rs:872` ("use of a void value"); no jogo é capturado em `pipeline/make.rs:173`; no `victory shaders` derrubava o comando (saída 101).
- **Conserto mínimo: 36b998b** em `fix/30-half-meet-block`: `order_waves` recusa com `HALF_MEET_CLOSED` ("a half-meet in a block already closed"). Teste `half_meet_in_a_closed_block_is_refused` (vermelho na base, verde depois). Suíte victory-shader: 266 ok / 34 falhos, as mesmas 34 da base por nome (SM 6.6 sem Agility). `victory shaders` sem `VICTORY_DEFRAG`: saída 0, 25.657 no cache, 232 sem tradução (195 contêineres sem programa, 37 "no wave32 lowering"). Está no build play-melhor. **Sem PR ainda.**
- Paliativo por chave enquanto não entra: `VICTORY_DEFRAG=0` (`emit/mod.rs:650`).

## Tradução de verdade (worker, `.worktrees/fix-30-exchange`, branch `fix/30-exchange-in-current-block`, sobre 36b998b; escrita, NÃO compilada)
- `Lower::exchange_block(at, made)`: com half-meets e `at != self.cur`, a troca inteira (store, encontro, loads, encontro) vai para o bloco aberto; o ballot fica no bloco que escreveu a máscara. Par trocado guardado em `met_ballots[(lo, cur_ir)]`. Incorpora a ideia do cool_fox PR #53: troca com barreira de grupo na definição quando o grupo inteiro passa por ela.
- Testes: `a_mask_written_before_is_exchanged_where_it_is_read` e, na placa (precisa de SM 6.6/Agility), `a_mask_read_after_its_block_is_the_consoles_on_two_waves_that_meet_on_their_own` (grupo de 128, s0 = 0, 20, 40, 64, 100, 128, comparação bit a bit com a máscara calculada no processador), mais um com máscara escrita num ramo externo e lida num interno.
- Não verificado: se `all_lanes(cur_ir)` vale na região wide do defrag; número de sites de encontro (`meet_cap`). Desenho completo em `…\c11be212-…\scratchpad\design-30-11.md` (cópia em `victory-tools\arquivo`).

## Os 37 (issue #11)
- "no wave32 lowering" nasce em `emit/mod.rs:771`; motivos em `lower.rs:1874-1909`; tentativas R1-R6 em `mod.rs:532-750`.
- Rodada `VICTORY_WAVE_WHY=1`: TODAS as linhas têm "LDS in a group over 32, an LDS read and write in one lane-branch region"; extras variam (mask as data, `v_mbcnt`, `v_readlane`, lane-constant mask); grupos de 64 na maioria, também 128-1024. Mesma família do defrag. Log: `…\c11be212-…\scratchpad\shaders30-why.log.err`. O WHY não imprime o hash (falta 1 linha em `lower.rs:1909`).
- Hashes vistos sem conversão no jogo: `5e3dd99e0f067aee`, `02384a60fda6f2d0`, `ae0deeafe219c15c`, `6b82bc7d22016cb5` (vfx_triangleClip), `7c9a6a83826cdfac`, `21f7ad7863620328`, `a10179ab7eda0dc3`, `926c46bab3b86bd6`. Três novos descartados 1x por quadro com "lane mask exchanged where not every lane runs" no offset 0x218: `8b437ab61d8008fe`, `477b52e73ddeb6f2`, `17af85f170e66f30` (traduziam no build antigo: suspeita de regressão dos mesmos commits).
- Decisão pendente: estender os half-meets do worker ou portar a forma paired do fxceless. O worker responde depois de medir quantos dos 37 a tradução dele destrava.

## Atualização 02:49: tradução de verdade do #30 (worker)
- Commit `fdd93ea` em `fix/30-exchange-in-current-block`. Testes de `victory-shader` sem Agility: 268 passam, 34 falham (as mesmas 34 da base, que pedem SM 6.6). Os dois testes novos de lowering passam. `clippy -D warnings` ok.
- `victory shaders` com cache próprio: 25.658 no cache, **231 sem tradução** (195 sem programa + **36** grupos de 64 sem conversão; antes 37). O que passou a traduzir foi só o `02384a60fda6f2d0` (`vfx_triangleClip:CS_DefragHeap_ComputeRelocations`, o defrag). Nenhum pânico.
- **NÃO provado na placa:** os dois testes com Agility (`test/30-exchange-agility`) dão "device removed". Causa provável, não confirmada: o shader com half-meets usa o root UAV `u252` (`SLOT_MEET`) e o harness `whole_uav` só liga um UAV; o cool_fox PR #53 acrescenta `Root::Uav` ao harness para isso.
- **NÃO levar `fdd93ea` ao build de jogo** enquanto não passar na placa: o fxceless relatou grade colorida nos rostos com o defrag rodando.
- `TRANSLATOR_VERSION` ainda não subiu (sobe quando a branch for para o jogo).
- Target compartilhado (`.worktrees\_target-shared`): 1º build de diagnóstico completo, frio: 349 s com `-j 6` em prioridade Idle e o jogo aberto.
- **RAM:** durante o link desse build a RAM livre caiu a 480 MB com o jogo aberto. Regra nova: nada de link pesado com o jogo aberto.
