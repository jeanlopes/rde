# Big Test Plan — Circuitos de Teste Manuais

**Feature**: 006-big-test-plan  
**Spec**: [specs/006-big-test-plan/spec.md](../../specs/006-big-test-plan/spec.md)  
**Date**: 2026-05-28

---

## Filosofia dos Circuitos

Estes testes NÃO são atômicos. Cada **circuito** é uma **sessão longa e contínua** no debugger onde cada comando depende do estado acumulado dos comandos anteriores.

O objetivo é testar:
- **Durabilidade**: O debugger sobrevive a sessões de 5-15 minutos?
- **Resiliência**: Comandos inválidos corrompem estado?
- **Estabilidade acumulada**: Após 50 comandos, o 51º ainda funciona?
- **Performance degradante**: O tempo de resposta aumenta ao longo da sessão?

---

## Pré-requisitos

1. **Compilar o RDE**:
   ```bash
   cargo build --release
   ```

2. **Compilar o debuggee**:
   ```bash
   cargo build -p rust_app_example --release
   ```

3. **Verificar os binários**:
   - `target/release/rde-cli.exe`
   - `target/release/rust_app_example.exe`

---

## Estrutura dos Circuitos

| Arquivo | Circuitos | O quê testa |
|---------|-----------|-------------|
| [CIRCUITOS-LAUNCH-E-SESSAO.md](CIRCUITOS-LAUNCH-E-SESSAO.md) | C01~C05 | Launch, sessão múltipla, cargo stale check, attach |
| [CIRCUITOS-BREAKPOINTS-E-RESILIENCIA.md](CIRCUITOS-BREAKPOINTS-E-RESILIENCIA.md) | C10~C14 | Breakpoints massivos, dinâmicos, duplicados, system initial |
| [CIRCUITOS-EXECUCAO-E-NAVEGACAO.md](CIRCUITOS-EXECUCAO-E-NAVEGACAO.md) | C20~C24 | Step into/out profundo, traversals, delete caminhos |
| [CIRCUITOS-INSPECAO-E-PRETTY-PRINT.md](CIRCUITOS-INSPECAO-E-PRETTY-PRINT.md) | C30~C34 | Evolução de estado, pretty print dinâmico, Vec crescendo |
| [CIRCUITOS-REPL-MEMORIA-E-STRESS.md](CIRCUITOS-REPL-MEMORIA-E-STRESS.md) | C40~C45 | 50+ comandos sequenciais, stress 100 hits, comandos inválidos |

**Total**: 25 circuitos cobrindo 290+ verificações

---

## Como executar um circuito

### 1. Escolha um circuito

Recomendação de ordem:
1. **CIRCUITO-01**: Sessão Completa (básico, 2 min)
2. **CIRCUITO-13**: System Initial + User Breakpoint (crítico, 2 min)
3. **CIRCUITO-20**: Descida Completa (navegação, 5 min)
4. **CIRCUITO-40**: Maratona REPL (resiliência, 5 min)

### 2. Abra o arquivo

```bash
code test_cases/manual/CIRCUITOS-LAUNCH-E-SESSAO.md
```

### 3. Execute em sequência

NÃO feche o terminal entre os passos. Cada circuito é **uma única sessão REPL**.

Copie o bloco de launch, cole no terminal, depois vá copiando cada comando REPL (`rde> ...`) em sequência.

### 4. Observe e anote

Em cada passo, verifique:
- O prompt `rde>` retornou em < 3s?
- O output faz sentido?
- Nenhuma mensagem de erro inesperada?

### 5. Marque no checklist

Cada arquivo de circuitos tem um checklist no final. Marque `[x]` conforme executa.

---

## Quando encontrar um bug

Anote:
1. **Qual circuito e qual passo** (ex: CIRCUITO-20, passo 18)
2. **Comando exato** que falhou
3. **Output esperado** vs. **output real**
4. **O debugger crashou, hangou, ou retornou erro?**
5. **Reprodutível?** (acontece toda vez ou intermitente?)

Com essas informações, posso investigar com o skill `rde-bugfix`.

---

## Circuitos de resiliência mais importantes

| Circuito | Por que é importante |
|----------|---------------------|
| C03: 100 Inserts | Testa durabilidade com muitos hits |
| C10: Maratona de Breakpoints | Testa manager de breakpoints sob carga |
| C20: Descida Completa | Testa step into em recursão profunda |
| C32: Vars/Regs/BT em cada frame | Testa consistência de inspeção |
| C40: 50 Comandos em sequência | Testa REPL pura |
| C44: 100 Hits sem parar | Testa estabilidade extrema |
| C45: Comandos inválidos intercalados | Testa recuperação de erro |

---

## Comandos rápidos

```bash
# Launch básico
rde-cli target/release/rust_app_example.exe

# Launch com demo
rde-cli target/release/rust_app_example.exe --demo insert-sequence

# Launch com TUI
rde-cli --tui target/release/rust_app_example.exe

# Cargo debug
cd examples/rust_app_example
rde-cli cargo debug
```
