# Passagem de bastão: vai-funcionar (27/11/2026, ~03:40 BRT)

Para: Goes (dono do repo) e a próxima sessão de agente que ele abrir.
De: sessão de revisão do Bruno (Lag0). Esta sessão só leu e revisou; não compilou, não abriu o jogo e não gravou nada no repositório.

## 1. Os repositórios

| Repo | Papel |
|---|---|
| **SrGoes/vai-funcionar** (https://github.com/SrGoes/vai-funcionar) | Este repo. `main` = `68b888b`, com os PRs #31 a #35 já mesclados. |
| **DoubleGhosty/cool_fox** (https://github.com/DoubleGhosty/cool_fox) | **Fork de origem** (o "forked from" do GitHub). Continuação pública do cool_fox do grupo CGBJ (`codec-d/cool_fox`, hoje fora do ar). Base comum com a nossa `main`: `69718c0`. |
| Linhagem acima | Marvisak/VIctory → fxceless/victory-fork → forks (avatarmanz, DexNom, Belobug…) → codec-d/cool_fox → DoubleGhosty/cool_fox → vai-funcionar. Licença GPL-3.0. |

**Divergência com o fork de origem:** o GitHub mostra a `main` 64+ commits à frente e **26 atrás** de `DoubleGhosty/cool_fox:main`. Os 26 foram revisados um a um, sem aplicar nada: **issue #43** (por ordem de valor).

> **Não clicar em "Sync fork".** Ele traria os 26 de uma vez, inclusive a parte `bitcode.rs` do `1bd5e1a` (troca o assert "use of a void value" por um valor errado em silêncio; já recusado) e o trecho `exact()` do `42ac562`, que conflita com o reaproveitamento do #32. O caminho é cherry-pick ou reescrita por item, seguindo a #43.

Outros forks já lidos, com o que vale portar de cada um: `docs/agents/references.md` e `docs/ports/2026-11-27-mgrein.md` (mgrein/victory-nvidia).

## 2. Onde está o estado (ler nesta ordem, tudo na `main`)

1. `docs/agents/README.md`: ordem de leitura.
2. `docs/agents/handoff.md`: estado, o que está provado e o que não está, próximos passos e a tabela das duas máquinas.
3. `docs/agents/relatos.md`: relatos de playtest R01-R15, com suspeita, teste e issue de cada um.
4. `docs/agents/gaps.md`: lacunas, cada uma com o teste que a decide.
5. `docs/agents/findings/`: dez achados de 27/11, com números e lugares do código.
6. `GLOSSARY.md`: o vocabulário usado em tudo (rota, kit, preparo, Series S/X…).
7. `scripts/measure/README.md`: medidor e comparadores de imagem (PR #35). A primeira rodada pelo `play.ps1` ainda não foi feita: **#44**.

Este arquivo não repete nada disso. Cobre só o que veio depois e o que muda para a máquina do Goes.

## 3. O que mudou na main hoje (resumo; detalhe nos PRs)

- **#32**: reaproveitamento das cópias de views. Conserta a queda de 12 para 3 FPS na NVIDIA (#29). Desliga com `VICTORY_VIEW_BYTES_REUSE=0`.
- **#31**: Agility SDK (`cargo xtask agility`). Obrigatório no Windows 10; no Windows 11 o SM 6.6 já vem no sistema.
- **#33**: um launcher só, com rota explícita: `scripts/play.ps1 -Route nvidia|amd`. Os `.cmd` viraram atalhos finos. Chave `VICTORY_*` definida na janela antes do atalho **não chega mais** ao jogo; use `-Vars "A=1;B=2"`.
- **#34**: conserto mínimo do pânico do tradutor (#30: `victory shaders` termina), os dois consertos de placa do mgrein (heap de views limitado e dispatch indireto largo; valem para Series X), captura por tempo no build de diagnóstico, eixo do analógico em `VICTORY_KEYS` e `docs/agents/`.
- **#35**: `scripts/measure` (medidor, matriz e comparação de imagem).

**Cache de shaders:** o #34 mexe no tradutor (`emit/lower.rs`), então a chave do cache muda em relação à `main` de antes dele. Refaça o pré-aquecimento: `victory shaders <shaders.rpf> --jobs N` e depois `victory warm --used`.

## 4. Resultado da revisão desta sessão (já registrado)

- **PR #34** (revisão de padrões e de especificação): achou o heap de samplers caindo no ramo de "heap limitado" em qualquer placa (consertado em `413bbc0`, com teste; o worker conferiu que não estragava a amostragem); o filtro de magenta ligado sem chave e sem ganho (revertido em `9ca6117`); três frases que o código desmentia (corrigidas antes do merge).
- **#41** (hipótese, não confirmada): o reaproveitamento devolve o buffer à reserva esperando menos submissões do que `retire_resource` (que soma `queue::queued()` e `bury_lag`). Se uma submissão ainda for ler o buffer, ela lê zeros. Candidato à cabeça preta (#38). Teste sem compilar: `VICTORY_VIEW_BYTES_REUSE=0` na mesma cena.
- **#42**: limpezas (três `agility.rs` quase iguais, `play.ps1` com testes repetidos, IDs de fabricante soltos, teste de GPU sem `#[ignore]`, referência a "issue #84" de outro repo).
- **#43**: os 26 commits do fork de origem, ver a seção 1.
- **#14 (comentário):** `VICTORY_AMBIENT_AIR` **não roda** com luz constante (`crates/victory-gpu/src/ambient.rs:539`: `constant > 0.0` pula o papel `Air`). A rota nvidia usa `VICTORY_AMBIENT=constant:0.8` com `AIR=1.5`, então a chave do ar não faz nada hoje. O relato R09 (véu) a culpava.
- **#36 (comentário):** `CS_TransmissionVolumetrics_Tiled` e `_LocalLights` continuam na lista `scene_kernel` (`crates/victory-shader/src/emit/mod.rs:635-638`), na forma aproximada. O `963f719` do fork de origem tira os dois da lista, e numa RTX 5080 a água ficou limpa.

## 5. Máquina do Goes: o que muda

As medições de 26 e 27/11 são da máquina do Bruno (RTX 4080 16 GB, Ryzen 7 9800X3D, 32 GB, Windows 10). Na máquina do Goes (RTX 5060 8 GB, i5-13400F, 16 GB, Windows 11):

- **O limite é a RAM, não a placa.** O port usa 24-25 GB privados com o kit da rota nvidia. Com 16 GB, Series S caía depois de ~9 min (#2) e Series X não se sustentava (#3). Tudo o que baixa memória vale mais aí do que na 4080.
- **Perfil de console:** o modo automático só escolhe Series X com 48 GiB de RAM e 12 GiB de VRAM (`apps/victory/src/main.rs:144`), então na 5060 fica Series S. O comentário do kit nvidia diz que Series S perde a malha da cabeça em placa de 8 GB; vale conferir, porque é exatamente a máquina dele.
- **Agility:** desnecessário no Windows 11, mas também não atrapalha.
- **O que pode ajudar na memória, por ordem:** (1) o `c9ce62f` do fork de origem (tira do release uma lista que cresce sem limite, `PLACED_RAW`; é suspeito do vazamento de 30-50 MB/min); (2) chaves já existentes, nunca medidas: `VICTORY_TEXTURE_BUDGET_MB=4096`, `VICTORY_WINDOW_BUDGET_MB=1024` com `VICTORY_WINDOW_IDLE_MS=10000`, `VICTORY_UPLOAD_AHEAD=0` (`docs/agents/findings/2026-11-27-memoria.md`); (3) o `1daa043` do fork de origem (só log: compara o pool de páginas físicas, ~6,1 GiB fora da memória privada, com o limite de commit).

## 6. Próximos passos recomendados

Testes por chave, sem compilar, um de cada vez, na mesma cena, com `play.ps1 -Route nvidia -Vars "..."`:
1. `VICTORY_WAVE_WIDE=0`: retângulos na água (#36) e árvores (#40). Esfria o cache.
2. `VICTORY_VIEW_BYTES_REUSE=0`: cabeça preta (#38), hipótese da #41.
3. `VICTORY_PORT_GUARD=1;VICTORY_PORT_JOURNAL=1`: custo de preparo no processador (o gargalo: 51-57 ms por quadro na 4080). O fork de origem ligou as duas por padrão; o mgrein mediu ganho numa 4070, e um fork com uma 5070 mediu perda.

Ports, por ordem de valor (detalhe e conflitos na #43):
1. `963f719`, só `emit/mod.rs`: a água, atrás de uma chave nova só para o par de transmission.
2. Luz do ar no modo constante: a parte `ambient.rs` do `1bd5e1a` mais o `fae8484` (NaN no ar vira 0). Reescrever, cerca de 15 linhas; não entra na chave do cache.
3. `c9ce62f`: memória.
4. `e8b62de`, `74eda3b`, `ca7e76a`: processador; aplicam limpo e não mudam o resultado. Medir os três juntos, antes e depois.
5. `dbb5878` (só no build de diagnóstico): conta os buffers de textura reescritos enquanto uma cópia ainda lê deles. Sonda do rosto com ruído (#5) e da cabeça preta (#38).
6. `cb1468d` + `47ca656`: o defrag do #30 do jeito validado numa RTX 5080, mais `VICTORY_DEFRAG` na chave do cache. Substitui a parte do `fdd93ea` (branch local `fix/30-exchange-in-current-block`) que deu "device removed".

Sempre: mesma cena, 3 rodadas, `scripts/measure` (#44 é a primeira rodada) e comparação de imagem. Uma mudança que melhora o FPS e piora a imagem fica atrás de chave, e a decisão vai para o humano com números e prints.

## 7. Pendências e avisos

- **Licença:** `crates/victory-loader/src/console_insn.rs` diz "Ported from XCL (GPL-2.0-only)", enquanto o `README.md` (~linha 113) afirma "no XCL code is included". Veio do fork de origem (commits de 25/11). GPL-2.0-only não combina com GPL-3.0. O Bruno ia falar disso com o Goes; nenhuma issue foi aberta.
- **PR #19** (diagnóstico da #2, antigo, `81ffe9e`): falta decidir entre rebasear e fechar.
- **#22** (o CI nunca rodou) e **#26** (testes de GPU intermitentes em paralelo): nenhum "verde" de suíte vale de verdade até essas duas saírem; até lá, rode `cargo test -p victory-gpu -- --test-threads=1` e compare os nomes que falham com os da base.
- **Branches só locais na máquina do Bruno** (não publicados): `fix/30-exchange-in-current-block` (`fdd93ea`), `diag/visual-counters`, `fix/26-gpu-test-isolation`. A lista está no fim do `docs/agents/handoff.md`.
- **Política de assets:** nada do jogo no repo, em issue ou em PR, inclusive prints e logs com dados do jogo (`docs/asset-policy.md`).
- **Coordenação:** a partir de 27/11 ~03:36 não há mais quadro nem trava entre sessões. Pedido novo vai direto para issue ou PR, com os rótulos de triagem (`docs/agents/triage-labels.md`).

## 8. Skills sugeridas para a próxima sessão

- `diagnosing-bugs`: para #36-#41 (água, cabeça preta, manchas, pisca), com reprodução e teste por chave antes de qualquer conserto.
- `tdd`: para os ports da #43 que mexem no tradutor ou na reserva (`cb1468d`, `963f719`, #41): teste vermelho primeiro.
- `code-review`: revisar cada branch de port contra a `main`, nas duas frentes (padrões e especificação).
- `pr`: corpo dos PRs no formato do repo (Summary, Evidence, Merge Danger).
- `domain-modeling`: para manter o `GLOSSARY.md` e abrir o primeiro ADR em `docs/adr/`, se uma decisão sair dos ports.
- `research`: para ler um fork novo ou documentação externa e registrar o resultado no repo.
