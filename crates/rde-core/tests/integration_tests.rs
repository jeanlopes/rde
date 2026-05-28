//! Integration tests for RDE core.

use rde_core::{EngineCommand, EngineEvent, ThreadState};

#[test]
fn test_select_thread_command() {
    let cmd = EngineCommand::SelectThread { id: 1234 };
    assert!(matches!(cmd, EngineCommand::SelectThread { id: 1234 }));
}

#[test]
fn test_thread_created_event_has_handle() {
    let evt = EngineEvent::ThreadCreated {
        id: 1234,
        handle: 0xABCD,
    };
    assert!(matches!(evt, EngineEvent::ThreadCreated { id: 1234, handle: 0xABCD }));
}

#[test]
fn test_module_unloaded_event() {
    let evt = EngineEvent::ModuleUnloaded { base: 0x7FF612340000 };
    assert!(matches!(evt, EngineEvent::ModuleUnloaded { base: 0x7FF612340000 }));
}

#[test]
fn test_thread_state_exited() {
    let state = ThreadState::Exited { exit_code: 0 };
    assert!(matches!(state, ThreadState::Exited { exit_code: 0 }));
}

#[test]
fn test_module_loaded_event() {
    let evt = EngineEvent::ModuleLoaded {
        name: "kernel32.dll".into(),
        base: 0x7FFEEABC0000,
    };
    assert!(matches!(evt, EngineEvent::ModuleLoaded { .. }));
}
