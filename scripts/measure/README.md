# Medidor

Ferramentas para medir FPS e comparar imagem sempre do mesmo jeito: abrem o jogo sozinhas pelo `scripts\play.ps1`, esperam o mundo, medem e fecham. Os resultados e as capturas ficam fora do repositório (por padrão em `%TEMP%`) e nunca vão para o GitHub.

## Antes de medir
- Windows, Python 3.9 ou mais novo, `pip install pillow numpy` (só para `ab.py` e `vis360.py`).
- `VICTORY_GAME_DIR` com a pasta da sua cópia do jogo (o executável é `game_scarlett_master.exe`, como no `Play-VIctory.cmd`).
- O build de diagnóstico (`cargo build --locked --profile diag -p victory --features diagnostics`); no Windows 10, `cargo xtask agility` antes. O build de release só dá o FPS do título da janela.
- Cache aquecido, o MESMO que a rodada vai usar. A rota nvidia usa `%LOCALAPPDATA%\VIctory\cache-pareto` quando `VICTORY_CACHE_DIR` não está definida; a rota amd usa o cache normal. Defina `VICTORY_CACHE_DIR` na janela (o `play.ps1` a mantém) e aqueça com ela: `target\diag\victory.exe shaders <arquivos .rpf>` e `target\diag\victory.exe warm --used`.
- Nada mais rodando: o medidor recusa começar com `cargo`/`rustc` abertos ou com outro `victory.exe` aberto.
- Meça pela **rota** da sua placa (`--route nvidia` numa GeForce, `--route amd` numa Radeon).

## Uma rodada: `bench.py`
```powershell
python scripts\measure\bench.py --name base --route nvidia --vars "VICTORY_HAIR=1"
```
Injeta um Enter aos 10 s (o seletor de conta) e outro aos 14 s; nenhum no mundo, onde Enter é o botão A e mexe o personagem. Outro save pode pedir outras teclas: `VICTORY_BENCH_KEYS` (no formato de `VICTORY_KEYS`) e `VICTORY_BENCH_WORLD_ALSO` (os quadros capturados para o `ab.py`) mudam os padrões. Espera o mundo (1.500 draws no título), assenta 20 s e mede 75 s. A rodada só vale se o jogo viver a amostra inteira; fechar a janela à mão ou sair do mundo a invalida, com o motivo escrito. O Fechar da janela fica cinza durante a rodada. Saída: `result.json` com FPS (mediana e 1000/ms), tempo de quadro, draws, a divisão do quadro (preparo, texturas, placa, cópia de volta), placa ocupada, draws descartados por motivo, memória privada no pico, RAM livre mínima, VRAM dedicada e compartilhada, acertos do cache de shaders, device removed e pânicos.

O p95 sai das leituras do título a cada 2 s, que já são médias: não é p95 quadro a quadro, e não há 1% low. Para isso, use PresentMon.

## Uma matriz: `matrix.py`
```powershell
python scripts\measure\matrix.py scripts\measure\matrix-example.json
```
Cada célula roda 3 rodadas em ordem alternada, depois de um aquecimento descartado por rota, console e build. Uma célula que muda a tradução (`VICTORY_WAVE_WIDE`, por exemplo) precisa de `warm_key` próprio. Rodada inválida se repete até 2 vezes. Saída: `runs.csv` (uma linha por rodada) e `summary.csv` (mediana das rodadas válidas por célula). Ao fim, ou se parar no meio, o medidor fecha só o jogo que ele mesmo abriu.

As matrizes de 27/11 (a do #29, por exemplo) usaram outro kit: `--route amd --safe`, assentamento de 45 s e Enters até 120 s. Para repeti-las, use `"route": "amd", "safe": true, "settle_s": 45` e `VICTORY_BENCH_KEYS`. Medições novas vão pela rota da placa.

## Imagem num ponto fixo: `ab.py`
```powershell
python -I scripts\measure\ab.py <rodada A> <rodada B> <saída> 264,304,360,400 --mask 460,130,760,440
```
Compara os quadros 264, 304, 360 e 400 depois do primeiro quadro do mundo (o `bench.py` já os captura), escolhendo em B a vista mais parecida até 40 quadros de distância. A máscara tira o personagem. Saída: lado a lado, mapa de diferença, recortes dos piores blocos e `metrics.json`. Entre rodadas iguais, 1,5 a 2,9% dos pixels passam de 32: trate como diferença real só acima de ~4% com `view_match_L1` abaixo de 12, e olhe os recortes.

## Giro de câmera: `vis360.py`
Precisa de um build com as chaves de captura por tempo (`VICTORY_CAPTURE_EVERY_MS`, `VICTORY_CAPTURE_BURST_MS` e os eixos em `VICTORY_KEYS`).
```powershell
$giro = "VICTORY_KEYS=w22-w34:RX+;VICTORY_CAPTURE_EVERY_MS=500;VICTORY_CAPTURE_BURST_MS=20000:10,40000:10"
python scripts\measure\vis360.py run base nvidia diag $giro 45
python scripts\measure\vis360.py run cand nvidia diag $giro 45        # com VIS_VICTORY=<outro victory.exe>
python scripts\measure\vis360.py compare <rodada base> <rodada cand> <saída> --giro 21-35 --mask 460,130,760,440
```
O roteiro: mundo, 20 s parado, rajada de 10 quadros seguidos, giro de 360 graus em 12 s com um print a cada 0,5 s, outra rajada aos 40 s, e fecha aos 45 s. O `compare` casa as capturas pelo tempo desde o primeiro quadro do mundo, então o mesmo ângulo cai no mesmo par mesmo com FPS diferente. Saída: uma folha de contato por ângulo (base, candidato, diferença), o pisca de cada rajada (um bloco que muda num quadro e volta no seguinte) e os draws descartados por motivo.

## Tempo de cada quadro com o PresentMon: `presentmon.ps1`
O título da janela dá o FPS médio; o PresentMon dá o tempo de cada quadro, de onde saem a mediana, o p95, o p99 e as travadas. Ele mede o jogo que **já está aberto**: abra pelo `Play-VIctory.cmd` ou pelo `scripts\play.ps1`, chegue ao mundo e rode em outra janela.

```powershell
scripts\measure\presentmon.ps1 -PresentMon <caminho do PresentMon-x64.exe> -Seconds 60
scripts\measure\presentmon.ps1 -Csv <um frames.csv já gravado>      # só o resumo
```

- O PresentMon (Intel, `github.com/GameTechDev/PresentMon`) não vem no repositório: baixe o executável de console e passe o caminho.
- Gravar pede uma janela do PowerShell como administrador, ou o usuário no grupo "Performance Log Users". O jogo nunca roda como administrador e este script não o abre nem fecha.
- Saída: `frames.csv` e `summary.txt` numa pasta nova em `%TEMP%` (ou em `-Out`), nunca no repositório.
- O resumo traz quadros, FPS médio, mediana, p95, p99 e o quadro mais longo, o 1% mais lento (1000 / p99, posto mais próximo), quantos quadros passaram de 150 ms, as médias de `MsCPUBusy`, `MsCPUWait`, `MsGPUBusy` e `MsGPUWait`, o intervalo de sincronização e o modo de apresentação.
- Para o perfil do processador (onde o tempo de preparo é gasto) ainda não há ferramenta aqui: o `wpr` do Windows gravou e falhou ao salvar na máquina das medições.
