use crate::server::hashira;

#[allow(dead_code)]
pub async fn generate_html_index() {
    println!("🟦 Generating layout html...");

    let service = hashira();
    let html = service.get_layout_html().await;
    let mut path = std::path::Path::new(".").to_path_buf();

    if !path.exists() {
        std::fs::create_dir_all(&path)
            .unwrap_or_else(|_| panic!("📛 failed to create directory: {}", path.display()));
    }

    // Append file name
    path.push("index.html");

    {
        let file_path = path.canonicalize().unwrap();
        println!("🟦 Writing layout html to: {}", file_path.display());
    }

    std::fs::write(path, html).expect("📛 Failed to write layout html");
    println!("✅ Done!\n")
}
