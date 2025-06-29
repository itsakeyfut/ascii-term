use std::env;

fn main() {
    // FFmpeg 初期化
    println!("cargo:rerun-if-changed=build.rs");

    // プラットフォーム固有の設定
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    match target_os.as_str() {
        "windows" => {
            configure_windows();
        }
        "macos" => {
            unimplemented!()
        }
        "linux" => {
            unimplemented!()
        }
        _ => {
            println!("cargo:warning=Unsupported target OS: {}", target_os);
        }
    }
}

fn configure_windows() {
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=kernel32");

    // Windows での vcpkg 使用を推奨
    if env::var("VCPKG_ROOT").is_ok() {
        println!("cargo:rustc-link-search=native={}\\installed\\x64-windows\\lib",
                env::var("VCPKG_ROOT").unwrap());
    }
}