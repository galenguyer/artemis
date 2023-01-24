fn main() {
    println!("cargo:rerun-if-changed=web/");
    std::process::Command::new("pnpm")
        .args(&["run", "build"])
        .current_dir("web")
        .status()
        .expect("Failed to run npm build");
}
