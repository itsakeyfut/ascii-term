use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

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

    configure_opencv();
}

fn configure_windows() {
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=kernel32");

    // FFmpeg: avio が FFMPEG_DIR を参照する
    if let Ok(ffmpeg_dir) = env::var("FFMPEG_DIR") {
        println!("cargo:rustc-link-search=native={}\\lib", ffmpeg_dir);
    }
}

fn configure_macos() {
    println!("cargo:rustc-link-search=native=/opt/homebrew/lib");
    println!("cargo:rustc-link-search=native=/usr/local/lib");

    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    println!("cargo:rustc-link-lib=framework=CoreMedia");
    println!("cargo:rustc-link-lib=framework=CoreVideo");
    println!("cargo:rustc-link-lib=framework=AVFoundation");
}

fn configure_linux() {
    println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
    println!("cargo:rustc-link-search=native=/usr/local/lib");

    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=dl");
    println!("cargo:rustc-link-lib=m");
}

fn configure_opencv() {
    if let Ok(opencv_dir) = env::var("OPENCV_DIR") {
        println!("cargo:rustc-link-search=native={}/lib", opencv_dir);
        println!("cargo:rustc-link-search=native={}/x64/vc16/lib", opencv_dir);
    }
}
