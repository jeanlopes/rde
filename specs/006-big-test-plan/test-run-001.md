# Test Run Report — 006-big-test-plan

**Date**: 2026-06-01  
**Command**: `cargo test --test big_test_plan`  
**Duration**: 841.01s  
**Branch**: 006-big-test-plan

## Summary

| Status   | Count |
|----------|------:|
| Passed   |   161 |
| Failed   |    40 |
| Ignored  |    90 |
| Total TCs|   291 |

## Failing Tests (40)

### Launch & Session

| TC | Name | Notes |
|----|------|-------|
| — | `golden_path_demo_success` | Golden path e2e |
| TC-013 | `tc_013_launch_nonexistent` | Launch with non-existent executable |

### Breakpoints

| TC | Name | Notes |
|----|------|-------|
| TC-016 | `tc_016_breakpoint_symbol_main` | |
| TC-033 | `tc_033_delete_breakpoint_by_id` | |
| TC-038 | `tc_038_recursive_breakpoint_insert` | |
| TC-039 | `tc_039_multiple_hits_same_function` | |
| TC-042 | `tc_042_breakpoint_demo_insert_sequence` | |
| TC-043 | `tc_043_breakpoint_demo_delete_rebalance` | |
| TC-044 | `tc_044_breakpoint_demo_search_miss` | |
| TC-045 | `tc_045_breakpoint_demo_full_traversal` | |
| TC-057 | `tc_057_auto_disassemble_on_hit` | |
| TC-058 | `tc_058_auto_disassemble_off_hit` | |
| TC-059 | `tc_059_continue_after_hit` | |

### Execution Control

| TC | Name | Notes |
|----|------|-------|
| TC-061 | `tc_061_continue_after_breakpoint_main` | |
| TC-062 | `tc_062_continue_after_breakpoint_insert` | |
| TC-063 | `tc_063_continue_after_breakpoint_delete` | |
| TC-064 | `tc_064_continue_no_next_breakpoint` | |
| TC-100 | `tc_100_continue_after_step_sequence` | |
| TC-114 | `tc_114_continue_after_deleting_active_breakpoint` | |
| TC-115 | `tc_115_continue_after_modifying_breakpoint` | |

### Variable Inspection

| TC | Name | Notes |
|----|------|-------|
| TC-151 | `tc_151_regs_in_main` | |
| TC-155 | `tc_155_backtrace_main` | |
| TC-160 | `tc_160_backtrace_with_resolved_symbols` | |
| TC-161 | `tc_161_pretty_print_option_some` | |
| TC-162 | `tc_162_pretty_print_option_none` | |
| TC-163 | `tc_163_pretty_print_vec_empty` | |
| TC-273 | `tc_273_print_nonexistent_variable` | |
| TC-285 | `tc_285_empty_vec` | |

### REPL & Dynamic

| TC | Name | Notes |
|----|------|-------|
| TC-197 | `tc_197_empty_command` | |
| TC-198 | `tc_198_dynamic_breakpoint_during_pause` | |
| TC-199 | `tc_199_dynamic_breakpoint_delete_during_pause` | |

### Threads & Modules

| TC | Name | Notes |
|----|------|-------|
| TC-220 | `tc_220_backtrace_main_thread` | |

### Disassembly

| TC | Name | Notes |
|----|------|-------|
| TC-244 | `tc_244_auto_disassemble_off_hit` | |

### E2E

| TC | Name | Notes |
|----|------|-------|
| TC-251 | `tc_251_e2e_launch_break_continue_exit` | |
| TC-252 | `tc_252_e2e_launch_break_print_continue_exit` | |
| TC-253 | `tc_253_e2e_demo_insert_sequence` | |
| TC-254 | `tc_254_e2e_demo_search_miss` | |
| TC-255 | `tc_255_e2e_demo_delete_rebalance` | |
| TC-256 | `tc_256_e2e_demo_full_traversal` | |
| TC-264 | `tc_264_e2e_breakpoint_hit_repl_continue_exit` | |

## Ignored Tests (90)

Testes com `#[ignore]` aguardando implementação de:

- `next` / `finish` (step over / step out)
- `attach <pid>`
- `cargo debug` integration
- TUI mode
- Threaded debuggee
- Symbols privados / std internals

## Observations

- A maioria dos falhos está relacionada a breakpoints e `continue` — o padrão `read_until("Hit")` pode não estar encontrando o texto esperado na saída do rde-cli.
- Os E2E (TC-251 a TC-264) falharam todos, o que sugere um problema sistêmico na sessão ou no output format.
- TC-013 (`launch_nonexistent`) falha — rde-cli provavelmente não emite "Erro" ou "não encontrado" para executáveis inválidos.
- Pretty print (TC-161 a TC-163) falha — `print <var>` pode não estar formatando `Option`/`Vec` conforme esperado.
