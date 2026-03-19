//! Development environment setup commands
//!
//! Platform-specific setup for FFmpeg, OpenCV, and build tools.
//!
//! # Dependency Strategy
//!
//! - FFmpeg: Manually installed by the user (set FFMPEG_DIR env var)
//! - OpenCV: Installed via official Windows release or system package manager
//! - LLVM/Clang: Required for opencv-rs bindgen

use anyhow::Result;
use colored::*;
use std::process::{Command, Stdio};

#[allow(unused_imports)]
use crate::execute_command;

/// Run setup for the current platform
pub fn run_setup(skip_verify: bool) -> Result<()> {
    println!(
        "{}",
        "=== ascii-term Development Environment Setup ==="
            .bold()
            .blue()
    );
    println!();

    let os = std::env::consts::OS;
    println!("{} Detected platform: {}", "→".blue(), os.bold());
    println!();

    match os {
        "windows" => setup_windows(skip_verify),
        "linux" => setup_linux(skip_verify),
        "macos" => setup_macos(skip_verify),
        _ => {
            println!("{} Unsupported platform: {}", "✗".red().bold(), os);
            anyhow::bail!("Unsupported platform")
        }
    }
}

#[cfg(target_os = "windows")]
fn setup_windows(skip_verify: bool) -> Result<()> {
    use std::path::Path;

    println!("{}", "Checking prerequisites...".bold());
    check_command("git", "Git not found. Install from: https://git-scm.com/")?;
    check_command("cargo", "Rust not found. Install from: https://rustup.rs/")?;

    let has_choco = Command::new("choco")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if has_choco {
        println!("{} Chocolatey found", "✓".green());
    } else {
        println!(
            "{} Chocolatey not found (optional but recommended)",
            "⚠".yellow()
        );
        println!(
            "   {} Install from: {}",
            "→".blue(),
            "https://chocolatey.org/".cyan()
        );
    }

    if !is_admin()? {
        println!();
        println!("{}", "✗ Administrator privileges required".red().bold());
        println!();
        println!(
            "{} Please run this command in an elevated terminal:",
            "→".blue()
        );
        println!(
            "   {}",
            "1. Right-click on your terminal (PowerShell/Git Bash)".cyan()
        );
        println!("   {}", "2. Select 'Run as Administrator'".cyan());
        println!(
            "   {}",
            "3. Navigate to this directory and run: cargo x setup".cyan()
        );
        println!();
        anyhow::bail!("Administrator privileges required");
    }

    println!("{} Prerequisites OK", "✓".green());
    println!();

    // Build tools (cmake, pkg-config)
    println!("{}", "Setting up build tools...".bold());
    setup_build_tools()?;
    println!();

    // FFmpeg: install pre-built binaries directly to D:\libs\ffmpeg
    println!("{}", "Setting up FFmpeg...".bold());
    setup_ffmpeg_windows()?;
    println!();

    // OpenCV: install directly
    println!("{}", "Setting up OpenCV...".bold());
    let opencv_dir = Path::new("D:\\libs\\opencv");
    if !opencv_dir.join("build").exists() {
        println!("{} OpenCV not found, installing...", "→".blue());

        run_powershell("if (!(Test-Path D:\\libs)) { New-Item -ItemType Directory -Path D:\\libs }")?;
        run_powershell("if (!(Test-Path D:\\libs\\tmp)) { New-Item -ItemType Directory -Path D:\\libs\\tmp }")?;

        println!(
            "{} Downloading OpenCV (this may take a while)...",
            "→".blue()
        );
        run_powershell(
            "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -Uri 'https://github.com/opencv/opencv/releases/download/4.9.0/opencv-4.9.0-windows.exe' -OutFile 'D:\\libs\\tmp\\opencv.exe' -UseBasicParsing",
        )?;

        println!("{} Extracting OpenCV...", "→".blue());
        run_powershell(
            "Start-Process -FilePath D:\\libs\\tmp\\opencv.exe -ArgumentList '-oD:\\libs -y' -Wait",
        )?;

        println!("{} Cleaning up temporary files...", "→".blue());
        let _ = run_powershell(
            "Remove-Item -Path 'D:\\libs\\tmp\\opencv.exe' -Force -ErrorAction SilentlyContinue",
        );

        println!("{} OpenCV installed to D:\\libs\\opencv", "✓".green());
    } else {
        println!("{} OpenCV already installed", "✓".green());
    }

    println!("{} Setting OpenCV environment variables...", "→".blue());
    run_powershell(
        "[Environment]::SetEnvironmentVariable('OPENCV_DIR', 'D:\\libs\\opencv\\build', 'User')",
    )?;
    run_powershell(
        "[Environment]::SetEnvironmentVariable('OPENCV_INCLUDE_PATHS', 'D:\\libs\\opencv\\build\\include', 'User')",
    )?;
    run_powershell(
        "[Environment]::SetEnvironmentVariable('OPENCV_LINK_PATHS', 'D:\\libs\\opencv\\build\\x64\\vc16\\lib', 'User')",
    )?;
    run_powershell(
        "[Environment]::SetEnvironmentVariable('OPENCV_LINK_LIBS', 'opencv_world490', 'User')",
    )?;
    run_powershell(
        "$path = [Environment]::GetEnvironmentVariable('Path', 'User'); if ($path -notlike '*D:\\libs\\opencv\\build\\x64\\vc16\\bin*') { [Environment]::SetEnvironmentVariable('Path', $path + ';D:\\libs\\opencv\\build\\x64\\vc16\\bin', 'User') }",
    )?;
    println!("{} Environment variables set", "✓".green());
    println!();

    // LLVM: required for opencv-rs clang-runtime feature
    println!("{}", "Setting up LLVM (clang)...".bold());
    let llvm_installed = Command::new("clang")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !llvm_installed {
        let llvm_path = Path::new("C:\\Program Files\\LLVM\\bin\\clang.exe");
        if !llvm_path.exists() {
            println!(
                "{} LLVM/clang not found, installing via Chocolatey...",
                "→".blue()
            );

            let mut cmd = Command::new("choco");
            cmd.arg("install").arg("llvm").arg("-y");

            let status = cmd
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status();

            match status {
                Ok(s) if s.success() => {
                    println!("{} LLVM installed", "✓".green());
                }
                _ => {
                    println!("{} LLVM installation failed", "✗".red().bold());
                    println!(
                        "{} Please install LLVM manually from: {}",
                        "→".blue(),
                        "https://releases.llvm.org/".cyan()
                    );
                    anyhow::bail!("Failed to install LLVM");
                }
            }
        } else {
            println!("{} LLVM already installed (not in PATH)", "✓".green());
            println!("{} Adding LLVM to PATH...", "→".blue());
            run_powershell(
                "$path = [Environment]::GetEnvironmentVariable('Path', 'User'); if ($path -notlike '*C:\\Program Files\\LLVM\\bin*') { [Environment]::SetEnvironmentVariable('Path', $path + ';C:\\Program Files\\LLVM\\bin', 'User') }",
            )?;
        }
    } else {
        println!("{} LLVM/clang already installed", "✓".green());
    }
    println!();

    if !skip_verify {
        println!("{}", "Verifying installation...".bold());
        verify_windows_setup()?;
    }

    println!();
    println!("{}", "✅ Setup complete!".green().bold());
    println!();
    println!(
        "{} Please restart your terminal to apply environment variable changes",
        "ℹ".blue()
    );
    println!("{} Then run: {}", "→".blue(), "cargo build".cyan());

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn setup_windows(_skip_verify: bool) -> Result<()> {
    anyhow::bail!("Windows setup can only be run on Windows")
}

#[cfg(target_os = "windows")]
fn setup_ffmpeg_windows() -> Result<()> {
    use std::path::Path;

    // FFmpeg via vcpkg (MSVC build) - required for compatibility with the MSVC Rust toolchain
    let vcpkg_dir = Path::new("D:\\libs\\vcpkg");
    let vcpkg_exe = vcpkg_dir.join("vcpkg.exe");
    let ffmpeg_lib = Path::new("D:\\libs\\vcpkg\\installed\\x64-windows\\lib\\avcodec.lib");

    // Step 1: Install or update vcpkg
    if vcpkg_exe.exists() {
        println!("{} vcpkg already installed at D:\\libs\\vcpkg", "✓".green());
        println!("{} Updating vcpkg...", "→".blue());
        let _ = Command::new("git")
            .args(["pull"])
            .current_dir(vcpkg_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    } else {
        println!("{} Installing vcpkg to D:\\libs\\vcpkg...", "→".blue());

        run_powershell("if (!(Test-Path D:\\libs)) { New-Item -ItemType Directory -Path D:\\libs }")?;

        let clone = Command::new("git")
            .args(["clone", "--depth=1", "https://github.com/microsoft/vcpkg.git", "D:\\libs\\vcpkg"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();

        if clone.map(|s| !s.success()).unwrap_or(true) {
            anyhow::bail!("Failed to clone vcpkg repository");
        }

        println!("{} Bootstrapping vcpkg...", "→".blue());
        let bootstrap = Command::new("cmd")
            .args(["/C", "bootstrap-vcpkg.bat", "-disableMetrics"])
            .current_dir(vcpkg_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();

        if bootstrap.map(|s| !s.success()).unwrap_or(true) {
            anyhow::bail!("Failed to bootstrap vcpkg");
        }

        println!("{} vcpkg installed to D:\\libs\\vcpkg", "✓".green());
    }

    // Step 2: Install FFmpeg via vcpkg (MSVC x64-windows build)
    if ffmpeg_lib.exists() {
        println!("{} FFmpeg (MSVC) already installed via vcpkg", "✓".green());
    } else {
        println!(
            "{} Installing FFmpeg via vcpkg (this may take 10-30 minutes)...",
            "→".blue()
        );
        let install = Command::new(vcpkg_exe.to_str().unwrap())
            .args([
                "install",
                "ffmpeg[core,avcodec,avformat,avfilter,swresample,swscale,gpl]:x64-windows",
                "--recurse",
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();

        if install.map(|s| !s.success()).unwrap_or(true) {
            anyhow::bail!("Failed to install FFmpeg via vcpkg");
        }

        println!("{} FFmpeg installed", "✓".green());
    }

    // Step 3: Set environment variables
    println!("{} Setting FFmpeg environment variables...", "→".blue());
    run_powershell(
        "[Environment]::SetEnvironmentVariable('FFMPEG_DIR', 'D:\\libs\\vcpkg\\installed\\x64-windows', 'User')",
    )?;
    run_powershell(
        "[Environment]::SetEnvironmentVariable('VCPKG_ROOT', 'D:\\libs\\vcpkg', 'User')",
    )?;
    run_powershell(
        "$path = [Environment]::GetEnvironmentVariable('Path', 'User'); if ($path -notlike '*D:\\libs\\vcpkg\\installed\\x64-windows\\bin*') { [Environment]::SetEnvironmentVariable('Path', $path + ';D:\\libs\\vcpkg\\installed\\x64-windows\\bin', 'User') }",
    )?;
    println!("{} Environment variables set", "✓".green());
    println!("   {} FFMPEG_DIR=D:\\libs\\vcpkg\\installed\\x64-windows", "→".blue());

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn setup_ffmpeg_windows() -> Result<()> {
    Ok(())
}

#[cfg(target_os = "windows")]
fn setup_build_tools() -> Result<()> {
    use std::path::Path;

    let cmake_installed = Command::new("cmake")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !cmake_installed {
        println!("{} CMake not found, installing...", "→".blue());

        run_powershell("if (!(Test-Path C:\\tmp)) { New-Item -ItemType Directory -Path C:\\tmp }")?;

        println!("{} Downloading CMake installer...", "→".blue());
        run_powershell(
            "Invoke-WebRequest -Uri 'https://github.com/Kitware/CMake/releases/download/v3.28.3/cmake-3.28.3-windows-x86_64.msi' -OutFile 'C:\\tmp\\cmake.msi'",
        )?;

        println!("{} Installing CMake...", "→".blue());
        run_powershell(
            "Start-Process msiexec.exe -ArgumentList '/i', 'C:\\tmp\\cmake.msi', '/quiet', '/norestart', 'ADD_CMAKE_TO_PATH=System' -Wait",
        )?;

        println!("{} Cleaning up temporary files...", "→".blue());
        run_powershell(
            "Remove-Item -Path C:\\tmp\\cmake.msi -Force -ErrorAction SilentlyContinue",
        )?;

        run_powershell(
            "$env:Path = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User')",
        )?;

        let cmake_now_installed = Path::new("C:\\Program Files\\CMake\\bin\\cmake.exe").exists();

        if !cmake_now_installed {
            println!("{} CMake installation failed", "✗".red().bold());
            println!("{} Please install CMake manually:", "→".blue());
            println!("   {}", "https://cmake.org/download/".cyan());
            anyhow::bail!("Failed to install CMake");
        }

        println!("{} CMake installed", "✓".green());
    } else {
        println!("{} CMake already installed", "✓".green());
    }

    let pkgconfig_installed = Command::new("pkg-config")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !pkgconfig_installed {
        println!(
            "{} pkg-config not found, attempting to install...",
            "→".blue()
        );
        let mut cmd = Command::new("choco");
        cmd.arg("install").arg("pkgconfiglite").arg("-y");
        let _ = cmd.stdout(Stdio::null()).stderr(Stdio::null()).status();

        let pkgconfig_now_installed = Command::new("pkg-config")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if pkgconfig_now_installed {
            println!("{} pkg-config installed", "✓".green());
        } else {
            println!(
                "{} pkg-config not available (optional on Windows)",
                "⚠".yellow()
            );
        }
    } else {
        println!("{} pkg-config already installed", "✓".green());
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn setup_build_tools() -> Result<()> {
    Ok(())
}

#[cfg(target_os = "windows")]
fn verify_windows_setup() -> Result<()> {
    use std::path::Path;

    println!("{}", "=== Verification ===".bold());
    println!();

    // FFmpeg (via vcpkg)
    let ffmpeg_lib = Path::new("D:\\libs\\vcpkg\\installed\\x64-windows\\lib\\avcodec.lib");
    if ffmpeg_lib.exists() {
        println!("{} FFmpeg OK (D:\\libs\\vcpkg\\installed\\x64-windows)", "✓".green());
    } else {
        println!("{} FFmpeg not found - run `cargo x setup` to install", "✗".red());
    }

    // OpenCV
    let opencv_dll = Path::new("D:\\libs\\opencv\\build\\x64\\vc16\\bin\\opencv_world490.dll");
    if opencv_dll.exists() {
        println!("{} OpenCV OK", "✓".green());
    } else {
        println!("{} OpenCV not found", "✗".red());
    }

    // LLVM
    let llvm_exe = Path::new("C:\\Program Files\\LLVM\\bin\\clang.exe");
    if llvm_exe.exists() {
        println!("{} LLVM OK", "✓".green());
    } else {
        println!("{} LLVM not found", "✗".red());
    }

    println!();
    println!("{}", "=== Environment Variables ===".bold());

    if std::env::var("FFMPEG_DIR").is_ok() {
        println!("{} FFMPEG_DIR set", "✓".green());
    } else {
        println!(
            "{} FFMPEG_DIR not set (restart terminal after setup)",
            "⚠".yellow()
        );
    }

    if std::env::var("OPENCV_DIR").is_ok() {
        println!("{} OPENCV_DIR set", "✓".green());
    } else {
        println!(
            "{} OPENCV_DIR not set (restart terminal after setup)",
            "⚠".yellow()
        );
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn verify_windows_setup() -> Result<()> {
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn setup_linux(skip_verify: bool) -> Result<()> {
    println!("{}", "Checking prerequisites...".bold());
    check_command("git", "Git not found")?;
    check_command("cargo", "Rust not found. Install from: https://rustup.rs/")?;
    println!("{} Prerequisites OK", "✓".green());
    println!();

    #[cfg(target_os = "linux")]
    {
        println!("{}", "Installing dependencies via apt...".bold());
        println!("{} Updating package list...", "→".blue());
        let mut cmd = Command::new("sudo");
        cmd.args(["apt-get", "update", "-y"]);
        execute_command(&mut cmd)?;

        println!("{} Installing build tools...", "→".blue());
        let mut cmd = Command::new("sudo");
        cmd.args([
            "apt-get", "install", "-y",
            "cmake", "pkg-config", "clang", "libclang-dev",
        ]);
        execute_command(&mut cmd)?;

        println!("{} Installing FFmpeg dev libraries...", "→".blue());
        let mut cmd = Command::new("sudo");
        cmd.args([
            "apt-get", "install", "-y",
            "libavcodec-dev", "libavformat-dev", "libavutil-dev",
            "libavdevice-dev", "libavfilter-dev",
            "libswscale-dev", "libswresample-dev",
        ]);
        execute_command(&mut cmd)?;

        println!("{} Installing OpenCV...", "→".blue());
        let mut cmd = Command::new("sudo");
        cmd.args([
            "apt-get", "install", "-y",
            "libopencv-dev",
        ]);
        execute_command(&mut cmd)?;

        println!("{} Installing audio libraries...", "→".blue());
        let mut cmd = Command::new("sudo");
        cmd.args(["apt-get", "install", "-y", "libasound2-dev"]);
        execute_command(&mut cmd)?;
    }

    println!();

    if !skip_verify {
        println!("{}", "Verifying installation...".bold());
        verify_linux_setup()?;
    }

    println!();
    println!("{}", "✅ Setup complete!".green().bold());
    println!("{} You can now run: {}", "→".blue(), "cargo build".cyan());

    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn setup_linux(_skip_verify: bool) -> Result<()> {
    anyhow::bail!("Linux/macOS setup can only be run on Linux/macOS")
}

#[cfg(target_os = "macos")]
fn setup_macos(skip_verify: bool) -> Result<()> {
    println!("{}", "Checking prerequisites...".bold());
    check_command("git", "Git not found")?;
    check_command("cargo", "Rust not found. Install from: https://rustup.rs/")?;
    check_command("brew", "Homebrew not found. Install from: https://brew.sh/")?;
    println!("{} Prerequisites OK", "✓".green());
    println!();

    println!("{}", "Installing dependencies via Homebrew...".bold());

    println!("{} Installing build tools...", "→".blue());
    let mut cmd = Command::new("brew");
    cmd.args(["install", "cmake", "pkg-config", "llvm"]);
    execute_command(&mut cmd)?;

    // brew's LLVM is not in PATH by default; set LIBCLANG_PATH so opencv-rs bindgen works
    println!("{} Configuring LLVM (brew)...", "→".blue());
    let llvm_prefix_output = Command::new("brew")
        .args(["--prefix", "llvm"])
        .output();
    if let Ok(out) = llvm_prefix_output {
        if out.status.success() {
            let prefix = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let libclang_path = format!("{}/lib", prefix);
            println!(
                "{} LIBCLANG_PATH={} (add to your shell profile)",
                "→".blue(),
                libclang_path.cyan()
            );
            println!(
                "   {}",
                format!("export LIBCLANG_PATH={}", libclang_path).cyan()
            );
            // Set for current process so cargo build works immediately after setup
            std::env::set_var("LIBCLANG_PATH", &libclang_path);
        }
    }

    println!("{} Installing OpenCV...", "→".blue());
    let mut cmd = Command::new("brew");
    cmd.args(["install", "opencv"]);
    execute_command(&mut cmd)?;

    println!("{} Installing FFmpeg...", "→".blue());
    let mut cmd = Command::new("brew");
    cmd.args(["install", "ffmpeg"]);
    execute_command(&mut cmd)?;

    println!();

    if !skip_verify {
        println!("{}", "Verifying installation...".bold());
        verify_macos_setup()?;
    }

    println!();
    println!("{}", "✅ Setup complete!".green().bold());
    println!("{} You can now run: {}", "→".blue(), "cargo build".cyan());

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn setup_macos(_skip_verify: bool) -> Result<()> {
    anyhow::bail!("macOS setup can only be run on macOS")
}

fn check_command(cmd: &str, error_msg: &str) -> Result<()> {
    let result = Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match result {
        Ok(status) if status.success() => {
            println!("{} {} found", "✓".green(), cmd);
            Ok(())
        }
        _ => {
            println!("{} {}", "✗".red().bold(), error_msg);
            anyhow::bail!(error_msg.to_string())
        }
    }
}

#[cfg(target_os = "windows")]
fn run_powershell(script: &str) -> Result<()> {
    let mut cmd = Command::new("powershell");
    cmd.arg("-NoLogo")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(script);

    let status = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if !status.success() {
        anyhow::bail!("PowerShell command failed");
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn run_powershell(_script: &str) -> Result<()> {
    anyhow::bail!("PowerShell is only available on Windows")
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn verify_linux_setup() -> Result<()> {
    check_pkg_config("libavcodec", "FFmpeg")?;
    check_pkg_config("opencv4", "OpenCV")?;

    #[cfg(target_os = "linux")]
    check_pkg_config("alsa", "ALSA")?;

    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[allow(dead_code)]
fn verify_linux_setup() -> Result<()> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn verify_macos_setup() -> Result<()> {
    verify_linux_setup()
}

#[cfg(not(target_os = "macos"))]
#[allow(dead_code)]
fn verify_macos_setup() -> Result<()> {
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn check_pkg_config(package: &str, name: &str) -> Result<()> {
    let result = Command::new("pkg-config")
        .arg("--exists")
        .arg(package)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match result {
        Ok(status) if status.success() => {
            println!("{} {} OK", "✓".green(), name);
            Ok(())
        }
        _ => {
            println!("{} {} not found", "✗".red(), name);
            Ok(()) // warn only
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[allow(dead_code)]
fn check_pkg_config(_package: &str, _name: &str) -> Result<()> {
    Ok(())
}

#[cfg(target_os = "windows")]
fn is_admin() -> Result<bool> {
    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg("([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)")
        .output()?;

    let is_admin = String::from_utf8_lossy(&output.stdout)
        .trim()
        .eq_ignore_ascii_case("true");

    Ok(is_admin)
}

#[cfg(not(target_os = "windows"))]
fn is_admin() -> Result<bool> {
    Ok(false)
}
