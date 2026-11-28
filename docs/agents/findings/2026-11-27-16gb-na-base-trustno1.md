# 16 GB na base do trustno1 (branch `perf/16gb-trustno1`), 27/11/2026

**Máquina B:** RTX 5060 8 GB, i5-13400F, 16 GB de RAM, Windows 11, driver 621.40.
**Base:** a `main` do fork depois do PR #58 (trustno1 `64d4931`), mais o commit `6cae48e`.
**Execução:** build release, `scripts\play-foxhound.ps1 -SeriesS`. As rodadas são de jogo normal, não de ponto fixo.

## O que a branch muda
- `residency.rs`: o freio pelo orçamento compartilhado do DXGI, vindo do PR #57 (85, 90, 95 e 97%); `VICTORY_SHARED_GUARD=0` desliga.
- `play-foxhound.ps1 -SeriesS`: `VICTORY_CARD_ROOM_FOR_TARGETS_MB=1152`. A reserva de 3.072 MB para render targets que ainda não existem deixava ~2 GB parados em 720p.
- **Não portado:** o `EXACT_LAZY` e a `EXACT_SPLIT` da árvore antiga. Esta base pede só 2 a 4 cópias `exact` por quadro, contra 1.500 a 2.200 na árvore antiga.

## O que se viu
| Sessão (Series S) | FPS no mundo | Fila de texturas | Recusas por "sem espaço" | Falhas de upload | Fim |
|---|---|---|---|---|---|
| Base sem a branch, sem DLSS | 10 a 12,7 | 15,8 mil | 332 | — | fechada à mão |
| Branch, sem DLSS | 8 a 13,3 (placa a 60-93%) | 1 a 11 | 1 | 0 | fechada à mão pelo F1 |
| Branch, DLSS Performance com geração de quadros 3x | — | — | 1.047 | **64.969** (todas nos 2 min finais) | **queda** aos ~17 min: `E_OUTOFMEMORY` ao fechar uma lista |
| Branch, DLSS Performance sem geração de quadros | **16,5 a 20,3 reais** (placa a 88-91%) | — | 69 | 0 | **queda** aos ~7 min |

- **Geração de quadros 3x:** ela reserva 1,5 a 2 GB a mais. Numa placa de 8 GB com 16 GB de RAM, a memória compartilhada ficou entre 7,2 e 7,26 de 7,45 GB, com o freio oscilando entre os níveis 3 e 4 (60 mudanças). Sem DLSS e sem a geração, a mesma branch teve 0 falhas e 0 mudanças de nível. A sessão com a geração começou do zero, num save novo, então a cena também era outra.
- **DLSS sem geração de quadros:** foi o melhor FPS nesta máquina, 16,5 a 20,3 reais. Ainda assim caiu dentro do driver em `nvwgf2umx.dll+0x34a44a` (leitura em `0xc`), num `CopyTextureRegion` de `texture::record_upload`, chamado de `draw::State::heap_textures` durante um `dispatch_mesh` (pilha resolvida pelo PDB do release). Nos 20 segundos antes, a RAM livre caiu para 10-386 MB, com 39 a 94 mil páginas lidas do disco por segundo. **Inferência:** a RAM do PC esgotou, e o driver falhou numa alocação interna da cópia. O mesmo endereço de queda já tinha aparecido na árvore antiga, numa rodada com tetos de memória.

## A seguir
- Em 16 GB, a RAM física é o limite que sobra. O freio olha o orçamento do DXGI, mas a RAM livre chegou a zero antes. Ideias:
  - um freio também pela RAM livre que aja de verdade (o atual só desliga com 5 GB livres e só limita o pré-carregamento);
  - deixar as janelas de memória do jogo ociosas irem mais cedo;
  - um teto de textura.
- Medir o mesmo trecho com DLSS sem geração, com e sem o ajuste da reserva, para separar o efeito dele no consumo de VRAM.
- Esta base nunca foi medida em 16 GB fora destas sessões.
