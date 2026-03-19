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

    // FFmpeg: manually installed, pointed to by FFMPEG_DIR
    if let Ok(ffmpeg_dir) = env::var("FFMPEG_DIR") {
        println!(
            "cargo:rustc-link-search=native={}\\lib",
            ffmpeg_dir
        );
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
    // OpenCV 環境変数チェック (OPENCV_DIR は cargo x setup で設定される)
    if let Ok(opencv_dir) = env::var("OPENCV_DIR") {
        println!("cargo:rustc-link-search=native={}/lib", opencv_dir);
        println!("cargo:rustc-link-search=native={}/x64/vc16/lib", opencv_dir);
    }
}

fn configure_ffmpeg() {
    // FFmpeg 環境変数チェック (FFMPEG_DIR はユーザーが手動でインストールして設定する)
    if let Ok(ffmpeg_dir) = env::var("FFMPEG_DIR") {
        println!("cargo:rustc-link-search=native={}/lib", ffmpeg_dir);
    } else {
        println!("cargo:warning=FFMPEG_DIR is not set. Please install FFmpeg and set FFMPEG_DIR.");
    }
}
