---
project: vai-funcionar
type: lessons
updated: 2026-11-27
---

# Lições (o princípio por trás de cada correção do Bruno)

1. **Ler como o repo manda rodar antes de medir qualquer coisa.** O `CLAUDE.md` já dizia que `Play-VIctory.cmd` é a rota testada em RTX e `play.ps1` a rota AMD; medimos a noite inteira com a rota errada. *Princípio: a configuração de execução é parte do experimento; conferir o caminho oficial para o hardware antes do primeiro número.*
2. **Entregar um resultado que o usuário vê, não seis builds de laboratório.** Espalhei consertos em worktrees separadas e ele só viu janelas abrindo e fechando. *Princípio: integrar o que já está provado num build único e mostrar esse; medir peças separadas longe do usuário.*
3. **Não prometer o que não foi visto.** Disse que a persiana "deve" ficar sem roxo com o filtro; só mudou de tom. *Princípio: previsão só com evidência do mesmo build e da mesma cena; senão dizer "não sei".*
4. **Dizer na primeira linha quando nada melhorou.** *Princípio: o usuário decide com a verdade, não com o esforço.*
5. **Perguntas no formato grilling, numeradas sem repetir números, com recomendação; fatos eu descubro.** Ele não é da área e não consegue responder pergunta aberta.
6. **Não pedir aprovação para leitura nem parar depois de cada mensagem.** *Princípio: autonomia dentro do combinado; chamar só para decisão real.*
7. **Avisar antes de abrir o jogo, um teste por vez, injetar o Enter inicial, e não repetir rodadas na frente dele.** Rodadas de comparação lentas de propósito parecem regressão para quem olha.
8. **"Jogável" não é FPS.** 30 FPS com boneco preto não é jogável. *Princípio: dois pisos juntos; trade-off vai para ele com números e imagens.*
9. **Falar a língua dele.** Glossário em `GLOSSARY.md`; "compilar" só para código. *Princípio: termo técnico só com definição.*
10. **Agente parado é desperdício.** O worker ficou 40 min esperando a fila de build. *Princípio: sempre deixar trabalho sem compilação (leitura, análise de log, escrita de patch) na fila de cada agente.*
11. **Persistir o estado em arquivo antes de compactar.** *Princípio: memória só existe em arquivo.*
12. **Comparar FPS só no mesmo ponto fixo.** Meus testes T1-T3 tiveram cenas com 4 a 11 mil draws e não permitem conclusão. *Princípio: mesma cena, mesma câmera, mais de uma rodada, ou não chamar de resultado.*
13. **Mudança que precisa de administrador custa a paciência do usuário.** Três cliques de UAC para uma gravação que falhou. *Princípio: testar o caminho inteiro sem elevação até onde der antes de pedir o clique.*
14. **Nada de link pesado com o jogo aberto.** O jogo com o kit NVIDIA usa ~24 GB privados numa máquina de 31 GB. Em 27/11 o link de um build de diagnóstico (worker, prioridade Idle) derrubou a RAM livre a 480 MB com o Bruno jogando. Prioridade Idle protege o processador, não a memória. **Princípio:** com o jogo aberto, só `cargo check` e testes leves; build completo espera a linha JOGO ficar livre.
15. **Reler os próprios relatórios depois de uma compactação, antes de despachar trabalho.** Em 27/11 uma nota externa apontou duas causas (recusa de tabelas, corte em 65.535) que a pesquisa da véspera já tinha descartado para a nossa cena. Sem reler `research/`, mandei o worker portar as duas e disse ao Bruno que ~300 draws descartados eram objetos sumindo, quando o relatório dizia que eram estágios do ray tracing. **Princípio:** o resumo da compactação não substitui os relatórios; antes de aceitar uma hipótese nova, procurar por ela em `research/`.
16. **Tirar caminho local e pasta do jogo de todo texto gerado por ferramenta antes de publicar.** O kit impresso pelo `-DryRun` levava `VICTORY_GAME_DIR` e o perfil do usuário para o corpo de um PR; o bloqueio pegou antes de publicar. *Princípio: saída de ferramenta carrega dados da máquina; revisar e trocar por marcadores antes de mandar ao GitHub.*
17. **Automação só fecha o que ela abriu.** O `matrix.py` matava qualquer `victory.exe` ao sair, inclusive um jogo aberto pelo usuário (achado na revisão do PR #35). *Princípio: uma ferramenta não destrói nada fora do escopo que ela mesma criou.*
18. **Conferir antes de repetir o estado de outra sessão.** Disse que a suíte do #34 ainda rodava no merge porque a linha BUILD continuava no quadro; ela já tinha terminado. *Princípio: uma linha de status é indício, não prova; a fonte (log, resultado, commit) decide.*
19. **Não inventar camada de coordenação.** Criei um quadro e um arquivo de trava para três sessões; o Bruno encerrou os dois: "tudo segue o SOP e as regras que estão lá, em conjunto com /setup-matt-pocock-skills". *Princípio: estado nos arquivos do SOP, trabalho em issue e PR, vocabulário no glossário; nada paralelo.*
20. **Revisão independente antes do merge, mesmo com pressa.** Uma sessão só de leitura achou um defeito real num commit portado (o ramo do heap limitado pegava o heap de samplers) e um conserto que não provava nada (filtro de magenta). Os três PRs limpos entraram na hora; o quarto, depois do conserto e da suíte. *Princípio: ordem de "faz o merge" se cumpre já no que está limpo e em minutos no resto, nunca com defeito conhecido.*
