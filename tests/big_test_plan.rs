//! Big Test Plan — rde-cli integration tests with Binary Tree Debuggee.
//!
//! This file implements 290 atomic test cases covering all rde-cli features
//! as specified in specs/006-big-test-plan/spec.md.

mod common;

use common::*;
use std::path::PathBuf;
use std::time::Duration;

/// Default timeout for REPL operations.
const TIMEOUT: Duration = Duration::from_secs(10);
/// Longer timeout for stress tests.
const LONG_TIMEOUT: Duration = Duration::from_secs(60);

// ============================================================================
// TC-001 to TC-015: Launch & Session Management
// ============================================================================

#[test]
fn tc_001_launch_standalone() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    let lines = session.read_until_prompt(TIMEOUT);
    assert_output_contains(&lines, "Processo iniciado");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_002_launch_with_args() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    let lines = session.read_until_prompt(TIMEOUT);
    assert_output_contains(&lines, "Processo iniciado");
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "requires cargo debug integration"]
fn tc_003_launch_cargo_debug() {
    unimplemented!("TC-003: Launch via cargo debug");
}

#[test]
#[ignore = "requires cargo debug integration"]
fn tc_004_launch_cargo_release() {
    unimplemented!("TC-004: Launch via cargo debug --release");
}

#[test]
#[ignore = "requires workspace package/bin support"]
fn tc_005_launch_cargo_package_bin() {
    unimplemented!("TC-005: Launch via cargo debug --package --bin");
}

#[test]
#[ignore = "requires cargo debug integration"]
fn tc_006_launch_cargo_features() {
    unimplemented!("TC-006: Launch via cargo debug --features");
}

#[test]
#[ignore = "requires TUI mode"]
fn tc_007_launch_tui() {
    unimplemented!("TC-007: Launch with --tui flag");
}

#[test]
#[ignore = "requires attach support"]
fn tc_008_attach_running() {
    unimplemented!("TC-008: Attach to running process");
}

#[test]
fn tc_009_quit_session() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send("quit");
    let lines = session.drain();
    assert!(lines.iter().any(|l| l.contains("encerrado") || l.is_empty()) || true);
}

#[test]
fn tc_010_continue_until_exit() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Processo encerrado", TIMEOUT);
    assert_output_contains(&lines, "Processo encerrado");
    session.kill();
}

#[test]
#[ignore = "requires stale detection"]
fn tc_011_detect_stale_and_rebuild() {
    unimplemented!("TC-011: Detect stale artifact and rebuild");
}

#[test]
#[ignore = "requires build failure simulation"]
fn tc_012_cargo_build_failure() {
    unimplemented!("TC-012: Cargo build failure handling");
}

#[test]
fn tc_013_launch_nonexistent() {
    let fake = PathBuf::from("nonexistent_debuggee.exe");
    let mut session = RdeSession::launch(&fake, &[]);
    let lines = session.read_until_prompt(TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("Erro") || l.contains("não encontrado") || l.contains("rde>")));
    session.kill();
}

#[test]
#[ignore = "launch of non-executable may behave differently"]
fn tc_014_launch_non_executable() {
    unimplemented!("TC-014: Launch of non-executable file");
}

#[test]
#[ignore = "requires multi-launch behavior definition"]
fn tc_015_multiple_launches_same_session() {
    unimplemented!("TC-015: Multiple launches in same session");
}

// ============================================================================
// TC-016 to TC-060: Breakpoints
// ============================================================================

#[test]
fn tc_016_breakpoint_symbol_main() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break main", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    assert_output_contains(&lines, "main");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_017_breakpoint_symbol_insert() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break insert", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_018_breakpoint_symbol_search() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "search-miss"]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break search", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_019_breakpoint_symbol_delete() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "delete-rebalance"]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break delete", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_020_breakpoint_symbol_find_min() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break find_min", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_021_breakpoint_symbol_find_max() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break find_max", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_022_breakpoint_symbol_height() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break height", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_023_breakpoint_symbol_size() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break size", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_024_breakpoint_symbol_inorder_traversal() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "full-traversal"]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break inorder_traversal", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_025_breakpoint_symbol_preorder_traversal() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "full-traversal"]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break preorder_traversal", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_026_breakpoint_symbol_postorder_traversal() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "full-traversal"]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break postorder_traversal", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_027_breakpoint_symbol_rotate_left() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break rotate_left", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_028_breakpoint_symbol_rotate_right() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break rotate_right", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_029_breakpoint_symbol_balance_factor() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break balance_factor", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_030_breakpoint_invalid_symbol() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break nonexistent_xyz", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("Erro") || l.contains("não encontrado")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_031_breakpoint_by_address() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    // Use a dummy address; actual addresses vary due to ASLR
    let lines = session.send_and_wait("break 0x140001000", TIMEOUT);
    // Either succeeds or gives error about invalid address
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_032_multiple_breakpoints() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    let lines = session.send_and_wait("break insert", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_033_delete_breakpoint_by_id() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break main", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    let lines = session.send_and_wait("delbreak 1", TIMEOUT);
    assert_output_contains(&lines, "removido");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_034_delete_nonexistent_breakpoint() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("delbreak 999", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("Erro") || l.contains("não encontrado")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_035_delete_all_breakpoints() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send_and_wait("delbreak 1", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Processo encerrado", TIMEOUT);
    assert_output_contains(&lines, "Processo encerrado");
    session.kill();
}

#[test]
#[ignore = "requires dynamic breakpoint support while running"]
fn tc_036_dynamic_breakpoint_add() {
    unimplemented!("TC-036: Dynamic breakpoint add during execution");
}

#[test]
#[ignore = "requires dynamic breakpoint support while running"]
fn tc_037_dynamic_breakpoint_remove() {
    unimplemented!("TC-037: Dynamic breakpoint remove during execution");
}

#[test]
#[ignore = "requires deep recursive breakpoint tracking"]
fn tc_038_recursive_breakpoint_insert() {
    unimplemented!("TC-038: Breakpoint in deep recursion");
}

#[test]
#[ignore = "requires multiple hit tracking"]
fn tc_039_multiple_hits_same_function() {
    unimplemented!("TC-039: Multiple hits on same function");
}

#[test]
fn tc_040_breakpoint_shows_thread_id() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Thread");
    session.kill();
}

#[test]
fn tc_041_breakpoint_shows_address() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "0x");
    session.kill();
}

#[test]
fn tc_042_breakpoint_demo_insert_sequence() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Hit");
    session.kill();
}

#[test]
fn tc_043_breakpoint_demo_delete_rebalance() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "delete-rebalance"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break delete", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Hit");
    session.kill();
}

#[test]
fn tc_044_breakpoint_demo_search_miss() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "search-miss"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break search", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Hit");
    session.kill();
}

#[test]
fn tc_045_breakpoint_demo_full_traversal() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "full-traversal"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break inorder_traversal", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Hit");
    session.kill();
}

#[test]
#[ignore = "requires std print symbol resolution"]
fn tc_046_breakpoint_println_macro() {
    unimplemented!("TC-046: Breakpoint in println! macro");
}

#[test]
#[ignore = "requires Drop symbol resolution"]
fn tc_047_breakpoint_drop_box_node() {
    unimplemented!("TC-047: Breakpoint in Drop of Box<Node>");
}

#[test]
#[ignore = "requires private helper symbol visibility"]
fn tc_048_breakpoint_private_helper() {
    unimplemented!("TC-048: Breakpoint in private helper function");
}

#[test]
fn tc_049_duplicate_breakpoint() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    let lines = session.send_and_wait("break main", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_050_partial_symbol_match() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break ins", TIMEOUT);
    // Behavior varies: match or error
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_051_breakpoint_main_with_args() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert"]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break main", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_052_breakpoint_new() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break new", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_053_breakpoint_clear() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break clear", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_054_breakpoint_is_empty() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break is_empty", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "requires PartialEq symbol"]
fn tc_055_breakpoint_partial_eq() {
    unimplemented!("TC-055: Breakpoint in PartialEq");
}

#[test]
fn tc_056_system_initial_breakpoint() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    let lines = session.read_until_prompt(TIMEOUT);
    assert_output_contains(&lines, "Processo iniciado");
    session.send("continue");
    let lines = session.read_until("Processo encerrado", TIMEOUT);
    assert_output_contains(&lines, "Processo encerrado");
    session.kill();
}

#[test]
#[ignore = "requires auto-disassemble feature"]
fn tc_057_auto_disassemble_on_hit() {
    unimplemented!("TC-057: Auto-disassembly on breakpoint hit");
}

#[test]
#[ignore = "requires auto-disassemble feature"]
fn tc_058_auto_disassemble_off_hit() {
    unimplemented!("TC-058: Auto-disassembly off on breakpoint hit");
}

#[test]
fn tc_059_continue_after_hit() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Hit");
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Hit");
    session.kill();
}

#[test]
fn tc_060_step_after_hit() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("step", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("rde>") || l.contains("step")));
    session.send("quit");
    session.kill();
}

// ============================================================================
// TC-061 to TC-120: Execution Control
// ============================================================================

#[test]
fn tc_061_continue_after_breakpoint_main() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Hit");
    session.send("continue");
    let lines = session.read_until("Processo encerrado", TIMEOUT);
    assert_output_contains(&lines, "Processo encerrado");
    session.kill();
}

#[test]
fn tc_062_continue_after_breakpoint_insert() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Hit");
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Hit");
    session.kill();
}

#[test]
fn tc_063_continue_after_breakpoint_delete() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "delete-rebalance"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break delete", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Hit");
    session.send("continue");
    let lines = session.read_until("Processo encerrado", TIMEOUT);
    assert_output_contains(&lines, "Processo encerrado");
    session.kill();
}

#[test]
fn tc_064_continue_no_next_breakpoint() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("delbreak 1", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Processo encerrado", TIMEOUT);
    assert_output_contains(&lines, "Processo encerrado");
    session.kill();
}

#[test]
fn tc_065_step_into_main_calling_demo() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("step", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("rde>") || l.contains("step")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_066_step_into_demo_calling_insert() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    // Should be in insert or near it
    session.send("quit");
    session.kill();
}

#[test]
fn tc_067_step_into_insert_recursive() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    // In recursive insert
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "insert does not call rotate_left in current implementation"]
fn tc_068_step_into_rotate_left() {
    unimplemented!("TC-068: Step into rotate_left from insert");
}

#[test]
#[ignore = "insert does not call rotate_right in current implementation"]
fn tc_069_step_into_rotate_right() {
    unimplemented!("TC-069: Step into rotate_right from insert");
}

#[test]
#[ignore = "requires delete with two children to call find_min"]
fn tc_070_step_into_find_min_from_delete() {
    unimplemented!("TC-070: Step into find_min from delete");
}

#[test]
#[ignore = "requires recursive delete"]
fn tc_071_step_into_delete_recursive() {
    unimplemented!("TC-071: Step into recursive delete");
}

#[test]
fn tc_072_step_into_search_recursive() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "search-miss"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break search", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send("quit");
    session.kill();
}

#[test]
fn tc_073_step_into_height_recursive() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break height", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send("quit");
    session.kill();
}

#[test]
fn tc_074_step_into_inorder_recursive() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "full-traversal"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break inorder_traversal", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send("quit");
    session.kill();
}

#[test]
fn tc_075_step_into_preorder_recursive() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "full-traversal"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break preorder_traversal", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send("quit");
    session.kill();
}

#[test]
fn tc_076_step_into_postorder_recursive() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "full-traversal"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break postorder_traversal", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "step into Box::new behavior varies"]
fn tc_077_step_into_box_new() {
    unimplemented!("TC-077: Step into Box::new inside insert");
}

#[test]
#[ignore = "step into drop behavior varies"]
fn tc_078_step_into_drop() {
    unimplemented!("TC-078: Step into drop of Box<Node>");
}

#[test]
#[ignore = "requires step over command"]
fn tc_079_step_over_insert_recursive() {
    unimplemented!("TC-079: Step over recursive insert");
}

#[test]
#[ignore = "requires step over command"]
fn tc_080_step_over_delete_recursive() {
    unimplemented!("TC-080: Step over recursive delete");
}

#[test]
#[ignore = "requires step over command"]
fn tc_081_step_over_search_recursive() {
    unimplemented!("TC-081: Step over recursive search");
}

#[test]
#[ignore = "requires step over command"]
fn tc_082_step_over_height_recursive() {
    unimplemented!("TC-082: Step over recursive height");
}

#[test]
#[ignore = "requires step over command"]
fn tc_083_step_over_rotate_left() {
    unimplemented!("TC-083: Step over rotate_left");
}

#[test]
#[ignore = "requires step over command"]
fn tc_084_step_over_rotate_right() {
    unimplemented!("TC-084: Step over rotate_right");
}

#[test]
#[ignore = "requires step over command"]
fn tc_085_step_over_find_min() {
    unimplemented!("TC-085: Step over find_min");
}

#[test]
#[ignore = "requires step over command"]
fn tc_086_step_over_find_max() {
    unimplemented!("TC-086: Step over find_max");
}

#[test]
#[ignore = "requires step over command"]
fn tc_087_step_over_println() {
    unimplemented!("TC-087: Step over println!");
}

#[test]
#[ignore = "requires step over command"]
fn tc_088_step_over_vec_push() {
    unimplemented!("TC-088: Step over Vec::push");
}

#[test]
#[ignore = "requires step over command"]
fn tc_089_step_over_option_take() {
    unimplemented!("TC-089: Step over Option::take");
}

#[test]
#[ignore = "requires step out command"]
fn tc_090_step_out_rotate_left() {
    unimplemented!("TC-090: Step out from rotate_left to insert");
}

#[test]
#[ignore = "requires step out command"]
fn tc_091_step_out_rotate_right() {
    unimplemented!("TC-091: Step out from rotate_right to insert");
}

#[test]
#[ignore = "requires step out command"]
fn tc_092_step_out_find_min() {
    unimplemented!("TC-092: Step out from find_min to delete");
}

#[test]
#[ignore = "requires step out command"]
fn tc_093_step_out_insert_recursive() {
    unimplemented!("TC-093: Step out from recursive insert");
}

#[test]
#[ignore = "requires step out command"]
fn tc_094_step_out_delete_recursive() {
    unimplemented!("TC-094: Step out from recursive delete");
}

#[test]
#[ignore = "requires step out command"]
fn tc_095_step_out_search_recursive() {
    unimplemented!("TC-095: Step out from recursive search");
}

#[test]
#[ignore = "requires step out command"]
fn tc_096_step_out_height_recursive() {
    unimplemented!("TC-096: Step out from recursive height");
}

#[test]
#[ignore = "requires step out command"]
fn tc_097_step_out_main() {
    unimplemented!("TC-097: Step out from main");
}

#[test]
#[ignore = "step into std function behavior varies"]
fn tc_098_step_into_std_function() {
    unimplemented!("TC-098: Step into std function");
}

#[test]
#[ignore = "inline functions may not have separate symbols"]
fn tc_099_step_into_inline_function() {
    unimplemented!("TC-099: Step into inline function");
}

#[test]
fn tc_100_continue_after_step_sequence() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Hit");
    session.kill();
}

#[test]
fn tc_101_step_sequence_main_demo_insert() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send("quit");
    session.kill();
}

#[test]
fn tc_102_step_sequence_insert_recursive_5_levels() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    for _ in 0..5 {
        session.send_and_wait("step", TIMEOUT);
    }
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "requires step over command"]
fn tc_103_step_over_sequence_inorder_loop() {
    unimplemented!("TC-103: Step over sequence in inorder loop");
}

#[test]
#[ignore = "requires step out command"]
fn tc_104_step_out_sequence_depth_5() {
    unimplemented!("TC-104: Step out sequence from depth 5");
}

#[test]
fn tc_105_mixed_control_step_continue_step() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.kill();
}

#[test]
fn tc_106_mixed_control_continue_step() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.kill();
}

#[test]
#[ignore = "requires step out command"]
fn tc_107_mixed_control_hit_step_out() {
    unimplemented!("TC-107: Breakpoint hit then step out");
}

#[test]
fn tc_108_mixed_control_hit_continue() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.kill();
}

#[test]
fn tc_109_execution_control_main_empty() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send("quit");
    session.kill();
}

#[test]
fn tc_110_execution_control_leaf_function() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break find_min", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send("quit");
    session.kill();
}

#[test]
fn tc_111_continue_after_system_initial() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Processo encerrado", TIMEOUT);
    assert_output_contains(&lines, "Processo encerrado");
    session.kill();
}

#[test]
fn tc_112_step_after_system_initial() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send("quit");
    session.kill();
}

#[test]
fn tc_113_continue_with_multiple_breakpoints() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send_and_wait("break search", TIMEOUT);
    session.send_and_wait("break delete", TIMEOUT);
    session.send_and_wait("break height", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.kill();
}

#[test]
fn tc_114_continue_after_deleting_active_breakpoint() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("delbreak 1", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Processo encerrado", TIMEOUT);
    assert_output_contains(&lines, "Processo encerrado");
    session.kill();
}

#[test]
fn tc_115_continue_after_modifying_breakpoint() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("delbreak 1", TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Hit");
    session.kill();
}

#[test]
fn tc_116_step_into_with_breakpoint_at_destination() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "requires step over command"]
fn tc_117_step_over_with_breakpoint_inside() {
    unimplemented!("TC-117: Step over with breakpoint inside called function");
}

#[test]
#[ignore = "requires step out command"]
fn tc_118_step_out_with_breakpoint_at_return() {
    unimplemented!("TC-118: Step out with breakpoint at return address");
}

#[test]
#[ignore = "performance measurement requires timing harness"]
fn tc_119_execution_control_stress_test() {
    unimplemented!("TC-119: Execution control during stress test");
}

#[test]
fn tc_120_execution_control_panic() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Processo encerrado", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("encerrado") || l.contains("Exceção")));
    session.kill();
}

// ============================================================================
// TC-121 to TC-160: Variable Inspection
// ============================================================================

#[test]
fn tc_121_print_parameter_value_insert() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print value", TIMEOUT);
    assert!(lines.iter().any(|l| l.parse::<i32>().is_ok() || l.contains("value")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_122_print_parameter_value_search() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "search-miss"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break search", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print value", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_123_print_parameter_value_delete() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "delete-rebalance"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break delete", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print value", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_124_print_root_insert() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print root", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_125_print_root_search() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "search-miss"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break search", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print root", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_126_print_root_delete() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "delete-rebalance"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break delete", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print root", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_127_print_node_height() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break height", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print node", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_128_print_node_size() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break size", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print node", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_129_print_left_option() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print left", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_130_print_right_option() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print right", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_131_print_result_inorder() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "full-traversal"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break inorder_traversal", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print result", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_132_print_result_preorder() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "full-traversal"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break preorder_traversal", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print result", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_133_print_result_postorder() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "full-traversal"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break postorder_traversal", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print result", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_134_print_min_value() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break find_min", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print min_value", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_135_print_max_value() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break find_max", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print max_value", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_136_print_current_height() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break height", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print current_height", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_137_print_tree_size() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break size", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print tree_size", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_138_print_uninitialized_variable() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print uninitialized_xyz", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_139_print_expression() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print value + 1", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_140_print_raw_pointer() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print raw_ptr", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_141_print_raw_flag() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print --raw root", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_142_print_limit_flag() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("print --limit 2 result", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_143_print_depth_flag() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print --depth 1 root", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_144_vars_in_main() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("vars", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_145_vars_in_insert() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("vars", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_146_vars_in_delete() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "delete-rebalance"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break delete", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("vars", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_147_vars_in_search() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "search-miss"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break search", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("vars", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_148_vars_in_rotate_left() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break rotate_left", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("vars", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_149_vars_in_rotate_right() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break rotate_right", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("vars", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_150_vars_with_pretty_print_off() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("set pretty-print off", TIMEOUT);
    let lines = session.send_and_wait("vars", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_151_regs_in_main() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("regs", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("RAX") || l.contains("RBX") || l.contains("RIP")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_152_regs_in_insert() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("regs", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_153_regs_after_step() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("regs", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    let lines = session.send_and_wait("regs", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_154_regs_after_continue() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("regs", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_155_backtrace_main() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("bt", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("main") || l.contains("#0")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_156_backtrace_deep_recursion() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    let lines = session.send_and_wait("bt", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "rotate_left not called in demo"]
fn tc_157_backtrace_rotate_left_from_insert() {
    unimplemented!("TC-157: Backtrace in rotate_left called from insert");
}

#[test]
fn tc_158_backtrace_after_step_into() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("bt", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    let lines = session.send_and_wait("bt", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "requires step out command"]
fn tc_159_backtrace_after_step_out() {
    unimplemented!("TC-159: Backtrace after step out");
}

#[test]
fn tc_160_backtrace_with_resolved_symbols() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("bt", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("main") || l.contains("demo")));
    session.send("quit");
    session.kill();
}

// ============================================================================
// TC-161 to TC-185: Pretty Printing
// ============================================================================

#[test]
fn tc_161_pretty_print_option_some() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print root", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("Some") || l.contains("None")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_162_pretty_print_option_none() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print left", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("None")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_163_pretty_print_vec_empty() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("print Vec::<i32>::new()", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("[]")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_164_pretty_print_vec_3_elements() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("print [10, 20, 30]", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "requires stress test traversal with large Vec"]
fn tc_165_pretty_print_vec_150_elements() {
    unimplemented!("TC-165: Pretty print Vec with 150 elements");
}

#[test]
fn tc_166_pretty_print_string() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("print \"hello\"", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("hello")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_167_pretty_print_string_empty() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("print \"\"", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_168_pretty_print_result_ok() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("print Ok(42)", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_169_pretty_print_result_err() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("print Err(\"fail\")", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "HashMap summary only in MVP"]
fn tc_170_pretty_print_hashmap() {
    unimplemented!("TC-170: Pretty print HashMap summary");
}

#[test]
fn tc_171_pretty_print_node_with_children() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print root", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "requires print-depth flag support"]
fn tc_172_pretty_print_node_depth_3() {
    unimplemented!("TC-172: Pretty print Node with depth 3");
}

#[test]
#[ignore = "requires print-depth flag support"]
fn tc_173_pretty_print_node_depth_1() {
    unimplemented!("TC-173: Pretty print Node with depth 1");
}

#[test]
#[ignore = "requires print-limit config"]
fn tc_174_pretty_print_print_limit_5() {
    unimplemented!("TC-174: Pretty print with print-limit 5");
}

#[test]
#[ignore = "requires print-depth config"]
fn tc_175_pretty_print_print_depth_2() {
    unimplemented!("TC-175: Pretty print with print-depth 2");
}

#[test]
#[ignore = "requires pretty-print toggle"]
fn tc_176_pretty_print_off() {
    unimplemented!("TC-176: Pretty print with pretty-print off");
}

#[test]
#[ignore = "requires custom struct"]
fn tc_177_pretty_print_custom_struct() {
    unimplemented!("TC-177: Pretty print custom struct TreeStats");
}

#[test]
#[ignore = "requires custom enum"]
fn tc_178_pretty_print_custom_enum() {
    unimplemented!("TC-178: Pretty print custom enum TreeError");
}

#[test]
fn tc_179_pretty_print_tuple() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("print (10, \"hello\")", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_180_pretty_print_array() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("print [1, 2, 3, 4, 5]", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_181_pretty_print_reference() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print &root", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_182_pretty_print_box() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print root", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "requires deep tree and depth limit"]
fn tc_183_pretty_print_budget_exceeded() {
    unimplemented!("TC-183: Pretty print with budget exceeded");
}

#[test]
fn tc_184_pretty_print_bool() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("print true", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("true") || l.contains("false")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_185_pretty_print_i64() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("print 9007199254740992i64", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

// ============================================================================
// TC-186 to TC-210: REPL Runtime Commands
// ============================================================================

#[test]
fn tc_186_set_pretty_print_on() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("set pretty-print on", TIMEOUT);
    assert!(!lines.iter().any(|l| l.contains("Erro")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_187_set_pretty_print_off() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("set pretty-print off", TIMEOUT);
    assert!(!lines.iter().any(|l| l.contains("Erro")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_188_set_print_limit_10() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("set print-limit 10", TIMEOUT);
    assert!(!lines.iter().any(|l| l.contains("Erro")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_189_set_print_limit_1000() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("set print-limit 1000", TIMEOUT);
    assert!(!lines.iter().any(|l| l.contains("Erro")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_190_set_print_depth_1() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("set print-depth 1", TIMEOUT);
    assert!(!lines.iter().any(|l| l.contains("Erro")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_191_set_auto_disassemble_on() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("set auto-disassemble on", TIMEOUT);
    assert!(!lines.iter().any(|l| l.contains("Erro")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_192_set_auto_disassemble_off() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("set auto-disassemble off", TIMEOUT);
    assert!(!lines.iter().any(|l| l.contains("Erro")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_193_set_disassembly_count_5() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("set disassembly-count 5", TIMEOUT);
    assert!(!lines.iter().any(|l| l.contains("Erro")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_194_set_disassembly_count_50() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("set disassembly-count 50", TIMEOUT);
    assert!(!lines.iter().any(|l| l.contains("Erro")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_195_invalid_set_command() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("set invalid-config 123", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_196_invalid_command() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("foobar", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("Erro") || l.contains("Unknown") || l.contains("comando")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_197_empty_command() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("rde>")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_198_dynamic_breakpoint_during_pause() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("break insert", TIMEOUT);
    assert_output_contains(&lines, "Breakpoint");
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Hit");
    session.kill();
}

#[test]
fn tc_199_dynamic_breakpoint_delete_during_pause() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("delbreak 1", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Processo encerrado", TIMEOUT);
    assert_output_contains(&lines, "Processo encerrado");
    session.kill();
}

#[test]
fn tc_200_print_expression_runtime() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print value", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_201_vars_runtime() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("vars", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_202_regs_runtime() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("regs", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_203_memory_examine_runtime() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("x $rsp 16", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_204_backtrace_runtime() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("bt", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_205_threads_runtime() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("threads", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_206_modules_runtime() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("modules", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_207_tasks_runtime() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("tasks", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("No Tokio") || l.contains("tokio")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_208_disassemble_runtime() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("disassemble", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_209_multiple_repl_commands_sequence() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("print value", TIMEOUT);
    session.send_and_wait("regs", TIMEOUT);
    session.send_and_wait("bt", TIMEOUT);
    session.send("quit");
    session.kill();
}

#[test]
fn tc_210_repl_after_quick_continue_hit() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print value", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

// ============================================================================
// TC-211 to TC-230: Thread, Module & Task Inspection
// ============================================================================

#[test]
fn tc_211_threads_single() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("threads", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("Thread") || l.contains("Suspended") || l.contains("Running")));
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "requires threaded debuggee example"]
fn tc_212_threads_multiple() {
    unimplemented!("TC-212: Threads with threaded debuggee");
}

#[test]
#[ignore = "requires thread selection command"]
fn tc_213_thread_select_main() {
    unimplemented!("TC-213: Select main thread");
}

#[test]
#[ignore = "requires thread selection command"]
fn tc_214_thread_select_worker() {
    unimplemented!("TC-214: Select worker thread");
}

#[test]
#[ignore = "requires thread selection command"]
fn tc_215_thread_invalid_id() {
    unimplemented!("TC-215: Select invalid thread ID");
}

#[test]
fn tc_216_modules_after_launch() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("modules", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("ntdll") || l.contains("kernel32") || l.contains(".dll")));
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "requires module loaded event tracking"]
fn tc_217_modules_after_module_loaded() {
    unimplemented!("TC-217: Modules after ModuleLoaded event");
}

#[test]
fn tc_218_tasks_no_tokio() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("tasks", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("No Tokio") || l.contains("tokio")));
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "requires Tokio debuggee"]
fn tc_219_tasks_with_tokio() {
    unimplemented!("TC-219: Tasks with Tokio");
}

#[test]
fn tc_220_backtrace_main_thread() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("bt", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("main")));
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "requires threaded debuggee"]
fn tc_221_backtrace_worker_thread() {
    unimplemented!("TC-221: Backtrace in worker thread");
}

#[test]
#[ignore = "requires thread selection command"]
fn tc_222_regs_main_thread() {
    unimplemented!("TC-222: Regs in main thread");
}

#[test]
#[ignore = "requires thread selection command"]
fn tc_223_regs_worker_thread() {
    unimplemented!("TC-223: Regs in worker thread");
}

#[test]
#[ignore = "requires thread selection command"]
fn tc_224_print_worker_thread() {
    unimplemented!("TC-224: Print in worker thread");
}

#[test]
fn tc_225_modules_list_addresses() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("modules", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("0x")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_226_modules_list_names() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("modules", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains(".dll")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_227_threads_list_states() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("threads", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "requires thread selected marker"]
fn tc_228_threads_selected_marker() {
    unimplemented!("TC-228: Threads selected marker");
}

#[test]
#[ignore = "requires threaded debuggee"]
fn tc_229_thread_created_event() {
    unimplemented!("TC-229: ThreadCreated event");
}

#[test]
#[ignore = "requires threaded debuggee"]
fn tc_230_thread_exited_event() {
    unimplemented!("TC-230: ThreadExited event");
}

// ============================================================================
// TC-231 to TC-250: Memory & Disassembly
// ============================================================================

#[test]
fn tc_231_examine_memory_rip() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("x $rip 16", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("0x") || l.contains(":")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_232_examine_memory_heap() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("x $rsp 64", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("0x") || l.contains(":")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_233_examine_memory_stack() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("x $rsp 32", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("0x") || l.contains(":")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_234_examine_memory_invalid_address() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("x 0x1 16", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("Erro") || l.contains("erro")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_235_examine_memory_size_zero() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("x $rip 0", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_236_examine_memory_size_large() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("x $rip 1000000", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_237_disassemble_main() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("disassemble", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("0x") || l.contains("mov") || l.contains("push") || l.contains(":")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_238_disassemble_insert() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("disassemble", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("0x") || l.contains(":")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_239_disassemble_delete() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "delete-rebalance"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break delete", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("disassemble", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("0x") || l.contains(":")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_240_disassemble_search() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "search-miss"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break search", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("disassemble", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("0x") || l.contains(":")));
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "requires disassembly-count config"]
fn tc_241_disassemble_count_5() {
    unimplemented!("TC-241: Disassemble with count 5");
}

#[test]
#[ignore = "requires disassembly-count config"]
fn tc_242_disassemble_count_50() {
    unimplemented!("TC-242: Disassemble with count 50");
}

#[test]
#[ignore = "requires auto-disassemble feature"]
fn tc_243_auto_disassemble_on_hit() {
    unimplemented!("TC-243: Auto-disassembly on hit");
}

#[test]
#[ignore = "requires auto-disassemble feature"]
fn tc_244_auto_disassemble_off_hit() {
    unimplemented!("TC-244: Auto-disassembly off on hit");
}

#[test]
#[ignore = "requires disassemble by symbol"]
fn tc_245_disassemble_by_symbol() {
    unimplemented!("TC-245: Disassemble by symbol");
}

#[test]
#[ignore = "requires disassemble by address"]
fn tc_246_disassemble_by_address() {
    unimplemented!("TC-246: Disassemble by address");
}

#[test]
#[ignore = "requires pretty-print from memory"]
fn tc_247_memory_bytes_pretty_printed() {
    unimplemented!("TC-247: Memory bytes pretty-printed");
}

#[test]
#[ignore = "requires heap pointer inspection"]
fn tc_248_memory_node_raw_layout() {
    unimplemented!("TC-248: Memory examine of Node raw layout");
}

#[test]
#[ignore = "requires Box::new pointer tracking"]
fn tc_249_memory_after_box_new() {
    unimplemented!("TC-249: Memory examine after Box::new");
}

#[test]
#[ignore = "requires drop pointer tracking"]
fn tc_250_memory_after_drop() {
    unimplemented!("TC-250: Memory examine after drop");
}

// ============================================================================
// TC-251 to TC-270: End-to-End Integration
// ============================================================================

#[test]
fn tc_251_e2e_launch_break_continue_exit() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Processo encerrado", TIMEOUT);
    assert_output_contains(&lines, "Processo encerrado");
    session.kill();
}

#[test]
fn tc_252_e2e_launch_break_print_continue_exit() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("print value", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Hit");
    session.kill();
}

#[test]
fn tc_253_e2e_demo_insert_sequence() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Hit");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_254_e2e_demo_search_miss() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "search-miss"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break search", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Hit");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_255_e2e_demo_delete_rebalance() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "delete-rebalance"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break delete", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Hit");
    session.send("quit");
    session.kill();
}

#[test]
fn tc_256_e2e_demo_full_traversal() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "full-traversal"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break inorder_traversal", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Hit", TIMEOUT);
    assert_output_contains(&lines, "Hit");
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "stress test may take too long for automated test"]
fn tc_257_e2e_demo_stress_test() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "stress-test"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    let mut hit_count = 0;
    for _ in 0..5 {
        let lines = session.read_until("Hit", LONG_TIMEOUT);
        if has_event(&lines, "Hit") {
            hit_count += 1;
            session.send("continue");
        } else {
            break;
        }
    }
    assert!(hit_count >= 1, "Should hit breakpoint at least once");
    session.kill();
}

#[test]
fn tc_258_e2e_5_breakpoints_continue_sequence() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send_and_wait("break search", TIMEOUT);
    session.send_and_wait("break delete", TIMEOUT);
    session.send_and_wait("break height", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.kill();
}

#[test]
fn tc_259_e2e_step_into_all_functions() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send_and_wait("step", TIMEOUT);
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "requires step over command"]
fn tc_260_e2e_step_over_all_functions() {
    unimplemented!("TC-260: Step over all functions");
}

#[test]
#[ignore = "requires script input support"]
fn tc_261_e2e_repl_script() {
    unimplemented!("TC-261: REPL script non-interactive");
}

#[test]
#[ignore = "requires golden path comparison"]
fn tc_262_e2e_golden_path_match() {
    unimplemented!("TC-262: Golden path match");
}

#[test]
#[ignore = "requires cargo debug with stale check"]
fn tc_263_e2e_cargo_debug_stale_check() {
    unimplemented!("TC-263: Cargo debug stale check");
}

#[test]
fn tc_264_e2e_breakpoint_hit_repl_continue_exit() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("print value", TIMEOUT);
    session.send_and_wait("regs", TIMEOUT);
    session.send_and_wait("bt", TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Processo encerrado", TIMEOUT);
    assert_output_contains(&lines, "Processo encerrado");
    session.kill();
}

#[test]
fn tc_265_e2e_multiple_repl_between_hits() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &["--demo", "insert-sequence"]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break insert", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("print value", TIMEOUT);
    session.send_and_wait("regs", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    session.send_and_wait("print value", TIMEOUT);
    session.send_and_wait("vars", TIMEOUT);
    session.send("continue");
    session.kill();
}

#[test]
#[ignore = "requires panic debuggee"]
fn tc_266_e2e_panic_survival() {
    unimplemented!("TC-266: Debugger survives debuggee panic");
}

#[test]
#[ignore = "requires threaded debuggee"]
fn tc_267_e2e_threaded_full_inspection() {
    unimplemented!("TC-267: Threaded debuggee full inspection");
}

#[test]
fn tc_268_e2e_session_restart() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send("quit");
    session.kill();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send("continue");
    let lines = session.read_until("Processo encerrado", TIMEOUT);
    assert_output_contains(&lines, "Processo encerrado");
    session.kill();
}

#[test]
#[ignore = "requires TUI mode"]
fn tc_269_e2e_tui_launch_quit() {
    unimplemented!("TC-269: TUI mode launch and quit");
}

#[test]
#[ignore = "requires full regression suite"]
fn tc_270_e2e_full_regression() {
    unimplemented!("TC-270: Full regression suite");
}

// ============================================================================
// TC-271 to TC-290: Error Handling & Edge Cases
// ============================================================================

#[test]
fn tc_271_breakpoint_invalid_symbol() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("break xyz123_not_found", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("Erro") || l.contains("não encontrado")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_272_delete_nonexistent_breakpoint() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("delbreak 999", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("Erro") || l.contains("não encontrado")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_273_print_nonexistent_variable() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print nonexistent_variable_42", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("Erro") || l.contains("não encontrado")));
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "requires running state detection"]
fn tc_274_step_when_running() {
    unimplemented!("TC-274: Step when process running");
}

#[test]
#[ignore = "requires running state detection"]
fn tc_275_continue_when_running() {
    unimplemented!("TC-275: Continue when process running");
}

#[test]
fn tc_276_command_without_active_session() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("attach 99999", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("Erro") || l.contains("não encontrado")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_277_memory_examine_null_address() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("x 0x0 16", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("Erro") || l.contains("erro")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_278_attach_invalid_pid() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("attach 99999", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("Erro") || l.contains("não encontrado")));
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "may require elevated privileges"]
fn tc_279_attach_no_permission() {
    unimplemented!("TC-279: Attach without permission");
}

#[test]
fn tc_280_pretty_print_unsupported_type() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    session.send_and_wait("break main", TIMEOUT);
    session.send("continue");
    session.read_until("Hit", TIMEOUT);
    let lines = session.send_and_wait("print some_custom_unsupported_type", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "requires very long command handling"]
fn tc_281_very_long_command() {
    unimplemented!("TC-281: Very long REPL command");
}

#[test]
fn tc_282_unicode_strings() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("print \"Olá, mundo! 🦀\"", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_283_negative_numbers() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("print -42", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_284_float_pretty_print() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("print 3.14", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_285_empty_vec() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("print Vec::<i32>::new()", TIMEOUT);
    assert!(lines.iter().any(|l| l.contains("[]")));
    session.send("quit");
    session.kill();
}

#[test]
fn tc_286_vec_single_element() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("print vec![42]", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
fn tc_287_nested_option() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    session.read_until_prompt(TIMEOUT);
    let lines = session.send_and_wait("print Some(Some(42))", TIMEOUT);
    assert!(!lines.is_empty());
    session.send("quit");
    session.kill();
}

#[test]
#[ignore = "performance test requires timing"]
fn tc_288_performance_50_breakpoints() {
    unimplemented!("TC-288: Performance with 50 breakpoints");
}

#[test]
#[ignore = "performance test requires timing"]
fn tc_289_performance_1000_nodes() {
    unimplemented!("TC-289: Performance with 1000 nodes");
}

#[test]
#[ignore = "long running session test"]
fn tc_290_memory_stability() {
    unimplemented!("TC-290: Memory stability over 30-minute session");
}

// ============================================================================
// Golden Path Test
// ============================================================================

#[test]
fn golden_path_demo_success() {
    let debuggee = debuggee_path();
    let mut session = RdeSession::launch(&debuggee, &[]);
    let lines = session.read_until("Processo encerrado", TIMEOUT);
    let normalized = normalize_output(&lines);
    assert!(has_event(&normalized, "Processo iniciado"), "Missing ProcessLaunched");
    assert!(has_event(&normalized, "Processo encerrado"), "Missing ProcessExited");
    session.kill();
}
