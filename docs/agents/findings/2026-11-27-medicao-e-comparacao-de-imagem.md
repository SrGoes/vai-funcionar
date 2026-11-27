# Medição e comparação de imagem

## Medidor (sessão de medição)
- (arquivo local, fora do repositório): `bench.py` (uma rodada), `matrix.py` (matriz com aquecimento, 3 rodadas em rodízio, para se vir cargo/rustc ou um victory.exe que não abriu, escreve a linha JOGO), `run.ps1`, `matrix-29*.json`. Rodada só vale com o jogo vivo os 75 s, sem "window closed" no log. Deixa o botão Fechar cinza (`VICTORY_BENCH_LOCK_CLOSE=0` desliga). Saída `runs.csv` e `summary.csv`.
- As rodadas C e C2 da noite de 26/11 foram encurtadas por fechamento manual da janela (`lifecycle.rs:252`, WM_SYSCOMMAND/SC_CLOSE).
- Limites: o "p95" é sobre amostras do título a cada 2 s; sem 1% low (para isso, PresentMon).

## Melhorias achadas pelo scout (adotar antes da próxima matriz)
1. **Um Enter só, aos 10 s** (`VICTORY_KEYS=10:Enter`). O seletor de conta fica pronto aos ~7,9 s; depois do Enter são 15 s fixos até abrir `story000` e 20-23 s de carga real: mundo = Enter + ~37 s (~47-52 s no total, contra 68-77). Os Enters de 75 s em diante caíam DENTRO do mundo (botão A) e mexiam o personagem: por isso cada rodada parava num lugar diferente.
2. **Janela estável:** sem entrada, tela preta em +120 quadros, pose parada na varanda a partir de +176, e a câmera ociosa corta para uma vista externa ~105-113 s depois do mundo. Medir com ~20 s de assentamento e terminar antes do corte.
3. **Capturas:** `VICTORY_CAPTURE=<pasta>` (só build de diagnóstico; meia resolução 960x540), `VICTORY_CAPTURE_WORLD=1` (primeiro quadro com 500+ draws marca o mundo; depois +2, +6, +30, +80 e 1 a cada 8), `VICTORY_CAPTURE_WORLD_ALSO=<deslocamentos>`, `VICTORY_CAPTURE_FRAMES=<lista>` (estes gravam também `frame-<n>-ops.txt` com uma linha por draw e as linhas `DROPPED <motivo>`), `VICTORY_DUMP_TARGETS=1`.
4. **`ab.py`** (cópia em (ferramenta local, fora do repositório)): casa os quadros pelo deslocamento desde o mundo, gera mapa de calor, lado a lado, recortes dos piores blocos e `metrics.json`. Uso: `python -I ab.py <rodadaA> <rodadaB> <saida> 264,304,360,400 --mask 460,130,760,440`. Piso de ruído: 1,5-2,9% de pixels acima de 32 entre rodadas iguais (com a máscara do personagem); diferença real só acima de ~4% com "match L1" abaixo de 12; sempre olhar os recortes.
5. **Replay de quadro gravado NÃO serve hoje para o mundo:** gravar no mundo faz o watchdog do jogo encerrá-lo (README:50; `pipeline/make.rs:7`); cobriria só mudanças do tradutor, não as decisões do host. `scripts/test-session.ps1` não existe.
6. Não existe chave para pular os vídeos de abertura.

## Ferramentas com administrador
- PresentMon 2.6.0: sem elevação dá "access denied" (precisa de administrador ou do grupo Performance Log Users). (ferramenta local, fora do repositório) roda com um clique de UAC via `Start-Process -Verb RunAs`. Colunas desta versão: `MsBetweenPresents`, `MsCPUBusy`, `MsCPUWait`, `MsGPUBusy`, `MsGPUWait`, `MsGPUTime`, `MsUntilDisplayed`, `TimeInMs`.
- `wpr -start CPU -filemode` funciona; `wpr -stop <arquivo>` falha com "Cannot change thread mode after it is set" (0x80010106), com e sem `-skipPdbGen`, e a sessão termina sozinha (conferido elevado: "WPR is not recording"). samply 0.13.1 grava via `xperf` (não instalado) e lê ETL com `samply import`. Próximo passo aprovado se precisar: instalar o Windows Performance Toolkit.
