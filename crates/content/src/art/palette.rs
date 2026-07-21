//! Master palette (~48 RGBA) + char ↔ index helpers.
//! Charset: `.` = transparent (0), `0-9` → 1..10, `a-z` → 11..36, `A-Z` → 37..62.

/// Transparent sentinel (always index 0 / `.`).
pub const TRANSPARENT: u8 = 0;

// Digits `0`–`9` → indices 1–10
pub const TUNIC_LIGHT: u8 = 1; // 0
pub const TUNIC: u8 = 2; // 1
pub const TUNIC_MID: u8 = 3; // 2
pub const TUNIC_DARK: u8 = 4; // 3
pub const GRASS_LIGHT: u8 = 5; // 4
pub const GRASS: u8 = 6; // 5
pub const GRASS_DARK: u8 = 7; // 6
pub const LEAF: u8 = 8; // 7
pub const WATER_SHIMMER: u8 = 9; // 8
pub const SLIME: u8 = 10; // 9

// `a`–`z` → 11–36
pub const SKIN_LIGHT: u8 = 11; // a
pub const SKIN: u8 = 12; // b
pub const SKIN_SHADOW: u8 = 13; // c
pub const HAIR: u8 = 14; // d
pub const HAIR_SHADOW: u8 = 15; // e
pub const CREAM: u8 = 16; // f
pub const STONE_LIGHT: u8 = 17; // g
pub const STONE: u8 = 18; // h
pub const STONE_MID: u8 = 19; // i
pub const STONE_DARK: u8 = 20; // j
pub const STONE_DEEP: u8 = 21; // k
pub const FLOOR_A: u8 = 22; // l
pub const FLOOR_B: u8 = 23; // m
pub const WALL_TOP: u8 = 24; // n
pub const RED_LIGHT: u8 = 25; // o
pub const RED: u8 = 26; // p
pub const RED_MID: u8 = 27; // q
pub const RED_DARK: u8 = 28; // r
pub const BLOOD: u8 = 29; // s
pub const BOOT: u8 = 30; // t
pub const BLUE_LIGHT: u8 = 31; // u
pub const BLUE: u8 = 32; // v
pub const BLUE_MID: u8 = 33; // w
pub const TEAL: u8 = 34; // x
pub const TEAL_DARK: u8 = 35; // y
pub const WATER: u8 = 36; // z

// `A`–`Z` → 37+
pub const GOLD_LIGHT: u8 = 37; // A
pub const GOLD: u8 = 38; // B
pub const GOLD_DARK: u8 = 39; // C
pub const YELLOW: u8 = 40; // D
pub const WHITE: u8 = 41; // E
pub const OUTLINE: u8 = 42; // F
pub const SHADOW: u8 = 43; // G
pub const SLIME_DARK: u8 = 44; // H
pub const SLIME_HIGHLIGHT: u8 = 45; // I
pub const BAT: u8 = 46; // J
pub const BAT_WING: u8 = 47; // K
pub const OCTOROK: u8 = 48; // L
pub const OCTOROK_DARK: u8 = 49; // M
pub const ROCK: u8 = 50; // N
pub const ROCK_DARK: u8 = 51; // O
pub const PURPLE: u8 = 52; // P
pub const UI_PANEL: u8 = 53; // Q
pub const UI_PANEL_EDGE: u8 = 54; // R
pub const BOOT_DARK: u8 = 55; // S
pub const CAPE: u8 = 56; // T
pub const EYE_WHITE: u8 = 57; // U
pub const EYE_DARK: u8 = 58; // V

const PALETTE: [[u8; 4]; 59] = [
    [0, 0, 0, 0],         // 0 .
    [120, 220, 96, 255],  // 1 0 tunic light
    [64, 176, 64, 255],   // 2 1 tunic
    [40, 128, 48, 255],   // 3 2 tunic mid
    [24, 88, 32, 255],    // 4 3 tunic dark
    [112, 184, 72, 255],  // 5 4 grass light
    [72, 140, 48, 255],   // 6 5 grass
    [48, 96, 32, 255],    // 7 6 grass dark
    [160, 200, 80, 255],  // 8 7 leaf
    [80, 180, 200, 255],  // 9 8 water shimmer
    [96, 200, 112, 255],  // 10 9 slime
    [255, 214, 176, 255], // 11 a skin light
    [232, 176, 128, 255], // 12 b skin
    [196, 128, 88, 255],  // 13 c skin shadow
    [72, 48, 32, 255],    // 14 d hair
    [48, 32, 24, 255],    // 15 e hair shadow
    [255, 244, 220, 255], // 16 f cream
    [200, 196, 184, 255], // 17 g stone light
    [160, 156, 144, 255], // 18 h stone
    [120, 116, 108, 255], // 19 i stone mid
    [80, 76, 72, 255],    // 20 j stone dark
    [52, 48, 48, 255],    // 21 k stone deep
    [88, 100, 72, 255],   // 22 l floor a
    [76, 88, 64, 255],    // 23 m floor b
    [140, 136, 128, 255], // 24 n wall top
    [255, 120, 120, 255], // 25 o red light
    [224, 56, 56, 255],   // 26 p red
    [168, 40, 40, 255],   // 27 q red mid
    [112, 24, 32, 255],   // 28 r red dark
    [80, 16, 24, 255],    // 29 s blood
    [72, 48, 40, 255],    // 30 t boot
    [160, 200, 255, 255], // 31 u blue light
    [64, 144, 240, 255],  // 32 v blue
    [40, 96, 176, 255],   // 33 w blue mid
    [48, 192, 168, 255],  // 34 x teal
    [24, 128, 112, 255],  // 35 y teal dark
    [32, 96, 120, 255],   // 36 z water
    [255, 232, 120, 255], // 37 A gold light
    [232, 184, 48, 255],  // 38 B gold
    [176, 128, 24, 255],  // 39 C gold dark
    [255, 248, 160, 255], // 40 D yellow
    [255, 255, 255, 255], // 41 E white
    [24, 20, 28, 255],    // 42 F outline
    [40, 36, 48, 255],    // 43 G shadow
    [48, 140, 72, 255],   // 44 H slime dark
    [176, 232, 160, 255], // 45 I slime highlight
    [72, 56, 96, 255],    // 46 J bat
    [120, 88, 152, 255],  // 47 K bat wing
    [200, 96, 64, 255],   // 48 L octorok
    [144, 56, 40, 255],   // 49 M octorok dark
    [168, 140, 112, 255], // 50 N rock
    [112, 88, 64, 255],   // 51 O rock dark
    [152, 88, 176, 255],  // 52 P purple
    [32, 36, 48, 255],    // 53 Q ui panel
    [72, 80, 104, 255],   // 54 R ui edge
    [48, 32, 28, 255],    // 55 S boot dark
    [40, 96, 160, 255],   // 56 T cape / shield
    [248, 248, 255, 255], // 57 U eye white
    [16, 12, 20, 255],    // 58 V eye dark
];

/// Remap selected palette indices when decoding a strip (enemy tints, variants).
#[derive(Clone, Copy, Debug)]
pub struct PaletteSwap {
    pub from: &'static [u8],
    pub to: &'static [u8],
}

pub fn rgba(idx: u8) -> [u8; 4] {
    PALETTE
        .get(idx as usize)
        .copied()
        .unwrap_or([255, 0, 255, 255])
}

pub fn char_to_index(c: char) -> Option<u8> {
    match c {
        '.' => Some(0),
        '0'..='9' => Some(1 + (c as u8 - b'0')),
        'a'..='z' => Some(11 + (c as u8 - b'a')),
        'A'..='Z' => Some(37 + (c as u8 - b'A')),
        _ => None,
    }
}

pub fn index_to_char(idx: u8) -> char {
    match idx {
        0 => '.',
        1..=10 => (b'0' + (idx - 1)) as char,
        11..=36 => (b'a' + (idx - 11)) as char,
        37..=62 => (b'A' + (idx - 37)) as char,
        _ => '?',
    }
}

pub fn apply_swap(idx: u8, swap: Option<&PaletteSwap>) -> u8 {
    let Some(swap) = swap else {
        return idx;
    };
    for (i, &from) in swap.from.iter().enumerate() {
        if from == idx {
            if let Some(&to) = swap.to.get(i) {
                return to;
            }
        }
    }
    idx
}

/// Angry / low-HP slime: green ramp → red ramp.
pub const SLIME_ANGRY: PaletteSwap = PaletteSwap {
    from: &[SLIME_HIGHLIGHT, SLIME, SLIME_DARK],
    to: &[RED_LIGHT, RED, RED_DARK],
};

/// Gray dummy tint of slime sheet.
pub const SLIME_DUMMY: PaletteSwap = PaletteSwap {
    from: &[SLIME_HIGHLIGHT, SLIME, SLIME_DARK],
    to: &[STONE_LIGHT, STONE_MID, STONE_DEEP],
};

/// Rock projectile warm tint.
pub const ROCK_WARM: PaletteSwap = PaletteSwap {
    from: &[ROCK, ROCK_DARK],
    to: &[GOLD_DARK, STONE_DARK],
};
