# vai-funcionar

Vocabulário do trabalho de fazer um jogo de Xbox Series rodar no Windows com placa NVIDIA, com imagem de console e desempenho jogável. Serve para o Bruno e os agentes falarem a mesma língua.

## Language

### O que é o quê

**Port**:
O programa que faz o jogo de Xbox rodar no Windows (`victory.exe`). Ele não é o jogo: carrega o jogo e responde no lugar do console.
_Avoid_: emulador, launcher, runtime, VIctory

**Jogo**:
O título de Xbox, intocado, que o port carrega. Nunca é alterado nem aberto direto.
_Avoid_: título, exe, dump

**Build**:
Uma versão compilada do port, identificada pelo commit de onde saiu. Dois builds diferentes podem dar imagem e FPS diferentes com o mesmo jogo.
_Avoid_: binário, versão, exe

**Build de diagnóstico**:
Build que registra medições detalhadas no log. É o usado em toda medição.
_Avoid_: diag, debug

**Chave**:
Uma configuração do port que liga, desliga ou ajusta um comportamento sem recompilar (as `VICTORY_*`).
_Avoid_: flag, switch, variável, opção

**Kit de chaves**:
Um conjunto de chaves aplicado junto por um script de abertura. Dois kits diferentes não são comparáveis.
_Avoid_: perfil, preset, configuração

**Rota**:
O kit de chaves feito para uma marca de placa: rota nvidia (`Play-VIctory.cmd`, `scripts\play.ps1 -Route nvidia`) ou rota amd (`scripts\play.ps1 -Route amd`). Medir e jogar sempre pela rota da placa; números de rotas diferentes não se comparam.
_Avoid_: launcher, perfil, modo

**Perfil de console**:
Qual Xbox o port finge ser, Series S ou Series X. Muda a resolução e a memória que o jogo usa.
_Avoid_: modo, console

### Imagem

**Shader**:
Um programa pequeno que a placa de vídeo roda para desenhar ou calcular uma coisa (a pele, a água, a luz de um cômodo). O jogo tem cerca de 25 mil.
_Avoid_: efeito, gráfico

**Textura**:
A imagem aplicada sobre uma superfície (a estampa de uma camisa, o desenho de uma tatuagem). Não é a luz nem a cor final na tela.
_Avoid_: gráfico, skin

**Material**:
Como uma superfície reage à luz: textura mais as regras do shader. Um material pode aparecer errado com a textura certa.
_Avoid_: textura (quando o problema é a luz)

**Iluminação do jogo**:
A luz indireta e as sombras, que o jogo calcula por ray tracing. Sem ela, o que o sol não atinge direto fica preto.
_Avoid_: RT, luz

**Ray tracing**:
A técnica com que o jogo calcula a iluminação do jogo. No port é um conjunto de shaders do próprio jogo que precisa ser traduzido, e hoje não roda na NVIDIA.
_Avoid_: RT, raios

**Luz substituta**:
Uma estimativa barata que o port põe no lugar da iluminação do jogo enquanto o ray tracing não roda. É a origem provável do roxo, dos retângulos na água e da imagem lavada.
_Avoid_: ambient, stand-in, substituto de luz

**Defeito visual**:
Qualquer diferença visível entre a imagem do port e a do console.
_Avoid_: bug gráfico, glitch, erro de shader

**Defeito que bloqueia**:
Defeito visual que se nota sem procurar e quebra a cena: personagem sem textura ou careca, objeto que pisca ou some, retângulos na água, material roxo ou preto. Deixa o jogo abaixo do piso visual.

**Defeito que degrada**:
Defeito visual que se vê mas não quebra a cena: iluminação pobre sem ray tracing (pele pálida, sem reflexo, aspecto de massinha), névoa estranha. O jogo continua acima do piso visual.

**Defeito tolerável**:
Defeito visual que só aparece procurando ou some sozinho em segundos.

### Tradução de shaders

**Tradutor**:
A parte do port que converte cada shader do formato do Xbox para o formato que a placa do PC executa.
_Avoid_: compilador, conversor

**Shader sem tradução**:
Shader que o tradutor não consegue converter. O que ele desenharia ou calcularia simplesmente não acontece. É defeito a corrigir, nunca resultado aceitável.
_Avoid_: shader recusado, não suportado, shader preto

**Grupo de 64 / grupo de 32**:
Quantos pontos a placa processa juntos num shader. O Xbox usa grupos de 64 em parte dos shaders; a NVIDIA só faz grupos de 32. Converter de um para o outro é o trabalho mais difícil do tradutor.
_Avoid_: wave64, wave32, lanes, warp

**Pânico do tradutor**:
O tradutor aborta por ter gerado um shader inválido, em vez de dizer que não sabe converter.
_Avoid_: crash, erro de shader

**Pré-aquecimento**:
Traduzir todos os shaders antes de abrir o jogo, para não traduzir durante a partida.
_Avoid_: warm, cache de shaders, compilar shaders

### Desempenho

**FPS**:
Quadros novos desenhados por segundo, medidos pelo próprio port. Sempre a mediana de uma amostra, nunca um número visto de relance.

**Quadro**:
Uma imagem completa desenhada pelo port. O tempo de um quadro é o inverso do FPS (83 ms = 12 FPS).
_Avoid_: frame

**Draw**:
Um pedido do jogo para desenhar um objeto. Milhares por quadro.
_Avoid_: draw call, chamada

**Draw descartado**:
Draw que o port não executa porque o shader dele não tem tradução ou ainda não está pronto. O objeto não aparece ou aparece errado.
_Avoid_: dropped

**Preparo**:
O trabalho que o port faz no processador antes de mandar cada lote de draws para a placa. É hoje o que mais limita o FPS.
_Avoid_: submissão, thread de preparo, CPU

**Placa esperando**:
A placa de vídeo marca uso alto mas consome pouca energia, porque passa o tempo esperando dados que estão na memória do computador e não na dela.
_Avoid_: GPU ociosa, GPU a 99%

**Travada**:
Um quadro isolado muito mais longo que os vizinhos, sentido como engasgo. Diferente de FPS baixo, que é constante.
_Avoid_: stutter, lag

### Medição

**Medidor**:
A ferramenta que abre o jogo sozinha, carrega o save, espera a cena assentar e mede 75 segundos.
_Avoid_: bench, benchmark, script

**Rodada**:
Uma abertura do jogo pelo medidor, do início ao fim. Só vale se completar os 75 segundos.
_Avoid_: run, execução

**Aquecimento**:
Rodada feita antes das que contam e descartada, para os caches ficarem prontos.
_Avoid_: warmup

**Matriz**:
Várias configurações medidas na mesma cena, três rodadas cada, em ordem alternada. É o que decide entre duas opções.
_Avoid_: teste, bateria

**Célula**:
Uma das configurações de uma matriz.

**Cena de medição**:
O lugar fixo do save onde toda rodada mede. Hoje é a casa.
_Avoid_: cenário, rota (no sentido de caminho pelo mapa)

### O norte

**Jogável**:
O jogo acima dos dois pisos ao mesmo tempo, o visual e o de desempenho. FPS alto com defeito que bloqueia não é jogável.

**Piso visual**:
Nenhum defeito que bloqueia: personagens com textura e cabelo, nada piscando ou sumindo, sem retângulos na água, sem materiais roxos ou pretos.

**Piso de desempenho**:
Casa dos 20 a 30 FPS, com 30 como mínimo aceitável a alcançar.

**Meta**:
Imagem igual à do console e 60 FPS.

### Trabalho

**Conserto**:
Mudança de código que remove a causa de um defeito.
_Avoid_: fix, patch, correção

**Paliativo**:
Chave ou mudança que esconde um defeito sem remover a causa. Sempre temporário e registrado.
_Avoid_: workaround, gambiarra

**Regressão**:
Algo que funcionava num build e deixou de funcionar num build mais novo.

**Conserto das cópias**:
O conserto que o Goes trouxe de outro port e que faz certos dados do jogo chegarem certos na NVIDIA. Na forma original derrubou o FPS de 12 para 3.
_Avoid_: correção do Belobug, cópias de views, 8977bb0, VIEW_BYTES

**Reaproveitamento**:
O conserto do worker que mantém o conserto das cópias ligado sem o custo de FPS.
_Avoid_: patch do worker, fix do #29, churn

**Quadro de coordenação**:
O arquivo onde cada sessão registra o que vai fazer antes de compilar, abrir o jogo ou criar issue ou PR.
_Avoid_: board, COORDENACAO

**Sessão**:
Um agente com janela e contexto próprios: main (coordena), worker (escreve código), pi (mede e integra).

**Scout**:
Um subagente temporário que só lê e devolve um relatório.
