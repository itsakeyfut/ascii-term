//! ASCII文字マップの定義
//!
//! 様々な密度と特性を持つ文字セットを提供し、
//! 画像の明度をテキスト文字にマッピングするために使用

/// 基本的なASCII文字セット（10文字）
pub const CHARS_BASIC: &str = " .:-=+*#%@";

/// 拡張ASCII文字セット（67文字）
pub const CHARS_EXTENDED: &str =
    r#" .'`^",:;Il!i~+_-?][}{1)(|/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$"#;

/// 完全ASCII文字セット（92文字）
pub const CHARS_FULL: &str = r#" `.-':_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@"#;

/// ブロック文字（Unicode）
pub const CHARS_BLOCKS: &str = " ░▒▓█";

/// 点字文字
pub const CHARS_BRAILLE: &str = " ⠁⠃⠇⠏⠟⠿⣿";

/// ドット文字
pub const CHARS_DOTS: &str = " ·∶⁚⁛⁜⁝⁞ ⣿";

/// グラデーション文字
pub const CHARS_GRADIENT: &str = " ▁▂▃▄▅▆▇█";

/// 2値文字（白黒）
pub const CHARS_BINARY: &str = " █";

/// 2値ドット文字
pub const CHARS_BINARY_DOTS: &str = " ⣿";

/// 絵文字風文字
pub const CHARS_EMOJI: &str = " ·•○●";

/// 利用可能な全ての文字マップ
pub const CHAR_MAPS: &[&str] = &[
    CHARS_BASIC,
    CHARS_EXTENDED,
    CHARS_FULL,
    CHARS_BLOCKS,
    CHARS_BRAILLE,
    CHARS_DOTS,
    CHARS_GRADIENT,
    CHARS_BINARY,
    CHARS_BINARY_DOTS,
    CHARS_EMOJI,
];

/// 文字マップの説明
pub const CHAR_MAP_NAMES: &[&str] = &[
    "Basic ASCII (10 chars)",
    "Extended ASCII (67 chars)",
    "Full ASCII (92 chars)",
    "Unicode Blocks",
    "Braille Characters",
    "Dot Characters",
    "Gradient Blocks",
    "Binary (Black/White)",
    "Binary Dots",
    "Emoji Style",
];

/// 指定されたインデックスの文字マップを取得
pub fn get_char_map(index: u8) -> &'static str {
    let index = (index as usize) % CHAR_MAPS.len();
    CHAR_MAPS[index]
}

/// 指定されたインデックスの文字マップ名を取得
pub fn get_char_map_name(index: u8) -> &'static str {
    let index = (index as usize) % CHAR_MAP_NAMES.len();
    CHAR_MAP_NAMES[index]
}

/// 文字マップの総数を取得
pub fn char_map_count() -> usize {
    CHAR_MAPS.len()
}

/// 明度値（0-255）を文字にマッピング
pub fn luminance_to_char(luminance: u8, char_map: &str) -> char {
    let chars: Vec<char> = char_map.chars().collect();
    if chars.is_empty() {
        return ' ';
    }

    let index = (luminance as usize * chars.len()) / 256;
    let index = index.min(chars.len() - 1);
    chars[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_map_access() {
        assert!(!get_char_map(0).is_empty());
        assert!(!get_char_map(255).is_empty()); // オーバーフローテスト
        assert_eq!(get_char_map(0), CHARS_BASIC);
    }

    #[test]
    fn test_luminance_mapping() {
        let char_map = CHARS_BASIC;
        assert_eq!(luminance_to_char(0, char_map), ' ');
        assert_eq!(luminance_to_char(255, char_map), '@');

        // 中間値のテスト
        let mid_char = luminance_to_char(128, char_map);
        assert!(mid_char != ' ' && mid_char != '@');
    }

    #[test]
    fn test_char_map_names() {
        assert_eq!(CHAR_MAPS.len(), CHAR_MAP_NAMES.len());
        assert!(!get_char_map_name(0).is_empty());
    }
}
