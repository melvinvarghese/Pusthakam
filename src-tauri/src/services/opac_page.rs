use base64::Engine;

/// All public catalogue assets are embedded in the desktop binary.
pub fn render(server_name: &str) -> String {
    let name = server_name
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;");
    let sans = base64::engine::general_purpose::STANDARD.encode(include_bytes!(
        "../../../node_modules/@fontsource-variable/hanken-grotesk/files/hanken-grotesk-latin-wght-normal.woff2"
    ));
    let mono = base64::engine::general_purpose::STANDARD.encode(include_bytes!(
        "../../../node_modules/@fontsource/courier-prime/files/courier-prime-latin-400-normal.woff2"
    ));
    let css = include_str!("opac/styles.css")
        .replace("{{SANS_FONT}}", &sans)
        .replace("{{MONO_FONT}}", &mono);
    let logo = base64::engine::general_purpose::STANDARD
        .encode(include_bytes!("../../../src/assets/pusthakam-logo.png"));
    include_str!("opac/index.html")
        .replace("{{STYLES}}", &css)
        .replace("{{SCRIPT}}", include_str!("opac/catalogue.js"))
        .replace("{{LOGO}}", &logo)
        .replace("{{LIBRARY_NAME}}", &name)
}

#[cfg(test)]
mod tests {
    #[test]
    fn catalogue_embeds_assets_and_escapes_library_name() {
        let html = super::render("<script>alert('library')</script> & Books");
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("data:font/woff2;base64,"));
        assert!(html.contains("/api/opac/search?"));
        assert!(!html.contains("{{"));
        assert!(!html.contains("Balyakalasakhi"));
    }
}
