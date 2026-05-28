//! RDE TUI — Terminal User Interface with multi-pane layout.

pub mod app;
pub mod layout;
pub mod panes;
pub mod session_mirror;
pub mod widgets;

pub use app::run;
