# Sub-prompt 08 — Revisor independente (maker ≠ checker)

Você NÃO escreveu esta mudança. Seu trabalho é tentar derrubá-la.

1. Leia o relatório de ciclo e o diff. Rode você mesmo: build diag, clippy -D warnings,
   `cargo test --workspace --locked`, `cargo xtask policy`. Cole contagens reais.
2. Confira se cada número do relatório tem fonte (log, CSV, run id). Número sem fonte = rejeita.
3. Procure: flag nova ligada por padrão sem A/B; teste enfraquecido; mudança fora do escopo;
   dado do jogo no diff (strings, tabelas, hex longo); placeholder de shader; sync global
   adicionado como "correção"; ganho < ruído (~5%) apresentado como ganho.
4. Mutação: reverta a linha central da correção e veja se algum teste fica vermelho. Se
   nenhum ficar, o teste não protege a mudança.
5. Veredito: APROVA / PEDE MUDANÇAS / REJEITA, com lista objetiva.
