use std::path::PathBuf;
use std::process::Command;

fn git_output(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn git_path(path: &str) -> Option<PathBuf> {
    let path = PathBuf::from(git_output(&["rev-parse", "--git-path", path])?);
    if path.is_absolute() {
        Some(path)
    } else {
        Some(std::env::current_dir().ok()?.join(path))
    }
}

fn track_git_state() -> bool {
    let Some(head) = git_path("HEAD") else {
        return false;
    };

    if head.exists() {
        println!("cargo:rerun-if-changed={}", head.display());
    }

    if let Some(reference) = git_output(&["symbolic-ref", "-q", "HEAD"]) {
        if let Some(reference_path) = git_path(&reference) {
            if reference_path.exists() {
                println!("cargo:rerun-if-changed={}", reference_path.display());
            }
        }
    }

    true
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // If a file named `.tag` is present, we'll take its contents for the
    // version number that we report in wezterm -h.
    let mut ci_tag = String::new();
    if let Ok(tag) = std::fs::read("../.tag") {
        if let Ok(s) = String::from_utf8(tag) {
            ci_tag = s.trim().to_string();
            println!("cargo:rerun-if-changed=../.tag");
        }
    } else if track_git_state() {
        // Otherwise derive it from git without linking libgit2 into this build script.
        if let Some(info) = git_output(&[
            "-c",
            "core.abbrev=8",
            "show",
            "-s",
            "--format=%cd-%h",
            "--date=format:%Y%m%d-%H%M%S",
        ]) {
            ci_tag = info;
        }
    }

    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());

    println!("cargo:rustc-env=WEZTERM_TARGET_TRIPLE={target}");
    println!("cargo:rustc-env=WEZTERM_CI_TAG={ci_tag}");
}
