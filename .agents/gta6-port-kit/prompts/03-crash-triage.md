# Sub-prompt 03 — Triagem de crash / device removed / tela preta

Objetivo: transformar um crash em causa reproduzível e um teste, antes de qualquer FPS.

## Coleta (sem dados do jogo no repo)
1. Build diag; `victory run --execute` já instala o crash reporter: falha como
   `module+rva` + pilha da thread. Copie o trecho do log (fora do repo).
2. GPU: `VICTORY_D3D12_DRED=light` (padrão no play.ps1) ou completo para breadcrumbs e page
   fault; em NVIDIA avalie Nsight Aftermath para dumps de GPU crash.
3. Debug layer / GPU-based validation do D3D12 só em sessões curtas (custo alto).
4. Anote: minuto do crash, cena, RAM/VRAM no momento, `E_OUTOFMEMORY`/`0x8007000E`/
   `DXGI_ERROR_DEVICE_REMOVED` + reason.

## Classificação
| Sinal | Classe provável | Próximo passo |
|---|---|---|
| `E_OUTOFMEMORY` / commit esgotado após minutos | memória (issues #2, #3) | sub-prompt 04 |
| Page fault de GPU num recurso recém-criado | slot exibido antes do upload (findings.md) | `VICTORY_SHOW_AFTER`, ordem de upload |
| Device removed com DRED apontando um pass | shader/barreira/descritor | replay do quadro + PIX/RenderDoc |
| Access violation em `module+rva` do título | HLE devolveu valor errado ou stub | sub-prompt 06 |
| Watchdog do jogo (XCL relata 3–14 min) | tempo de frame / fences | Nsight Systems |

## Regra
Não "conserte" engolindo a exceção ou devolvendo zero. Ache a primeira divergência e
escreva um teste (fixture `hle-probe` ou teste unitário) que falhe sem a correção.
