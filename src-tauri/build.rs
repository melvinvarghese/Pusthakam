fn main() {
    embed_staff_assets();
    tauri_build::build();

    // Integration tests that start the Tauri runtime crash on Windows (STATUS_ENTRYPOINT_NOT_FOUND)
    // without a Common Controls v6 manifest. Embed one for test executables only; the app keeps
    // Tauri's own manifest.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        let dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
        let manifest = std::path::Path::new(&dir).join("windows-test-manifest.xml");
        println!("cargo:rerun-if-changed={}", manifest.display());
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}",
            manifest.display()
        );
    }
}

fn embed_staff_assets() {
    use std::{fs, path::Path};
    let manifest =
        std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest directory"));
    let dist = manifest.join("../dist");
    println!("cargo:rerun-if-changed={}", dist.display());
    fn collect(root: &Path, folder: &Path, entries: &mut Vec<(String, String, String)>) {
        let Ok(files) = fs::read_dir(folder) else {
            return;
        };
        for file in files.flatten() {
            let path = file.path();
            let Ok(kind) = file.file_type() else {
                continue;
            };
            if kind.is_dir() {
                collect(root, &path, entries);
            } else if kind.is_file() {
                let Ok(relative) = path.strip_prefix(root) else {
                    continue;
                };
                let web = format!("/{}", relative.to_string_lossy().replace('\\', "/"));
                let mime = match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
                    "html" => "text/html; charset=utf-8",
                    "js" => "text/javascript; charset=utf-8",
                    "css" => "text/css; charset=utf-8",
                    "png" => "image/png",
                    "svg" => "image/svg+xml",
                    "ico" => "image/x-icon",
                    "woff2" => "font/woff2",
                    "woff" => "font/woff",
                    "json" => "application/json",
                    _ => "application/octet-stream",
                };
                if let Ok(absolute) = path.canonicalize() {
                    entries.push((web, mime.into(), absolute.to_string_lossy().into()));
                }
            }
        }
    }
    let mut entries = Vec::new();
    collect(&dist, &dist, &mut entries);
    entries.sort();
    let mut generated =
        String::from("pub fn asset(path:&str)->Option<(&'static str,&'static [u8])>{match path{\n");
    for (web, mime, file) in entries {
        generated.push_str(&format!(
            "{web:?}=>Some(({mime:?},include_bytes!({file:?}))),\n"
        ));
    }
    generated.push_str("_=>None}}\n");
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("build output"));
    fs::write(out.join("staff_assets.rs"), generated).expect("write embedded staff assets");
}
