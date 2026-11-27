# Regras de trabalho aprendidas (26 e 27/11/2026)

1. **Meça pela rota da placa.** Uma noite inteira de medições foi feita pela rota amd numa GeForce. A configuração de execução é parte do experimento: confira com `-DryRun` antes do primeiro número.
2. **Compare só no mesmo ponto fixo.** Cenas com 4 a 11 mil draws não se comparam. Mesma cena, mesma câmera, três rodadas, ou não chame de resultado.
3. **Um build de jogar por vez.** Consertos espalhados em branches não chegam a quem joga. Integre o que está provado num build só; o candidato só toma o lugar depois de passar na comparação de imagem e de FPS.
4. **Não prometa o que não foi visto.** Previsão só com evidência do mesmo build e da mesma cena. Senão, diga "não sei".
5. **Diga na primeira linha quando nada melhorou.**
6. **"Jogável" não é FPS.** São os dois pisos juntos. Troca entre imagem e FPS vai para decisão humana, com números e imagens.
7. **Releia os achados antes de aceitar uma hipótese nova.** Duas causas trazidas por uma nota externa já estavam descartadas em `findings/`; foram portadas de novo por falta de releitura.
8. **Leia os draws descartados antes de olhar a imagem.** A linha `dropped N/frame without a host pipeline: <motivo>` do build de diagnóstico diz o que falta no quadro. Sem ela, ~300 draws do ray tracing foram tomados por objetos sumindo.
9. **Nada de link pesado com o jogo aberto.** O jogo usa ~24 GB em 32; um link derrubou a RAM livre a 480 MB. Prioridade baixa protege o processador, não a memória.
10. **Uma trava para o jogo e outra para o build** quando há mais de uma sessão na mesma máquina. Ninguém compila durante medição de FPS.
11. **Agente parado é desperdício.** Deixe sempre trabalho sem compilação na fila: leitura, análise de log, escrita de patch.
12. **Memória só existe em arquivo.** Estado, achados e decisões vão para `docs/agents/` no mesmo PR do trabalho.
13. **Nada do jogo no GitHub:** nem imagem, nem log bruto, nem caminho local.
