# ASCII Term

ターミナル上でメディアファイル（動画・音声・画像）を ASCII アートに変換して再生する Rust ライブラリ。

## 特徴

- 動画再生: FFmpeg-next と OpenCV を使用した高品質な動画デコード
- 音声再生: Rodio による音声再生（同期再生対応）
- 画像表示: 様々な画像形式をサポート
- カラー表示: RGB 色情報を保持したカラフルな ASCII アート
- ライブリサイズ: ターミナルサイズに動的対応
- 複数文字マップ: 10 種類の文字セットから選択可能
- YouTube 対応: yt-dlp を使用した YouTube 動画ダウンロード
- ループ再生: 動画のループ再生機能
- 再生制御: 再生/一時停止/停止/ミュート制御

## システム要件

### 依存ライブラリ

#### Windows (vcpkg 推奨)

```sh
vcpkg install ffmpeg opencv[core,imgproc,videoio]
```

#### macOS (Homebrew)

```sh
brew install ffmpeg opencv pkg-config
```

#### Linux (Ubuntu/Debian)

```sh
sudo apt update
sudo apt install -y build-essential pkg-config
sudo apt install -y libavformat-dev libavcodec-dev libavutil-dev libswscale-dev libswresample-dev
sudo apt install -y libopencv-dev libclang-dev
```

#### Linux (CentOS/RHEL/Fedora)

```sh
sudo dnf install -y ffmpeg-devel opencv-devel clang-devel pkg-config
```

### オプション依存

YouTube 動画ダウンロード機能を使用する場合：

```sh
pip install yt-dlp
```

手動でメディアをダウンロードする場合：

```sh
yt-dlp -f best https://www.youtube.com/watch?v=FtutLA63Cp8 -o test.mp4
```

## インストール

### Cargo からインストール

```sh
cargo install terminal-player
```

### ソースからビルド

```sh
git clone git@github.com:itsakeyfut/ascii-term.git
cd ascii-term
cargo build --release
```

## 使い方

### 基本的な使い方

```sh
# 動画ファイルを再生
terminal-player video.mp4

# 音声ファイルを再生
terminal-player music.mp3

# 画像を表示
terminal-player image.jpg

# YouTube動画を再生
terminal-player "https://www.youtube.com/watch?v=VIDEO_ID"

# ループ再生
terminal-player -l video.mp4

# カスタムFPS指定
terminal-player --fps 24 video.mp4

# グレースケールモード
terminal-player -g video.mp4
```

### コマンドライン引数

```
USAGE:
    terminal-player [OPTIONS] <INPUT>

ARGS:
    <INPUT>    Input file path or URL

OPTIONS:
    -f, --fps <FPS>              Force specific frame rate
    -b, --browser <BROWSER>      Browser for cookie extraction [default: firefox]
    -l, --loop-playback          Loop playback
    -c, --char-map <CHAR_MAP>    Character map selection (0-9) [default: 0]
    -g, --gray                   Enable grayscale mode
    -w, --width-mod <WIDTH_MOD>  Width modifier for character aspect ratio [default: 1]
        --allow-frame-skip       Allow frame skipping when behind
    -n, --newlines               Add newlines to output
        --no-audio               Disable audio playback
    -h, --help                   Print help information
    -V, --version                Print version information
```

### キーボード操作

再生中に以下のキーで操作できます：

| キー   | 機能                   |
| ------ | ---------------------- |
| Space  | 再生/一時停止          |
| Q, Esc | 終了                   |
| M      | ミュート/ミュート解除  |
| G      | グレースケール切り替え |
| 0-9    | 文字マップ変更         |
| H      | ヘルプ表示             |

### 文字マップ

| インデックス | 名前           | 説明                            |
| ------------ | -------------- | ------------------------------- |
| 0            | Basic ASCII    | 基本的な 10 文字 (` .:-=+*#%@`) |
| 1            | Extended ASCII | 拡張 67 文字セット              |
| 2            | Full ASCII     | 完全 92 文字セット              |
| 3            | Unicode Blocks | ブロック文字 (` ░▒▓█`)          |
| 4            | Braille        | 点字文字                        |
| 5            | Dots           | ドット文字                      |
| 6            | Gradient       | グラデーションブロック          |
| 7            | Binary         | 2 値文字（白黒）                |
| 8            | Binary Dots    | 2 値ドット文字                  |
| 9            | Emoji Style    | 絵文字風文字                    |

## プロジェクト構成

```
├── Cargo.toml              # ワークスペース設定
├── media-core/             # メディア処理コアライブラリ
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
│   │   └── main.rs         # メイン関数
│   └── Cargo.toml
└── downloader/             # ダウンロード機能
    ├── src/
    │   ├── youtube.rs      # YouTube対応
    │   ├── errors.rs       # エラー定義
    │   └── lib.rs
    └── Cargo.toml
```

## ライブラリとして使用

### media-core

```rs
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

### ビデオでコード

```rs
use media_core::{MediaFile, video::VideoDecoder};

let media = MediaFile::open("video.mp4")?;
let mut decoder = VideoDecoder::new(&media)?;

// フレームをデコード
while let Ok((stream, packet)) = media.format_context().read_packet() {
    if let Some(frame) = decoder.decode_next_frame(&packet)? {
        // フレーム処理
        println!("Frame: {}x{} at {:?}",
                 frame.width, frame.height, frame.timestamp);
    }
}
```

### ASCII レンダラー

```rs
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

## トラブルシューティング

### ビルドエラー

1. FFmpeg not found

```sh
# pkg-configをインストール
sudo apt install pkg-config  # Linux
brew install pkg-config      # macOS
```

2. OpenCV not found

```sh
export OPENSSL_DIR=/usr
export OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu
export OPENSSL_INCLUDE_DIR=/usr/include
```

3. Windows vcpkg issues

```sh
set VCPKG_ROOT=C:\vcpkg
vcpkg integrate install
```

### 実行時エラー

1. Audio device not found

- 音声デバイスが利用できない場合は `--no-audio` フラグを使用

もし音声ありを再生したい場合、セットアップが必要です。

#### 以下手順

検証しながらセットアップしたため、定まった手順が明確になっていません。

環境：Ubuntu

```
sudo apt install pulseaudio-module-protocol-native
pulseaudio --start

mkdir -p ~/.pulse
echo "default-server = tcp:localhost:4713" > ~/.pulse/client.conf

aplay -l
pulseaudio --check -v

export PULSE_SERVER="tcp:127.0.0.1:4713"
pactl info

echo $DISPLAY
echo $XDG_RUNTIME_DIR
ls -la /mnt/wslg/runtime-dir/pulse/
export PULSE_SERVER="unix:/mnt/wslg/runtime-dir/pulse/native"
echo "default-server = unix:/mnt/wslg/runtime-dir/pulse/native" > ~/.pulse/client.conf
pactl info
pactl list short sinks
```

```
paplay /usr/share/sounds/alsa/Front_Left.wav 2>/dev/null || echo "Test sound not available"
pactl get-sink-volume RDPSink
```

2. YouTube download fails

- yt-dlp が最新版であることを確認
- ブラウザを変更してみる（`--browser chrome` など）

3. Terminal size issues

- より大きなターミナルウィンドウを使用
- `--width-mod 2` で文字幅を調整

## 将来の計画

- より多くのコーデックサポート
- リアルタイムエフェクト
- プレイリスト機能
- 設定ファイル対応
- ストリーミング対応
