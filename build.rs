use std::process::Command;

fn main() {
    add_git_revision();
}

fn add_git_revision() {
    let rev = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| output.status.success().then_some(output.stdout))
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .map_or_else(|| "unknown".to_string(), |value| value.trim().to_string());

    println!("cargo:rustc-env=GIT_REVISION={rev}");
}
