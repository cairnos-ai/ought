use std::io::Write;
use std::sync::Mutex;

use oharness_core::event::{
    EventKind, LlmFailedPayload, LlmResponsePayload, ToolCallFailedPayload,
    ToolCallFinishedPayload, ToolCallStartedPayload,
};
use oharness_core::{Content, Event, EventSink};
use oharness_llm::{BlockStartKind, Chunk};

static PRINT_LOCK: Mutex<()> = Mutex::new(());

pub(crate) struct TerminalEventSink {
    assignment_id: String,
    state: Mutex<TerminalState>,
}

#[derive(Default)]
struct TerminalState {
    saw_stream_in_call: bool,
    line_open: bool,
}

impl TerminalEventSink {
    pub(crate) fn new(assignment_id: impl Into<String>) -> Self {
        Self {
            assignment_id: assignment_id.into(),
            state: Mutex::new(TerminalState::default()),
        }
    }

    fn handle(&self, event: Event) {
        match event.kind {
            EventKind::LlmRequest(_) => {
                let mut state = lock_state(&self.state);
                state.saw_stream_in_call = false;
                close_open_line(&self.assignment_id, &mut state);
            }
            EventKind::LlmStreamChunk(payload) => {
                let Ok(chunk) = serde_json::from_value::<Chunk>(payload) else {
                    return;
                };
                self.handle_chunk(chunk);
            }
            EventKind::LlmResponse(LlmResponsePayload { response }) => {
                let mut state = lock_state(&self.state);
                if !state.saw_stream_in_call {
                    render_response_content(&self.assignment_id, &mut state, &response.content);
                }
            }
            EventKind::LlmFailed(LlmFailedPayload { reason }) => {
                let mut state = lock_state(&self.state);
                close_open_line(&self.assignment_id, &mut state);
                with_stderr(|stderr| {
                    write_multiline_detail(stderr, &self.assignment_id, "llm error", &reason);
                });
            }
            EventKind::ToolCallStarted(ToolCallStartedPayload {
                tool_name,
                tool_use_id,
                ..
            }) => {
                let mut state = lock_state(&self.state);
                close_open_line(&self.assignment_id, &mut state);
                with_stderr(|stderr| {
                    let _ = writeln!(
                        stderr,
                        "  [agent {}] tool call: {tool_name} ({tool_use_id})",
                        self.assignment_id
                    );
                });
            }
            EventKind::ToolCallFinished(ToolCallFinishedPayload {
                tool_name,
                tool_use_id,
                ..
            }) => {
                with_stderr(|stderr| {
                    let _ = writeln!(
                        stderr,
                        "  [agent {}] tool done: {tool_name} ({tool_use_id})",
                        self.assignment_id
                    );
                });
            }
            EventKind::ToolCallFailed(ToolCallFailedPayload {
                tool_name,
                tool_use_id,
                reason,
                ..
            }) => {
                with_stderr(|stderr| {
                    write_multiline_detail(
                        stderr,
                        &self.assignment_id,
                        &format!("tool failed: {tool_name} ({tool_use_id})"),
                        &reason,
                    );
                });
            }
            _ => {}
        }
    }

    fn handle_chunk(&self, chunk: Chunk) {
        let mut state = lock_state(&self.state);
        state.saw_stream_in_call = true;
        match chunk {
            Chunk::BlockStart {
                start: BlockStartKind::Text,
                ..
            } => start_line(&self.assignment_id, &mut state, "message"),
            Chunk::BlockStart {
                start: BlockStartKind::Thinking,
                ..
            } => start_line(&self.assignment_id, &mut state, "thinking"),
            Chunk::BlockStart {
                start: BlockStartKind::ToolUse { name, id },
                ..
            } => {
                close_open_line(&self.assignment_id, &mut state);
                with_stderr(|stderr| {
                    let _ = writeln!(
                        stderr,
                        "  [agent {}] model requested tool: {name} ({id})",
                        self.assignment_id
                    );
                });
            }
            Chunk::TextDelta { text, .. } | Chunk::ThinkingDelta { text, .. } => {
                write_delta(&text);
            }
            Chunk::BlockStop { .. } | Chunk::MessageStop => {
                close_open_line(&self.assignment_id, &mut state);
            }
            Chunk::MessageStart { .. }
            | Chunk::ToolUseDelta { .. }
            | Chunk::StopReason { .. }
            | Chunk::Usage { .. }
            | Chunk::Raw { .. } => {}
        }
    }
}

impl EventSink for TerminalEventSink {
    fn emit(&self, event: Event) {
        self.handle(event);
    }

    fn try_emit(&self, event: Event) -> Result<(), Event> {
        self.handle(event);
        Ok(())
    }
}

fn render_response_content(assignment_id: &str, state: &mut TerminalState, content: &[Content]) {
    for block in content {
        match block {
            Content::Text { text } if !text.is_empty() => {
                start_line(assignment_id, state, "message");
                write_delta(text);
                close_open_line(assignment_id, state);
            }
            Content::Thinking { thinking } if !thinking.is_empty() => {
                start_line(assignment_id, state, "thinking");
                write_delta(thinking);
                close_open_line(assignment_id, state);
            }
            Content::ToolUse { name, id, .. } => {
                close_open_line(assignment_id, state);
                with_stderr(|stderr| {
                    let _ = writeln!(
                        stderr,
                        "  [agent {assignment_id}] model requested tool: {name} ({id})"
                    );
                });
            }
            _ => {}
        }
    }
}

fn start_line(assignment_id: &str, state: &mut TerminalState, label: &str) {
    close_open_line(assignment_id, state);
    with_stderr(|stderr| {
        let _ = write!(stderr, "  [agent {assignment_id}] {label}: ");
        let _ = stderr.flush();
    });
    state.line_open = true;
}

fn write_delta(text: &str) {
    with_stderr(|stderr| {
        let _ = write!(stderr, "{text}");
        let _ = stderr.flush();
    });
}

fn close_open_line(_assignment_id: &str, state: &mut TerminalState) {
    if state.line_open {
        with_stderr(|stderr| {
            let _ = writeln!(stderr);
        });
        state.line_open = false;
    }
}

fn write_multiline_detail(
    stderr: &mut std::io::Stderr,
    assignment_id: &str,
    label: &str,
    detail: &str,
) {
    let mut lines = detail.lines();
    if let Some(first) = lines.next() {
        let _ = writeln!(stderr, "  [agent {assignment_id}] {label}: {first}");
    } else {
        let _ = writeln!(stderr, "  [agent {assignment_id}] {label}");
        return;
    }
    for line in lines {
        let _ = writeln!(stderr, "    {line}");
    }
}

fn with_stderr(f: impl FnOnce(&mut std::io::Stderr)) {
    let _guard = PRINT_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut stderr = std::io::stderr();
    f(&mut stderr);
}

fn lock_state<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}
