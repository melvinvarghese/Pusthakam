fn main() {
    tauri_build::build();

    // Integration tests that start the Tauri runtime crash on Windows (STATUS_ENTRYPOINT_NOT_FOUND)
    // without a Common Controls v6 manifest. Embed one for test executables only; the app keeps
    // Tauri's own manifest.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        let dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
        let manifest = std::path::Path::new(&dir).join("windows-test-manifest.xml");
        println!("cargo:rerun-if-changed={}", manifest.display());
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}", manifest.display());
    }
}
