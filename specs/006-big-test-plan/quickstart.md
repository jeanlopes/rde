# Quickstart: Big Test Plan

**Feature**: 006-big-test-plan
**Date**: 2026-05-28

---

## Prerequisites

- Windows 10/11 x86-64
- Rust toolchain (stable, MSRV 1.78+)
- `cargo` em PATH
- RDE compilado: `cargo build --release` (produz `target/release/rde-cli.exe`)

---

## 1. Build the Debuggee

```bash
cd examples/rust_app_example
cargo build --release
```

O binário será produzido em `examples/rust_app_example/target/release/rust_app_example.exe` (se projeto independente) ou `target/release/examples/rust_app_example.exe` (se workspace member).

---

## 2. Run a Single Test Case

```bash
cargo test --test big_test_plan tc_001 -- --nocapture
```

Para executar um caso específico, use o nome da função de teste:

```bash
cargo test --test big_test_plan tc_042 -- --nocapture
```

---

## 3. Run a Category of Tests

Os testes são organizados por módulos internos no arquivo `big_test_plan.rs`:

```bash
# Launch & Session tests (TC-001 to TC-015)
cargo test --test big_test_plan launch:: -- --nocapture

# Breakpoint tests (TC-016 to TC-060)
cargo test --test big_test_plan breakpoints:: -- --nocapture

# Execution Control tests (TC-061 to TC-120)
cargo test --test big_test_plan execution:: -- --nocapture

# Variable Inspection tests (TC-121 to TC-160)
cargo test --test big_test_plan inspection:: -- --nocapture

# Pretty Printing tests (TC-161 to TC-185)
cargo test --test big_test_plan pretty_print:: -- --nocapture

# REPL Runtime tests (TC-186 to TC-210)
cargo test --test big_test_plan repl:: -- --nocapture

# Thread/Module/Task tests (TC-211 to TC-230)
cargo test --test big_test_plan threads:: -- --nocapture

# Memory/Disassembly tests (TC-231 to TC-250)
cargo test --test big_test_plan memory:: -- --nocapture

# E2E Integration tests (TC-251 to TC-270)
cargo test --test big_test_plan e2e:: -- --nocapture

# Error Handling tests (TC-271 to TC-290)
cargo test --test big_test_plan errors:: -- --nocapture
```

---

## 4. Run the Full Suite

```bash
cargo test --test big_test_plan
```

---

## 5. Update Golden Paths

Se o comportamento do rde-cli mudar intencionalmente (ex: formatação de output atualizada), atualize os golden paths:

```bash
# Se usando insta
INSTA_UPDATE=always cargo test --test big_test_plan

# Se usando comparação manual
cargo test --test big_test_plan -- --bless
```

---

## 6. Debug a Failing Test

```bash
# Ver logs detalhados do harness
RUST_LOG=trace cargo test --test big_test_plan tc_042 -- --nocapture

# Ver output bruto do rde-cli
RUST_LOG=trace RDE_TEST_RAW_OUTPUT=1 cargo test --test big_test_plan tc_042 -- --nocapture
```

---

## 7. Manual REPL Session

Para explorar o debuggee manualmente:

```bash
# Standalone executable
rde-cli examples/rust_app_example/target/release/rust_app_example.exe --demo insert-sequence

# From Cargo project
cd examples/rust_app_example
rde-cli cargo debug -- --demo insert-sequence
```

No REPL:
```
rde> break main
rde> continue
rde> print root
rde> step
rde> regs
rde> continue
rde> quit
```

---

## 8. Verify a New Test Case

Para adicionar um novo caso de teste:

1. Copie um caso existente (ex: `tc_001`) como template.
2. Atualize o doc comment com o novo ID, cenário e critério de sucesso.
3. Execute o caso isolado para verificar.
4. Se produzir um golden path novo, adicione-o em `test_data/golden_paths/`.
5. Execute a categoria completa para garantir que não há regressões.

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| `rde-cli not found` | Compile o RDE: `cargo build --release` |
| `rust_app_example.exe not found` | Compile o debuggee: `cd examples/rust_app_example && cargo build --release` |
| Test hangs | O rde-cli pode estar esperando input. Verifique se o harness enviou `quit`. |
| Golden path mismatch | Endereços hex podem não estar normalizados. Verifique o regex de normalização no harness. |
| "No Tokio runtime detected" | Comportamento esperado para o debuggee padrão. Não é um erro. |
| Breakpoint não dispara | Verifique se o símbolo existe com `nm` ou `dumpbin /symbols`. |
