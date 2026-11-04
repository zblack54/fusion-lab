fn main() {
    println!(
        "cargo:rustc-env=HISTORY_TEST_RUSTC_PATH={}",
        std::env::var("RUSTC").expect("Cargo supplies its selected compiler")
    );
    println!("cargo:rerun-if-changed=build.rs");
}
