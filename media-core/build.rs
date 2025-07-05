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
            configure_macos();
        }
        "linux" => {
            configure_linux();
        }
        _ => {
            println!("cargo:warning=Unsupported target OS: {}", target_os);
        }
    }

    // OpenCV 設定
    configure_opencv();

    // FFmpeg 設定
    configure_ffmpeg();
}

fn configure_windows() {
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=kernel32");

    // Windows での vcpkg 使用を推奨
    if let Ok(vcpkg_root) = env::var("VCPKG_ROOT") {
        println!("cargo:rustc-link-search=native={}\\installed\\x64-windows\\lib", vcpkg_root);
    }
}

fn configure_macos() {
    // Homebrew のパスを追加
    println!("cargo:rustc-link-search=native=/opt/homebrew/lib");
    println!("cargo:rustc-link-search=native=/usr/local/lib");

    // macOS 固有のフレームワーク
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    println!("cargo:rustc-link-lib=framework=CoreMedia");
    println!("cargo:rustc-link-lib=framework=CoreVideo");
    println!("cargo:rustc-link-lib=framework=AVFoundation");
}

fn configure_linux() {
    // Linux ライブラリパスを追加
    println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
    println!("cargo:rustc-link-search=native=/usr/local/lib");

    // システムライブラリ
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=dl");
    println!("cargo:rustc-link-lib=m");
}

fn configure_opencv() {
    // OpenCV 環境変数チェック
    if let Ok(opencv_dir) = env::var("OPENCV_DIR") {
        println!("cargo:rustc-link-search=native={}/lib", opencv_dir);
    }

    // pkg-config を使用して OpenCV を検出
    if pkg_config::probe_library("opencv4").or_else(|_| pkg_config::probe_library("opencv")).is_err() {
        println!("cargo:warning=OpenCV not found via pkg-config, using system defaults");
    }
}

fn configure_ffmpeg() {
    // FFmpeg 環境変数チェック
    if let Ok(ffmpeg_dir) = env::var("FFMPEG_DIR") {
        println!("cargo:rustc-link-search=native={}/lib", ffmpeg_dir);
    }

    // pkg-config を使用して FFmpeg を検出
    let ffmpeg_libs = ["libavformat", "libavcodec", "libavutil", "libswscale", "libswresample"];

    for lib in &ffmpeg_libs {
        if pkg_config::probe_library(lib).is_err() {
            println!("cargo:warning={} not found via pkg-config", lib);
        }
    }

    // 静的リンクが有効な場合の設定
    if env::var("CARGO_FEATURE_FFMPEG_STATIC").is_ok() {
        println!("cargo:rustc-link-lib=static=avformat");
        println!("cargo:rustc-link-lib=static=avcodec");
        println!("cargo:rustc-link-lib=static=avutil");
        println!("cargo:rustc-link-lib=static=swscale");
        println!("cargo:rustc-link-lib=static=swresample");
    }
}