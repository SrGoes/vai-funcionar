# Medidor

Ferramentas para medir FPS e comparar imagem sempre do mesmo jeito: abrem o jogo sozinhas pelo `scripts\play.ps1`, esperam o mundo, medem e fecham. Os resultados e as capturas ficam fora do repositório (por padrão em `%TEMP%`) e nunca vão para o GitHub.

## Antes de medir
- Windows, Python 3.9 ou mais novo, `pip install pillow numpy` (só para `ab.py` e `vis360.py`).
- `VICTORY_GAME_DIR` com a pasta da sua cópia do jogo (`VICTORY_GAME_EXE` muda o nome do executável; padrão `game_scarlett_master.exe`).
- O build de diagnóstico (`cargo build --locked --profile diag -p victory --features diagnostics`); no Windows 10, `cargo xtask agility` antes. O build de release só dá o FPS do título da janela.
- Cache aquecido: `target\diag\victory.exe shaders <arquivos .rpf>` e `target\diag\victory.exe warm --used`.
- Nada mais rodando: o medidor recusa começar com `cargo`/`rustc` abertos ou com outro `victory.exe` aberto.
- Meça pela **rota** da sua placa (`--route nvidia` numa GeForce, `--route amd` numa Radeon).

## Uma rodada: `bench.py`
```powershell
python scripts\measure\bench.py --name base --route nvidia --vars "VICTORY_HAIR=1"
```
Injeta um Enter aos 10 s (o seletor de conta) e outro aos 14 s; nenhum no mundo, onde Enter é o botão A e mexe o personagem. Espera o mundo (1.500 draws no título), assenta 20 s e mede 75 s. A rodada só vale se o jogo viver a amostra inteira; fechar a janela à mão ou sair do mundo a invalida, com o motivo escrito. O Fechar da janela fica cinza durante a rodada. Saída: `result.json` com FPS (mediana e 1000/ms), tempo de quadro, draws, a divisão do quadro (preparo, texturas, placa, cópia de volta), placa ocupada, draws descartados por motivo, memória privada no pico, RAM livre mínima, VRAM dedicada e compartilhada, acertos do cache de shaders, device removed e pânicos.

O p95 sai das leituras do título a cada 2 s, que já são médias: não é p95 quadro a quadro, e não há 1% low. Para isso, use PresentMon.

## Uma matriz: `matrix.py`
```powershell
python scripts\measure\matrix.py scripts\measure\matrix-example.json
```
Cada célula roda 3 rodadas em ordem alternada, depois de um aquecimento descartado por build. Uma célula que muda a tradução (`VICTORY_WAVE_WIDE`, por exemplo) precisa de `warm_key` próprio. Rodada inválida se repete até 2 vezes. Saída: `runs.csv` (uma linha por rodada) e `summary.csv` (mediana das rodadas válidas por célula).

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
