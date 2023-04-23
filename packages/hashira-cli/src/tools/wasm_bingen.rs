use super::{InstallOptions, Tool};

pub struct WasmBindgen;

#[async_trait::async_trait]
impl Tool for WasmBindgen {
    fn name(&self) -> &'static str {
        "wasm-bingen"
    }

    fn bin_name(&self) -> &'static str {
        if cfg!(target_os = "windows") {
            "wasm-bingen.exe"
        } else {
            "wasm-bingen"
        }
    }

    fn version(&self) -> &'static str {
        "0.2.84"
    }

    async fn test_version(&self) -> anyhow::Result<String> {
        todo!()
    }

    async fn get(opts: InstallOptions) -> anyhow::Result<Self> {
        todo!()
    }

    async fn exec(&self, args: Vec<String>) -> anyhow::Result<()> {
        todo!()
    }
}

fn get_url(version: &str) -> anyhow::Result<String> {
    let target_os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        anyhow::bail!("unsupported OS")
    };

    let os = match target_os {
        "windows" => "pc-windows-msvc",
        "macos" => "apple-darwin",
        "linux" => "unknown-linux-musl",
        _ => unreachable!(),
    };

    Ok(format!("https://github.com/rustwasm/wasm-bindgen/releases/download/{version}/wasm-bindgen-{version}-x86_64-{os}.tar.gz"))
}
