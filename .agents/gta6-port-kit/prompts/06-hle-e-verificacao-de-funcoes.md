# Sub-prompt 06 — HLE, imports e verificação de comportamento de funções

Este runtime não recompila o código da CPU: o x64 do título roda nativo e as APIs do console
são reimplementadas (`victory-hle`). "Verificar uma função" aqui = provar que nossa
implementação de um import responde como o console espera.

## Passos
1. `victory census` (sem executar nada) mostra imports cobertos, passados ao host, servidos
   por DLL do título ou stub. Liste os stubs que o título chama de fato (log `--unimplemented
   zero` loga uma vez; `trap` para no debugger).
2. Para cada import: documentação pública do GDK (Microsoft Learn) para assinatura e
   semântica; nada de copiar headers proprietários para o repo.
3. Teste no fixture `tests/fixtures/hle-probe` (código nosso) cobrindo retorno, erros e
   efeitos colaterais.
4. Se usar Ghidra/IDA localmente para entender uma chamada do título, nenhum pseudo-código,
   disassembly ou offset fora do que `docs/asset-policy.md` permite vai para o repo. Use
   assinaturas de bytes e nomes escolhidos por nós.
5. Assistentes de IA em decompilador (GhidraMCP, ida-pro-mcp, DAILA) são aceitáveis
   localmente; trate a saída deles como hipótese, nunca como fato, e não cole no repo.

## Prova
Teste vermelho → verde; e, se aplicável, run do título onde o caminho passa sem stub.
