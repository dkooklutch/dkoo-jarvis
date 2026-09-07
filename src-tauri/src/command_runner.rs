use crate::{
    error::{JarvisError, Result},
    models::CommandResult,
    permission_gate::PermissionGate,
};
use std::{collections::HashMap, path::Path, process::Stdio, time::Instant};
use tokio::{
    process::Command,
    time::{timeout, Duration},
};

const TIMEOUT_SECONDS: u64 = 300;
const META_CHARS: [&str; 12] = [
    ";", "&&", "||", "|", "`", "$(", ">", "<", "\n", "\r", "*", "?",
];

fn validate(program: &str, args: &[String]) -> Result<()> {
    let allowed = [
        "git", "npm", "pnpm", "yarn", "bun", "npx", "node", "python", "python3", "pytest",
    ];
    if !allowed.contains(&program) {
        return Err(JarvisError::Security(format!(
            "program '{program}' is not approved"
        )));
    }
    if args
        .iter()
        .any(|a| META_CHARS.iter().any(|m| a.contains(m)))
    {
        return Err(JarvisError::Security(
            "shell metacharacters are forbidden".into(),
        ));
    }
    let joined = format!("{program} {}", args.join(" ")).to_ascii_lowercase();
    for forbidden in [
        "sudo",
        " rm ",
        "rm -",
        "unlink",
        "rmdir",
        "shred",
        "reset --hard",
        "push --force",
        "push -f",
        "branch -d",
        "clean -f",
        "rebase",
        "filter-branch",
        "osascript",
        "security dump-keychain",
        "diskutil",
        "killall",
        "pkill",
    ] {
        if joined.contains(forbidden) {
            return Err(JarvisError::Security(format!(
                "forbidden command pattern: {forbidden}"
            )));
        }
    }
    if program == "git" {
        let sub = args.first().map(String::as_str).unwrap_or("");
        if ![
            "status",
            "diff",
            "log",
            "show",
            "branch",
            "rev-parse",
            "ls-files",
            "switch",
            "checkout",
            "add",
            "commit",
        ]
        .contains(&sub)
        {
            return Err(JarvisError::Security(format!(
                "git subcommand '{sub}' requires a dedicated approved workflow"
            )));
        }
    }
    if ["npm", "pnpm", "yarn", "bun"].contains(&program) {
        let sub = args.first().map(String::as_str).unwrap_or("");
        if !["run", "test", "install", "add", "exec", "lint", "build"].contains(&sub) {
            return Err(JarvisError::Security(format!(
                "package-manager operation '{sub}' is not approved"
            )));
        }
    }
    if ["node", "python", "python3"].contains(&program)
        && !args
            .iter()
            .any(|a| a == "--version" || a == "-V" || a == "-m")
    {
        return Err(JarvisError::Security(
            "direct interpreters are limited to version checks or approved modules".into(),
        ));
    }
    Ok(())
}

pub async fn run(
    gate: &PermissionGate,
    project_id: &str,
    program: &str,
    args: Vec<String>,
) -> Result<CommandResult> {
    validate(program, &args)?;
    let root = gate.root(project_id)?;
    run_at(&root, program, args).await
}

async fn run_at(root: &Path, program: &str, args: Vec<String>) -> Result<CommandResult> {
    let start = Instant::now();
    let mut cmd = Command::new(program);
    cmd.args(&args)
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .env_clear();
    let safe: HashMap<&str, String> = ["PATH", "LANG", "LC_ALL", "TERM"]
        .into_iter()
        .filter_map(|k| std::env::var(k).ok().map(|v| (k, v)))
        .collect();
    cmd.envs(safe);
    let output = timeout(Duration::from_secs(TIMEOUT_SECONDS), cmd.output())
        .await
        .map_err(|_| JarvisError::Operation("command timed out and was terminated".into()))??;
    Ok(CommandResult {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        exit_code: output.status.code().unwrap_or(-1),
        duration_ms: start.elapsed().as_millis(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }
    #[test]
    fn allows_safe_reads() {
        assert!(validate("git", &s(&["status", "--short"])).is_ok())
    }
    #[test]
    fn blocks_sudo() {
        assert!(validate("sudo", &s(&["git", "status"])).is_err())
    }
    #[test]
    fn blocks_reset_hard() {
        assert!(validate("git", &s(&["reset", "--hard"])).is_err())
    }
    #[test]
    fn blocks_force_push() {
        assert!(validate("git", &s(&["push", "--force"])).is_err())
    }
    #[test]
    fn blocks_delete() {
        assert!(validate("rm", &s(&["-rf", "."])).is_err())
    }
    #[test]
    fn blocks_shell_injection() {
        assert!(validate("npm", &s(&["test;", "cat", ".env"])).is_err());
        assert!(validate("git", &s(&["status", "$(whoami)"])).is_err())
    }
    #[test]
    fn blocks_osascript() {
        assert!(validate("osascript", &s(&["-e", "tell app"])).is_err())
    }
}
