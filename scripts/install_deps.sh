#!/bin/bash
# ascii-term - System Dependencies Installation Script
#
# This script installs all required system libraries for building ascii-term.
# Supports: Ubuntu/Debian, Fedora, Arch Linux, macOS (Homebrew)
# Windows: use `cargo x setup` instead.

set -e  # Exit on error

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_info()    { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
print_error()   { echo -e "${RED}[ERROR]${NC} $1"; }

# Detect operating system
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if [ -f /etc/debian_version ]; then
            echo "debian"
        elif [ -f /etc/fedora-release ]; then
            echo "fedora"
        elif [ -f /etc/arch-release ]; then
            echo "arch"
        else
            echo "linux-unknown"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    else
        echo "unknown"
    fi
}

install_debian() {
    print_info "Installing dependencies for Ubuntu/Debian..."

    sudo apt-get update

    print_info "Installing FFmpeg development libraries..."
    sudo apt-get install -y \
        libavcodec-dev \
        libavformat-dev \
        libavutil-dev \
        libavdevice-dev \
        libavfilter-dev \
        libswscale-dev \
        libswresample-dev

    print_info "Installing OpenCV..."
    sudo apt-get install -y libopencv-dev

    print_info "Installing build tools..."
    sudo apt-get install -y \
        pkg-config \
        clang \
        libclang-dev \
        cmake

    print_info "Installing audio libraries..."
    sudo apt-get install -y libasound2-dev

    print_success "All dependencies installed successfully!"
}

install_fedora() {
    print_info "Installing dependencies for Fedora..."

    print_info "Installing FFmpeg development libraries..."
    sudo dnf install -y ffmpeg-devel

    print_info "Installing OpenCV..."
    sudo dnf install -y opencv-devel

    print_info "Installing build tools..."
    sudo dnf install -y \
        pkg-config \
        clang \
        clang-devel \
        cmake

    print_info "Installing audio libraries..."
    sudo dnf install -y alsa-lib-devel

    print_success "All dependencies installed successfully!"
}

install_arch() {
    print_info "Installing dependencies for Arch Linux..."

    sudo pacman -Sy
    sudo pacman -S --needed --noconfirm \
        ffmpeg \
        opencv \
        alsa-lib \
        pkg-config \
        clang \
        cmake

    print_success "All dependencies installed successfully!"
}

install_macos() {
    print_info "Installing dependencies for macOS..."

    if ! command -v brew &> /dev/null; then
        print_error "Homebrew is not installed. Please install it first:"
        print_info "Visit: https://brew.sh/"
        exit 1
    fi

    brew update

    print_info "Installing FFmpeg..."
    brew install ffmpeg

    print_info "Installing OpenCV..."
    brew install opencv

    print_info "Installing build tools..."
    brew install pkg-config cmake llvm

    # brew's LLVM is not in PATH by default; opencv-rs bindgen needs LIBCLANG_PATH
    LLVM_PREFIX=$(brew --prefix llvm 2>/dev/null)
    if [ -n "$LLVM_PREFIX" ]; then
        print_info "Setting LIBCLANG_PATH for current session..."
        export LIBCLANG_PATH="${LLVM_PREFIX}/lib"
        print_success "LIBCLANG_PATH=${LIBCLANG_PATH}"
        print_warning "Add to your shell profile (~/.zshrc or ~/.bash_profile):"
        print_info "  export LIBCLANG_PATH=${LLVM_PREFIX}/lib"
    fi

    if ! xcode-select -p &> /dev/null; then
        print_warning "Xcode Command Line Tools not found. Installing..."
        xcode-select --install
    fi

    print_success "All dependencies installed successfully!"
}

verify_installation() {
    print_info "Verifying installation..."

    local has_error=0

    if command -v pkg-config &> /dev/null; then
        print_success "pkg-config found"

        if pkg-config --exists libavcodec; then
            print_success "FFmpeg found (version: $(pkg-config --modversion libavcodec))"
        else
            print_error "FFmpeg libraries not found via pkg-config"
            has_error=1
        fi

        if pkg-config --exists opencv4 || pkg-config --exists opencv; then
            print_success "OpenCV found"
        else
            print_warning "OpenCV not found (optional)"
        fi
    else
        print_error "pkg-config not found"
        has_error=1
    fi

    return $has_error
}

main() {
    echo ""
    print_info "ascii-term - Dependency Installation Script"
    echo "============================================="
    echo ""

    OS=$(detect_os)
    print_info "Detected OS: $OS"
    echo ""

    case $OS in
        debian)   install_debian ;;
        fedora)   install_fedora ;;
        arch)     install_arch ;;
        macos)    install_macos ;;
        *)
            print_error "Unsupported OS: $OS"
            print_info "On Windows, use: cargo x setup"
            exit 1
            ;;
    esac

    echo ""

    if verify_installation; then
        echo ""
        print_success "Installation completed successfully!"
        print_info "You can now build ascii-term: cargo build"
    else
        echo ""
        print_warning "Installation completed with some warnings."
        print_info "Please check the errors above and install missing dependencies manually."
        exit 1
    fi
}

main
