use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() {
    // Get the directory where the output will be placed.
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("git_commit.rs");
    let mut f = File::create(dest_path).unwrap();

    // Use git to get the current commit hash.
    let git_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    let git_hash = String::from_utf8_lossy(&git_output.stdout).trim().to_string();

    // Write the git hash to a file as a constant.
    writeln!(f, "pub const GIT_COMMIT: &str = \"{}\";", git_hash).unwrap();
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
}