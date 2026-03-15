//! AI CLI subprocess runners.

use std::process::Command;

/// Call Claude Code CLI in print mode (non-interactive, text-only).
pub fn run_claude(prompt: &str) -> Result<String, String> {
    use std::io::Write;
    let claude_cwd = std::env::temp_dir();
    let mut child = if cfg!(windows) {
        Command::new("cmd")
            .args(["/C", "claude", "-p", "--model", "haiku", "--no-session-persistence"])
            .current_dir(&claude_cwd)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
    } else {
        Command::new("claude")
            .args(["-p", "--model", "haiku", "--no-session-persistence"])
            .current_dir(&claude_cwd)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
    }
    .map_err(|e| format!("spawn error: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(prompt.as_bytes());
    }

    let output = child.wait_with_output().map_err(|e| format!("wait error: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("exit {}: {stderr}", output.status));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if stdout.trim().is_empty() {
        return Err("empty response".to_string());
    }

    Ok(stdout)
}

/// Call Codex CLI in non-interactive exec mode.
pub fn run_codex(prompt: &str) -> Result<String, String> {
    let output = if cfg!(windows) {
        Command::new("cmd")
            .args(["/C", "codex", "exec", "--ephemeral", prompt])
            .output()
    } else {
        Command::new("codex")
            .args(["exec", "--ephemeral", prompt])
            .output()
    }
    .map_err(|e| format!("spawn error: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("exit {}: {stderr}", output.status));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if stdout.trim().is_empty() {
        return Err("empty response".to_string());
    }

    Ok(stdout)
}
