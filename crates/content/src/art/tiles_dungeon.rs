//! Dungeon cool-stone + teal tile art (16×16).
use super::SpriteDef;

pub static D_FLOOR_A: SpriteDef = SpriteDef {
    name: "d_floor_a",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
    "gjjjjjjgjjjjjjgj\n",
    "jhhhhhghhhhhhghj\n",
    "jhhhhghhhhhhghhj\n",
    "jhhhghhhhhhghhhj\n",
    "jhhghhhhhhghhhhj\n",
    "jhghhhhhhghhhhhj\n",
    "jghhhhhhghhhhhhg\n",
    "ghhhhhhghhhhhhgj\n",
    "jhhhhhghhhhhhghj\n",
    "jhhhhghhhhhhghhj\n",
    "jhhhghhhhhhghhhj\n",
    "jhhghhhhhhghhhhj\n",
    "jhghhhhhhghhhhhj\n",
    "jghhhhhhghhhhhhg\n",
    "ghhhhhhghhhhhhgj\n",
    "jjjjjjgjjjjjjgjj\n"
    ),
};

pub static D_FLOOR_B: SpriteDef = SpriteDef {
    name: "d_floor_b",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
    "hjjjjjjhjjjjjjhj\n",
    "jiiiiihiiiiiihij\n",
    "jiiiihiiiiiihiij\n",
    "jiiihiiiiiihiiij\n",
    "jiihiiiiiihiiiij\n",
    "jihiiiiiihiiiiij\n",
    "jhiiiiiihiiiiiih\n",
    "hiiiiiihiiiiiihj\n",
    "jiiiiihiiiiiihij\n",
    "jiiiihiiiiiihiij\n",
    "jiiihiiiiiihiiij\n",
    "jiihiiiiiihiiiij\n",
    "jihiiiiiihiiiiij\n",
    "jhiiiiiihiiiiiih\n",
    "hiiiiiihiiiiiihj\n",
    "jjjjjjhjjjjjjhjj\n"
    ),
};

pub static D_FLOOR_RUNE: SpriteDef = SpriteDef {
    name: "d_floor_rune",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhxhxhxhhhhhh\n",
    "hhhhhhxhxhxhhhhh\n",
    "hhhhhxhxhxhhhhhh\n",
    "hhhhhhxhxhxhhhhh\n",
    "hhhhhxhxhxhhhhhh\n",
    "hhhhhhxhxhxhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhh\n"
    ),
};

pub static D_WALL: SpriteDef = SpriteDef {
    name: "d_wall",
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

pub static D_WALL_TOP: SpriteDef = SpriteDef {
    name: "d_wall_top",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
    "kkkkkkkkkkkkkkkk\n",
    "kkkkkkkkkkkkkkkk\n",
    "kkkkkkkkkkkkkkkk\n",
    "kkkkkkkkkkkkkkkk\n",
    "kkkkkkkkkkkkkkkk\n",
    "kkkkkkkkkkkkkkkk\n",
    "kkkkkkkkkkkkkkkk\n",
    "kkkkkkkkkkkkkkkk\n",
    "kkkkkkkkkkkkkkkk\n",
    "kkkkkkkkkkkkkkkk\n",
    "kkkkkkkkkkkkkkkk\n",
    "kkkkkkkkkkkkkkkk\n",
    "kkkkkkkkkkkkkkkk\n",
    "kkkkkkkkkkkkkkkk\n",
    "kkkkkkkkkkkkkkkk\n",
    "kkkkkkkkkkkkkkkk\n"
    ),
};

pub static D_PIT: SpriteDef = SpriteDef {
    name: "d_pit",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
    "jjjjjjjjjjjjjjjj\n",
    "jjjjjjjjjjjjjjjj\n",
    "jjkkkkkkkkkkkkjj\n",
    "jjkkkkkkkkkkkkjj\n",
    "jjkkkkkkkkkkkkjj\n",
    "jjkkkkkkkkkkkkjj\n",
    "jjkkkkkkkkkkkkjj\n",
    "jjkkkkkkkkkkkkjj\n",
    "jjkkkkkkkkkkkkjj\n",
    "jjkkkkkkkkkkkkjj\n",
    "jjkkkkkkkkkkkkjj\n",
    "jjkkkkkkkkkkkkjj\n",
    "jjkkkkkkkkkkkkjj\n",
    "jjkkkkkkkkkkkkjj\n",
    "jjjjjjjjjjjjjjjj\n",
    "jjjjjjjjjjjjjjjj\n"
    ),
};

pub static D_WATER: SpriteDef = SpriteDef {
    name: "d_water",
    w: 16,
    h: 16,
    frames: 2,
    grid: concat!(
    "xyzvxyzvxyzvxyzvyzvxyzvxyzvxyzvx\n",
    "yzvxyzvxyzvxyzvxzvxyzvxyzvxyzvxy\n",
    "zvxyzvxyzvxyzvxyvxyzvxyzvxyzvxyz\n",
    "vxyzvxyzvxyzvxyzxyzvxyzvxyzvxyzv\n",
    "xyzvxyzvxyzvxyzvyzvxyzvxyzvxyzvx\n",
    "yzvxyzvxyzvxyzvxzvxyzvxyzvxyzvxy\n",
    "zvxyzvxyzvxyzvxyvxyzvxyzvxyzvxyz\n",
    "vxyzvxyzvxyzvxyzxyzvxyzvxyzvxyzv\n",
    "xyzvxyzvxyzvxyzvyzvxyzvxyzvxyzvx\n",
    "yzvxyzvxyzvxyzvxzvxyzvxyzvxyzvxy\n",
    "zvxyzvxyzvxyzvxyvxyzvxyzvxyzvxyz\n",
    "vxyzvxyzvxyzvxyzxyzvxyzvxyzvxyzv\n",
    "xyzvxyzvxyzvxyzvyzvxyzvxyzvxyzvx\n",
    "yzvxyzvxyzvxyzvxzvxyzvxyzvxyzvxy\n",
    "zvxyzvxyzvxyzvxyvxyzvxyzvxyzvxyz\n",
    "vxyzvxyzvxyzvxyzxyzvxyzvxyzvxyzv\n"
    ),
};

pub static D_WATER_EDGE: SpriteDef = SpriteDef {
    name: "d_water_edge",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
    "yjjjjjjyjjjjjjyj\n",
    "jxxxxxyxxxxxxyxj\n",
    "jxxxxyxxxxxxyxxj\n",
    "jxxxyxxxxxxyxxxj\n",
    "jxxyxxxxxxyxxxxj\n",
    "jxyxxxxxxyxxxxxj\n",
    "jyxxxxxxyxxxxxxy\n",
    "yxxxxxxyxxxxxxyj\n",
    "jxxxxxyxxxxxxyxj\n",
    "jxxxxyxxxxxxyxxj\n",
    "jxxxyxxxxxxyxxxj\n",
    "jxxyxxxxxxyxxxxj\n",
    "jxyxxxxxxyxxxxxj\n",
    "jyxxxxxxyxxxxxxy\n",
    "yxxxxxxyxxxxxxyj\n",
    "jjjjjjyjjjjjjyjj\n"
    ),
};

pub static D_STAIRS: SpriteDef = SpriteDef {
    name: "d_stairs",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
    "xjjjjjjxjjjjjjxj\n",
    "jgggggxggggggxgj\n",
    "jggggxggggggxggj\n",
    "jgggxggggggxgggj\n",
    "jggxggggggxggggj\n",
    "jgxggggggxgggggj\n",
    "jxggggggxggggggx\n",
    "xggggggxggggggxj\n",
    "jgggggxggggggxgj\n",
    "jggggxggggggxggj\n",
    "jgggxggggggxgggj\n",
    "jggxggggggxggggj\n",
    "jgxggggggxgggggj\n",
    "jxggggggxggggggx\n",
    "xggggggxggggggxj\n",
    "jjjjjjxjjjjjjxjj\n"
    ),
};

pub static D_DOOR_OPEN: SpriteDef = SpriteDef {
    name: "d_door_open",
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

pub static D_DOOR_LOCKED: SpriteDef = SpriteDef {
    name: "d_door_locked",
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

pub static D_DOOR_BOSS: SpriteDef = SpriteDef {
    name: "d_door_boss",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
    "BBBBBBBBBBBBBBBB\n",
    "CCCCCCCCCCCCCCCC\n",
    "CCCCCCCCCCCCCCCC\n",
    "BBBBBBBBBBBBBBBB\n",
    "CCCCCCCCCCCCCCCC\n",
    "CCCCCCCCCCCCCCCC\n",
    "BBBBBBBBBBBBBBBB\n",
    "CCCCCCCCCCCCCCCC\n",
    "CCCCCCCCCCCCCCCC\n",
    "BBBBBBBBBBBBBBBB\n",
    "CCCCCCCCCCCCCCCC\n",
    "CCCCCCCCCCCCCCCC\n",
    "BBBBBBBBBBBBBBBB\n",
    "CCCCCCCCCCCCCCCC\n",
    "CCCCCCCCCCCCCCCC\n",
    "BBBBBBBBBBBBBBBB\n"
    ),
};

pub static D_SHUTTER: SpriteDef = SpriteDef {
    name: "d_shutter",
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

pub static D_LIFT: SpriteDef = SpriteDef {
    name: "d_lift",
    w: 16,
    h: 16,
    frames: 2,
    grid: concat!(
    "hhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh\n",
    "hhhhxxxxxxxxhhhhhhhhuuuuuuuuhhhh\n",
    "hhhhxxxxxxxxhhhhhhhhuuuuuuuuhhhh\n",
    "hhhhxxxxxxxxhhhhhhhhuuuuuuuuhhhh\n",
    "hhhhxxxxxxxxhhhhhhhhuuuuuuuuhhhh\n",
    "hhhhxxxxxxxxhhhhhhhhuuuuuuuuhhhh\n",
    "hhhhxxxxxxxxhhhhhhhhuuuuuuuuhhhh\n",
    "hhhhxxxxxxxxhhhhhhhhuuuuuuuuhhhh\n",
    "hhhhxxxxxxxxhhhhhhhhuuuuuuuuhhhh\n",
    "hhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh\n",
    "hhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh\n"
    ),
};

pub static D_CRYSTAL_BLUE: SpriteDef = SpriteDef {
    name: "d_crystal_blue",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
    "................\n",
    "................\n",
    "................\n",
    ".......EE.......\n",
    "......EuuE......\n",
    ".....EuuuuE.....\n",
    "....EuuuuuuE....\n",
    "...EuuuuuuuuE...\n",
    "...EuuuuuuuuE...\n",
    "....EuuuuuuE....\n",
    ".....EuuuuE.....\n",
    "......EuuE......\n",
    ".......EE.......\n",
    "................\n",
    "................\n",
    "................\n"
    ),
};

pub static D_CRYSTAL_AMBER: SpriteDef = SpriteDef {
    name: "d_crystal_amber",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
    "................\n",
    "................\n",
    "................\n",
    ".......EE.......\n",
    "......EDDE......\n",
    ".....EDDDDE.....\n",
    "....EDDDDDDE....\n",
    "...EDDDDDDDDE...\n",
    "...EDDDDDDDDE...\n",
    "....EDDDDDDE....\n",
    ".....EDDDDE.....\n",
    "......EDDE......\n",
    ".......EE.......\n",
    "................\n",
    "................\n",
    "................\n"
    ),
};

pub static D_GATE_BLUE_UP: SpriteDef = SpriteDef {
    name: "d_gate_blue_up",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
    "vvvvvvvvvvvvvvvv\n",
    "vvvjjjjjjjjjjvvv\n",
    "vvvvvvvvvvvvvvvv\n",
    "vvvjjjjjjjjjjvvv\n",
    "vvvvvvvvvvvvvvvv\n",
    "vvvjjjjjjjjjjvvv\n",
    "vvvvvvvvvvvvvvvv\n",
    "vvvjjjjjjjjjjvvv\n",
    "vvvvvvvvvvvvvvvv\n",
    "vvvjjjjjjjjjjvvv\n",
    "vvvvvvvvvvvvvvvv\n",
    "vvvjjjjjjjjjjvvv\n",
    "vvvvvvvvvvvvvvvv\n",
    "vvvjjjjjjjjjjvvv\n",
    "vvvvvvvvvvvvvvvv\n",
    "vvvjjjjjjjjjjvvv\n"
    ),
};

pub static D_GATE_BLUE_DOWN: SpriteDef = SpriteDef {
    name: "d_gate_blue_down",
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

pub static D_GATE_AMBER_UP: SpriteDef = SpriteDef {
    name: "d_gate_amber_up",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
    "DDDDDDDDDDDDDDDD\n",
    "DDDjjjjjjjjjjDDD\n",
    "DDDDDDDDDDDDDDDD\n",
    "DDDjjjjjjjjjjDDD\n",
    "DDDDDDDDDDDDDDDD\n",
    "DDDjjjjjjjjjjDDD\n",
    "DDDDDDDDDDDDDDDD\n",
    "DDDjjjjjjjjjjDDD\n",
    "DDDDDDDDDDDDDDDD\n",
    "DDDjjjjjjjjjjDDD\n",
    "DDDDDDDDDDDDDDDD\n",
    "DDDjjjjjjjjjjDDD\n",
    "DDDDDDDDDDDDDDDD\n",
    "DDDjjjjjjjjjjDDD\n",
    "DDDDDDDDDDDDDDDD\n",
    "DDDjjjjjjjjjjDDD\n"
    ),
};

pub static D_GATE_AMBER_DOWN: SpriteDef = SpriteDef {
    name: "d_gate_amber_down",
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
