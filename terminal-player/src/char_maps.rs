//! ASCII character map definition
//!
//! Provides character sets with various densities and characteristics,
//! used to map image luminosity to text characters

/// Basic ASCII character set (10 characters)
pub const CHARS_BASIC: &str = " .:-=+*#%@";

/// Extended ASCII character set (67 characters)
pub const CHARS_EXTENDED: &str =
    r#" .'`^",:;Il!i~+_-?][}{1)(|/tfjrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$"#;

/// Full ASCII character set (92 characters)
pub const CHARS_FULL: &str = r#" `.-':_,^=;><+!rc*/z?sLTv)J7(|Fi{C}fI31tlu[neoZ5Yxjya]2ESwqkP6h9d4VpOGbUAKXHm8RD#$Bg0MNWQ%&@"#;

/// Block characters (Unicode)
pub const CHARS_BLOCKS: &str = " ░▒▓█";

/// Braille
pub const CHARS_BRAILLE: &str = " ⠁⠃⠇⠏⠟⠿⣿";

/// Dot
pub const CHARS_DOTS: &str = " ·∶⁚⁛⁜⁝⁞ ⣿";

/// Gradient
pub const CHARS_GRADIENT: &str = " ▁▂▃▄▅▆▇█";

/// Binary character (B/W)
pub const CHARS_BINARY: &str = " █";

/// Binary dot character
pub const CHARS_BINARY_DOTS: &str = " ⣿";

/// Pictograph-style character
pub const CHARS_EMOJI: &str = " ·•○●";

/// All available character maps
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

/// Character map description
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

/// Obtains a character map for a given index
pub fn get_char_map(index: u8) -> &'static str {
    let index = (index as usize) % CHAR_MAPS.len();
    CHAR_MAPS[index]
}

/// Obtains the character map name at the specified index
pub fn get_char_map_name(index: u8) -> &'static str {
    let index = (index as usize) % CHAR_MAP_NAMES.len();
    CHAR_MAP_NAMES[index]
}

/// Get the total number of character maps
pub fn char_map_count() -> usize {
    CHAR_MAPS.len()
}

/// Mapping lightness values (0-255) to characters
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
        assert!(!get_char_map(255).is_empty()); // overflow test
        assert_eq!(get_char_map(0), CHARS_BASIC);
    }

    #[test]
    fn test_luminance_mapping() {
        let char_map = CHARS_BASIC;
        assert_eq!(luminance_to_char(0, char_map), ' ');
        assert_eq!(luminance_to_char(255, char_map), '@');

        // Intermediate value test
        let mid_char = luminance_to_char(128, char_map);
        assert!(mid_char != ' ' && mid_char != '@');
    }

    #[test]
    fn test_char_map_names() {
        assert_eq!(CHAR_MAPS.len(), CHAR_MAP_NAMES.len());
        assert!(!get_char_map_name(0).is_empty());
    }
}
