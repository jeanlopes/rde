//! Test harness for RDE CLI integration tests.
//!
//! Provides helpers to spawn rde-cli, send REPL commands, and capture/normalize output.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use std::sync::mpsc;
use std::thread;

/// Locate the rde-cli executable (debug or release).
pub fn rde_cli_path() -> PathBuf {
    let candidates = [
        PathBuf::from("target/release/rde-cli.exe"),
        PathBuf::from("target/debug/rde-cli.exe"),
    ];
    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }
    panic!("rde-cli executable not found. Run `cargo build` or `cargo build --release` first.");
}

/// Locate the rust_app_example executable.
pub fn debuggee_path() -> PathBuf {
    let candidates = [
        PathBuf::from("target/release/rust_app_example.exe"),
        PathBuf::from("target/debug/rust_app_example.exe"),
    ];
    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }
    panic!("rust_app_example executable not found. Run `cargo build -p rust_app_example` first.");
}

/// A running RDE CLI session.
pub struct RdeSession {
    child: Child,
    reader: mpsc::Receiver<String>,
    _reader_thread: thread::JoinHandle<()>,
}

impl RdeSession {
    /// Launch rde-cli with the given debuggee and arguments.
    pub fn launch(debuggee: &Path, args: &[&str]) -> Self {
        let cli = rde_cli_path();
        let mut cmd = Command::new(&cli);
        cmd.arg(debuggee);
        for a in args {
            cmd.arg(a);
        }
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().expect("Failed to spawn rde-cli");
        let stdout = child.stdout.take().expect("No stdout");

        let (tx, rx) = mpsc::channel();
        let reader_thread = thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    let _ = tx.send(line);
                }
            }
        });

        RdeSession {
            child,
            reader: rx,
            _reader_thread: reader_thread,
        }
    }

    /// Read lines until the REPL prompt appears or timeout.
    pub fn read_until_prompt(&self, timeout: Duration) -> Vec<String> {
        let mut lines = Vec::new();
        let deadline = std::time::Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            match self.reader.recv_timeout(remaining) {
                Ok(line) => {
                    let is_prompt = line.trim() == "rde>";
                    lines.push(line);
                    if is_prompt {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        lines
    }

    /// Read lines until a specific pattern is found or timeout.
    pub fn read_until(&self, pattern: &str, timeout: Duration) -> Vec<String> {
        let mut lines = Vec::new();
        let deadline = std::time::Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                break;
            }
            match self.reader.recv_timeout(remaining) {
                Ok(line) => {
                    let found = line.contains(pattern);
                    lines.push(line);
                    if found {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        lines
    }

    /// Send a command to the REPL.
    pub fn send(&mut self, command: &str) {
        let stdin = self.child.stdin.as_mut().expect("No stdin");
        writeln!(stdin, "{}", command).expect("Failed to write to stdin");
    }

    /// Send a command and wait for the prompt to return.
    pub fn send_and_wait(&mut self, command: &str, timeout: Duration) -> Vec<String> {
        self.send(command);
        self.read_until_prompt(timeout)
    }

    /// Kill the session.
    pub fn kill(&mut self) {
        let _ = self.child.kill();
    }

    /// Drain all remaining output.
    pub fn drain(&self) -> Vec<String> {
        let mut lines = Vec::new();
        while let Ok(line) = self.reader.recv_timeout(Duration::from_millis(100)) {
            lines.push(line);
        }
        lines
    }
}

/// Normalize dynamic values in RDE output for golden path comparison.
pub fn normalize_output(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .map(|line| {
            let mut line = line.clone();
            // Replace "PID <number>"
            if let Some(pos) = line.find("PID ") {
                let start = pos + 4;
                let end = line[start..].find(|c: char| !c.is_ascii_digit()).map(|i| start + i).unwrap_or(line.len());
                line.replace_range(start..end, "<PID>");
            }
            // Replace "Thread <number>"
            if let Some(pos) = line.find("Thread ") {
                let start = pos + 7;
                let end = line[start..].find(|c: char| !c.is_ascii_digit()).map(|i| start + i).unwrap_or(line.len());
                line.replace_range(start..end, "<TID>");
            }
            // Replace hex addresses like 0x7FF812340000
            let mut result = String::new();
            let bytes = line.as_bytes();
            let mut i = 0;
            while i < bytes.len() {
                if i + 2 <= bytes.len() && bytes[i] == b'0' && (bytes[i + 1] == b'x' || bytes[i + 1] == b'X') {
                    result.push_str("0x<ADDR>");
                    i += 2;
                    while i < bytes.len() && bytes[i].is_ascii_hexdigit() {
                        i += 1;
                    }
                } else {
                    result.push(bytes[i] as char);
                    i += 1;
                }
            }
            result
        })
        .collect()
}

/// Check if output contains an event pattern.
pub fn has_event(lines: &[String], event: &str) -> bool {
    lines.iter().any(|l| l.contains(event))
}

/// Assert that a command produces expected output.
pub fn assert_output_contains(lines: &[String], expected: &str) {
    assert!(
        has_event(lines, expected),
        "Expected output to contain '{}', but got:\n{}",
        expected,
        lines.join("\n")
    );
}
