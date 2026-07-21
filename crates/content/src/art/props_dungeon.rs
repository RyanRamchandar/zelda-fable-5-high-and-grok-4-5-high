//! Dungeon props (torches, runes, seals).
use super::SpriteDef;

pub static D_TORCH_LIT: SpriteDef = SpriteDef {
    name: "d_torch_lit",
    w: 16,
    h: 16,
    frames: 2,
    grid: concat!(
    "................................\n",
    "................................\n",
    ".....DoDoDo..........oDoDoD.....\n",
    ".....oDoDoD..........DoDoDo.....\n",
    ".....DoDoDo..........oDoDoD.....\n",
    ".....oDoDoD..........DoDoDo.....\n",
    ".....DoDoDo..........oDoDoD.....\n",
    ".....oDoDoD..........DoDoDo.....\n",
    "......jjjj............jjjj......\n",
    "......jjjj............jjjj......\n",
    "......jjjj............jjjj......\n",
    "......jjjj............jjjj......\n",
    "......jjjj............jjjj......\n",
    "......jjjj............jjjj......\n",
    "......jjjj............jjjj......\n",
    "................................\n"
    ),
};

pub static D_TORCH_UNLIT: SpriteDef = SpriteDef {
    name: "d_torch_unlit",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
    "................\n",
    "................\n",
    "................\n",
    "................\n",
    "................\n",
    "................\n",
    "................\n",
    "................\n",
    "......jjjj......\n",
    "......jjjj......\n",
    "......jjjj......\n",
    "......jjjj......\n",
    "......jjjj......\n",
    "......jjjj......\n",
    "......jjjj......\n",
    "................\n"
    ),
};

pub static D_BRAZIER_ETERNAL: SpriteDef = SpriteDef {
    name: "d_brazier_eternal",
    w: 16,
    h: 16,
    frames: 2,
    grid: concat!(
    "................................\n",
    "................................\n",
    "................................\n",
    ".....DoDoDo..........oDoDoD.....\n",
    ".....oDoDoD..........DoDoDo.....\n",
    ".....DoDoDo..........oDoDoD.....\n",
    ".....oDoDoD..........DoDoDo.....\n",
    ".....DoDoDo..........oDoDoD.....\n",
    ".....oDoDoD..........DoDoDo.....\n",
    ".....DoDoDo..........oDoDoD.....\n",
    "....jjjjjjjj........jjjjjjjj....\n",
    "....jjjjjjjj........jjjjjjjj....\n",
    "....jjjjjjjj........jjjjjjjj....\n",
    "....jjjjjjjj........jjjjjjjj....\n",
    "....jjjjjjjj........jjjjjjjj....\n",
    "................................\n"
    ),
};

pub static D_RUNE_1: SpriteDef = SpriteDef {
    name: "d_rune_1",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
    "................\n",
    "................\n",
    "................\n",
    "...ggxggxggxg...\n",
    "...gxggxggxgg...\n",
    "...xggxggxggx...\n",
    "...ggxggxggxg...\n",
    "...gxggxggxgg...\n",
    "...xggxggxggx...\n",
    "...ggxggxggxg...\n",
    "...gxggxggxgg...\n",
    "...xggxggxggx...\n",
    "...ggxggxggxg...\n",
    "................\n",
    "................\n",
    "................\n"
    ),
};

pub static D_RUNE_2: SpriteDef = SpriteDef {
    name: "d_rune_2",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
    "................\n",
    "................\n",
    "................\n",
    "...gxggxggxgg...\n",
    "...xggxggxggx...\n",
    "...ggxggxggxg...\n",
    "...gxggxggxgg...\n",
    "...xggxggxggx...\n",
    "...ggxggxggxg...\n",
    "...gxggxggxgg...\n",
    "...xggxggxggx...\n",
    "...ggxggxggxg...\n",
    "...gxggxggxgg...\n",
    "................\n",
    "................\n",
    "................\n"
    ),
};

pub static D_RUNE_3: SpriteDef = SpriteDef {
    name: "d_rune_3",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
    "................\n",
    "................\n",
    "................\n",
    "...xggxggxggx...\n",
    "...ggxggxggxg...\n",
    "...gxggxggxgg...\n",
    "...xggxggxggx...\n",
    "...ggxggxggxg...\n",
    "...gxggxggxgg...\n",
    "...xggxggxggx...\n",
    "...ggxggxggxg...\n",
    "...gxggxggxgg...\n",
    "...xggxggxggx...\n",
    "................\n",
    "................\n",
    "................\n"
    ),
};

pub static D_SEAL_DOOR: SpriteDef = SpriteDef {
    name: "d_seal_door",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
    "jjjjjjjjjjjjjjjj\n",
    "jjjjjjjjjjjjjjjj\n",
    "jjjjjjjjjjjjjjjj\n",
    "jjjjjjjjjjjjjjjj\n",
    "jjjjjjjjjjjjjjjj\n",
    "jjjjjjjjjjjjjjjj\n",
    "jjjjjjjjjjjjjjjj\n",
    "jjjjjjjjjjjjjjjj\n",
    "jjjjjjjjjjjjjjjj\n",
    "jjjjjjjjjjjjjjjj\n",
    "jjjjjjjjjjjjjjjj\n",
    "jjjjjjjjjjjjjjjj\n",
    "jjjjjjjjjjjjjjjj\n",
    "jjjjjjjjjjjjjjjj\n",
    "jjjjjjjjjjjjjjjj\n",
    "jjjjjjjjjjjjjjjj\n"
    ),
};

pub static D_SEAL_BROKEN: SpriteDef = SpriteDef {
    name: "d_seal_broken",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n"
    ),
};
