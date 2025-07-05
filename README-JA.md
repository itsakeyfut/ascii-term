# ASCII ターミナルメディアプレイヤー

[![Status](https://img.shields.io/badge/status-active--development-brightgreen?style=flat-square)]()
[![Rust Version](https://img.shields.io/badge/rust-1.87+-blue.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

![bad_apple](./gallery/bad_apple.gif)

![cookie_bomb_rush](./gallery/cookie_bomb_rush.gif)

まだあるよ ⇒ [Gallery](./gallery/gallery.md)

- [English](./README.md)

ターミナル上でメディアファイル（動画・音声・画像）をリアルタイムで ASCII アートに変換して再生する Rust 製メディアプレイヤー。音声同期再生とカラフルなターミナル出力が特徴です。

## 概要

このプロジェクトは、Rust における高度なメディア処理能力を実証し、リアルタイム動画デコード、ASCII アート生成、ターミナル環境での同期音声再生を実現しています。明確な責任分離を持つモジュラーアーキテクチャで構築されており、機能的なメディアプレイヤーとしてだけでなく、将来の動画編集ソフトウェア開発の基盤としても機能します。

## 主な機能

### メディア再生

- **多形式対応**: MP4、AVI、MKV、MOV、MP3、FLAC、WAV、JPG、PNG など
- **リアルタイム ASCII 描画**: 10 種類の文字マップオプションによる高品質な動画-ASCII 変換
- **同期音声再生**: Rodio を使用したフレーム精度の音声・動画同期
- **カラー ASCII アート**: RGB 色情報を保持したカラフルなターミナル出力
- **動的ターミナル適応**: 自動スケーリング付きライブリサイズ対応
- **インタラクティブ制御**: キーボードショートカットによる再生/一時停止/停止/ミュート
- **ループ再生**: 連続メディア再生機能

### 文字マップバリエーション

| インデックス | 名前           | 説明                       |
| ------------ | -------------- | -------------------------- |
| 0            | Basic ASCII    | 10 文字 (` .:-=+*#%@`)     |
| 1            | Extended ASCII | 拡張 67 文字セット         |
| 2            | Full ASCII     | 完全 92 文字セット         |
| 3            | Unicode Blocks | ブロック文字 (` ░▒▓█`)     |
| 4            | Braille        | 点字パターン               |
| 5            | Dots           | ドットベース文字           |
| 6            | Gradient       | グラデーションブロック文字 |
| 7            | Binary         | 2 値（白黒）文字           |
| 8            | Binary Dots    | 2 値ドットパターン         |
| 9            | Emoji Style    | 絵文字スタイル文字         |

### 技術アーキテクチャ

- **モジュラー設計**: コアメディア処理（`media-core`）と UI 表示（`terminal-player`）の明確な分離
- **安全性優先アプローチ**: 堅牢な開発のための FFmpeg と OpenCV ラッパークレートの広範囲使用
- **マルチスレッド処理**: 動画デコード、音声処理、ターミナル描画の独立スレッド
- **メモリ効率ストリーミング**: 最適化されたメモリ使用量による制御されたバッファリング
- **クロスプラットフォーム互換性**: Linux、macOS、Windows ターミナルでの動作

## プロジェクト構成

```
├── Cargo.toml              # ワークスペース設定
├── media-core/             # コアメディア処理ライブラリ
│   ├── src/
│   │   ├── video/          # 動画デコード・処理
│   │   ├── audio/          # 音声デコード・処理
│   │   ├── image/          # 画像処理
│   │   ├── media.rs        # メディアファイル管理
│   │   ├── pipeline.rs     # 処理パイプライン
│   │   └── errors.rs       # エラー定義
│   ├── build.rs            # ビルドスクリプト
│   └── Cargo.toml
├── terminal-player/        # ターミナルプレイヤーアプリ
│   ├── src/
│   │   ├── renderer.rs     # ASCIIレンダラー
│   │   ├── terminal.rs     # ターミナル管理
│   │   ├── player.rs       # プレイヤー制御
│   │   ├── audio.rs        # 音声再生
│   │   ├── char_maps.rs    # 文字マップ定義
│   │   └── main.rs         # メインアプリケーション
│   └── Cargo.toml
└── downloader/             # ダウンロード機能（実験的）
    ├── src/
    │   ├── youtube.rs      # YouTube対応
    │   ├── errors.rs       # エラー定義
    │   └── lib.rs
    └── Cargo.toml
```

## システム要件

### 依存関係

#### Linux (Ubuntu/Debian)

```bash
sudo apt update
sudo apt install -y build-essential pkg-config
sudo apt install -y libavformat-dev libavcodec-dev libavutil-dev libswscale-dev libswresample-dev
sudo apt install -y libopencv-dev libclang-dev
```

#### Linux (CentOS/RHEL/Fedora)

```bash
sudo dnf install -y ffmpeg-devel opencv-devel clang-devel pkg-config
```

#### macOS (Homebrew)

```bash
brew install ffmpeg opencv pkg-config
```

#### Windows (vcpkg 推奨)

```bash
vcpkg install ffmpeg opencv[core,imgproc,videoio]
```

### オプション依存関係

YouTube 動画ダウンロード機能用：

```bash
pip install yt-dlp
```

手動メディアダウンロード例：

```bash
yt-dlp -f best https://www.youtube.com/watch?v=SW3GGXbLDv4 -o video.mp4
```

## インストール

### Cargo からインストール

```bash
cargo install terminal-player
```

### ソースからビルド

```bash
git clone https://github.com/itsakeyfut/ascii-term.git
cd ascii-term
cargo build --release
```

## 使い方

### 基本コマンド

```bash
# 動画ファイルを再生
terminal-player video.mp4

# 音声ファイルを再生
terminal-player music.mp3

# 画像を表示
terminal-player image.jpg

# YouTube URL（実験的）
terminal-player "https://www.youtube.com/watch?v=n8CojbNl2ZA"

# ループ再生
terminal-player -l video.mp4

# カスタムフレームレート
terminal-player --fps 24 video.mp4

# グレースケールモード
terminal-player -g video.mp4
```

### コマンドラインオプション

```
使い方:
    terminal-player [OPTIONS] <INPUT>

引数:
    <INPUT>    入力ファイルパスまたはURL

オプション:
    -f, --fps <FPS>              特定のフレームレートを強制
    -b, --browser <BROWSER>      Cookie抽出用ブラウザ [default: firefox]
    -l, --loop-playback          ループ再生
    -c, --char-map <CHAR_MAP>    文字マップ選択 (0-9) [default: 0]
    -g, --gray                   グレースケールモードを有効化
    -w, --width-mod <WIDTH_MOD>  文字アスペクト比の幅修正値 [default: 1]
        --allow-frame-skip       遅延時のフレームスキップを許可
    -n, --newlines               出力に改行を追加
        --no-audio               音声再生を無効化
    -h, --help                   ヘルプ情報を表示
    -V, --version                バージョン情報を表示
```

### インタラクティブ制御

| キー   | 機能                   |
| ------ | ---------------------- |
| Space  | 再生/一時停止切り替え  |
| Q, Esc | プレイヤー終了         |
| M      | ミュート/ミュート解除  |
| G      | グレースケール切り替え |
| 0-9    | 文字マップ切り替え     |
| H      | ヘルプ表示             |

## ライブラリ使用方法

### コアメディア処理

```rust
use media_core::{MediaFile, MediaType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ライブラリ初期化
    media_core::init()?;

    // メディアファイルを開く
    let media = MediaFile::open("video.mp4")?;

    println!("Media Type: {:?}", media.media_type);
    println!("Duration: {:?}", media.info.duration);

    if media.info.has_video {
        println!("Resolution: {}x{}",
                 media.info.width.unwrap_or(0),
                 media.info.height.unwrap_or(0));
    }

    Ok(())
}
```

### 動画デコード

```rust
use media_core::{MediaFile, video::VideoDecoder};

let media = MediaFile::open("video.mp4")?;
let mut decoder = VideoDecoder::new(&media)?;

// フレームをデコード
while let Ok((stream, packet)) = media.format_context().read_packet() {
    if let Some(frame) = decoder.decode_next_frame(&packet)? {
        println!("Frame: {}x{} at {:?}",
                 frame.width, frame.height, frame.timestamp);
    }
}
```

### ASCII 描画

```rust
use terminal_player::renderer::{AsciiRenderer, RenderConfig};
use image::open;

let config = RenderConfig {
    target_width: 80,
    target_height: 24,
    char_map_index: 0,
    grayscale: false,
    add_newlines: true,
    width_modifier: 1,
};

let mut renderer = AsciiRenderer::new(config);
let image = open("image.jpg")?;
let result = renderer.render_image(&image)?;

println!("{}", result.ascii_text);
```

## 技術的ハイライト

### 設計思想

- **ラッパークレートによる安全性**: メモリ安全性とエラーハンドリングのための FFmpeg と OpenCV Rust ラッパーの広範囲使用
- **モジュラーアーキテクチャ**: コアメディア処理と UI 表示レイヤーの明確な分離
- **将来対応基盤**: 包括的な動画編集ソフトウェアへの踏み台として設計
- **責任分離**: 再利用性のため`media-core`にメディア処理ロジックを分離

### パフォーマンス特性

- **メモリ使用量**: 制御されたバッファリングによる最適化されたストリーミング
- **CPU 効率**: 最小オーバーヘッドのマルチスレッド処理
- **リアルタイム性能**: 低遅延でのフレーム精度同期
- **ターミナル互換性**: 様々なターミナルサイズと機能への適応的描画

## 現在の制限事項と今後の開発

### 既知の課題

- **音声同期タイミング**: 特定のシナリオでの軽微なタイミングずれ
- **メモリ使用量最適化**: さらなるメモリ効率改善の機会
- **画像表示タイミング**: 静止画タイミング同期の改良が必要

これらの課題は主に`terminal-player`UI レイヤーに集中しており、集中的な改善努力の良いターゲットとなっています。

### 実験的機能

- **YouTube ダウンロード機能**: 実装済みだが、信頼性のある動作には改良が必要
- **URL からダウンロード**: ウェブベースのメディア取得（現在不安定）

信頼性のある動作のため、事前にローカルメディアファイルを準備することを推奨します。`yt-dlp`ツールは優秀なメディアダウンロード機能を提供します：

```bash
# 簡単なメディア取得のためのyt-dlpインストール
pip install yt-dlp

yt-dlp -f best "https://www.youtube.com/watch?v=fuLPnN62p1g" -o "%(title)s.%(ext)s"
```

### ロードマップ

- **拡張動画編集機能**: `media-core`基盤の上に構築
- **高度なフィルタリングとエフェクト**: 静止画処理と動画フィルター
- **パフォーマンス最適化**: メモリ使用量と処理効率の改善
- **UI/UX 拡張**: より良いターミナルプレイヤー体験
- **ストリーミング対応**: ネットワークメディアストリーム処理
- **プラグインアーキテクチャ**: 拡張可能な処理モジュール

## トラブルシューティング

### ビルド問題

1. **FFmpeg not found**

```bash
# pkg-configをインストール
sudo apt install pkg-config  # Linux
brew install pkg-config      # macOS
```

2. **OpenCV not found**

```bash
export OPENSSL_DIR=/usr
export OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
export OPENSSL_INCLUDE_DIR=/usr/include
```

3. **Windows vcpkg 問題**

```bash
set VCPKG_ROOT=C:\vcpkg
vcpkg integrate install
```

### 実行時問題

1. **音声デバイスが見つからない**
   - 音声デバイスが利用できない場合は`--no-audio`フラグを使用
   - Ubuntu/WSL での音声設定：

```bash
# Ubuntu/WSL用音声設定
sudo apt install pulseaudio-module-protocol-native
pulseaudio --start

# PulseAudio設定
export PULSE_SERVER="unix:/mnt/wslg/runtime-dir/pulse/native"
echo "default-server = unix:/mnt/wslg/runtime-dir/pulse/native" > ~/.pulse/client.conf

# 音声設定確認
pactl info
pactl list short sinks

# 音声再生テスト
paplay /usr/share/sounds/alsa/Front_Left.wav 2>/dev/null || echo "Test sound not available"
```

2. **YouTube ダウンロード失敗**

   - yt-dlp が最新であることを確認
   - 異なるブラウザオプションを試す（`--browser chrome`）

3. **ターミナル表示問題**
   - より大きなターミナルウィンドウを使用
   - `--width-mod 2`で文字幅を調整
   - 異なる文字マップを試す（0-9）

## 貢献

貢献を歓迎します！特に関心のある分野：

- **`terminal-player`での性能最適化**
- **拡張文字マッピングアルゴリズム**
- **音声同期改善**
- **メモリ使用量最適化**
- **クロスプラットフォーム互換性拡張**

## ライセンス

MIT License

## 作者

**itsakeyfut**

このプロジェクトは、リアルタイムメディア処理、ASCII アート生成、同期音声再生を実証する高度な Rust マルチメディアプログラミング技術を紹介し、包括的な動画編集ソフトウェアへの将来の拡張に適したクリーンでモジュラーなアーキテクチャを維持しています。
FFmpeg/OpenCV 統合、マルチスレッドアーキテクチャ、革新的なターミナルベースメディア表示を特徴とする、Rust でのマルチメディア処理の技術的探求。機能的なメディアプレイヤーと将来の動画編集アプリケーションの基盤の両方として設計されています。
