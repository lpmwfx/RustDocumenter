//! `rustdocumenter diag` — verify AI backend availability.

const TEST_PROMPT: &str = "Reply with only the word: OK";

/// Run diagnostics and return output lines + exit code.
pub fn run() -> (i32, Vec<String>) {
    let mut lines = Vec::new();
    lines.push("rustdocumenter diag".to_string());
    lines.push(String::new());

    match rustdocumenter::gateway::run_claude(TEST_PROMPT) {
        Ok(resp) => {
            let trimmed = resp.trim().to_string();
            lines.push(format!("  claude: OK  (response: {trimmed:?})"));
            return (0, lines);
        }
        Err(e) => {
            lines.push(format!("  claude: FAIL  ({e})"));
        }
    }

    match rustdocumenter::gateway::run_codex(TEST_PROMPT) {
        Ok(resp) => {
            let trimmed = resp.trim().to_string();
            lines.push(format!("  codex:  OK  (response: {trimmed:?})"));
            return (0, lines);
        }
        Err(e) => {
            lines.push(format!("  codex:  FAIL  ({e})"));
        }
    }

    lines.push(String::new());
    lines.push("ERROR: no AI backend available — rustdocumenter doc will produce 0 items".to_string());
    lines.push("  Install Claude Code CLI: https://claude.ai/download".to_string());
    (1, lines)
}
