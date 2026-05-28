//! Async channel wrappers for engine communication.

use tokio::sync::mpsc;

use crate::{EngineCommand, EngineEvent};

/// Create a pair of channels for engine communication.
///
/// Returns `(command_tx, event_rx)` for the consumer (REPL/UI)
/// and `(command_rx, event_tx)` for the engine.
pub fn engine_channels() -> (
    mpsc::UnboundedSender<EngineCommand>,
    mpsc::UnboundedReceiver<EngineEvent>,
    mpsc::UnboundedReceiver<EngineCommand>,
    mpsc::UnboundedSender<EngineEvent>,
) {
    let (command_tx, command_rx) = mpsc::unbounded_channel();
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    (command_tx, event_rx, command_rx, event_tx)
}
