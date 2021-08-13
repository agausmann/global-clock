fn main() {
    // By default, Cargo will re-run the script (and transitively, rebuild the crate) if any file
    // in the project changes, defeating the purpose of keeping assets external.
    // Suppress this behavior:
    println!("cargo:rerun-if-changed=build.rs");

    println!(
        "cargo:rustc-env=ASSETS_ROOT={}/assets",
        std::env::var("PWD").unwrap()
    );
}
