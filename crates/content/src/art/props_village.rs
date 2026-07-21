//! Village props + POI icons (Phase 2B).

use super::SpriteDef;

pub static PROP_LANTERN: SpriteDef = SpriteDef {
    name: "prop_lantern",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
        "....FF..........\n",
        "...FBBF.........\n",
        "...FBDF.........\n",
        "...FBBF.........\n",
        "....ZZ..........\n",
        "....ZZ..........\n",
        "....FF..........\n",
        "................\n",
        "....FF..........\n",
        "...FBBF.........\n",
        "...FAAF.........\n",
        "...FBBF.........\n",
        "....ZZ..........\n",
        "....ZZ..........\n",
        "....FF..........\n",
        "................\n",
    ),
};

pub static PROP_FLOWER_BED: SpriteDef = SpriteDef {
    name: "prop_flower_bed",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
        "................\n",
        "..op..op..op....\n",
        ".4pp4.4pp4.4pp..\n",
        "..55...55...55..\n",
        "................\n",
        "..op.....op.....\n",
        ".4pp4...4pp4....\n",
        "..55.....55.....\n",
        "................\n",
        ".....op.........\n",
        "....4pp4........\n",
        ".....55.........\n",
        "................\n",
        "................\n",
        "................\n",
        "................\n",
    ),
};

pub static PROP_STALL: SpriteDef = SpriteDef {
    name: "prop_stall",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
        "..FFFFFFFFFFF...\n",
        ".FpppppppppppF..\n",
        ".FpppppppppppF..\n",
        ".FZ.......Z..F..\n",
        ".FZ.......Z..F..\n",
        ".FZZZZZZZZZZZF..\n",
        ".FZ.......Z..F..\n",
        ".FZ.......Z..F..\n",
        "..F.......F.....\n",
        "..F.......F.....\n",
        "..F.......F.....\n",
        "................\n",
        "................\n",
        "................\n",
        "................\n",
        "................\n",
    ),
};

pub static PROP_BASIN: SpriteDef = SpriteDef {
    name: "prop_basin",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
        "................\n",
        "...gggggggg.....\n",
        "..gzzzzzzzzg....\n",
        ".gz........zg...\n",
        ".gz.uuuuuu.zg...\n",
        ".gz.uvvvvu.zg...\n",
        ".gz.uvvvvu.zg...\n",
        ".gz.uuuuuu.zg...\n",
        ".gz........zg...\n",
        "..gzzzzzzzzg....\n",
        "...gggggggg.....\n",
        "................\n",
        "................\n",
        "................\n",
        "................\n",
        "................\n",
    ),
};

pub static PROP_SIGN: SpriteDef = SpriteDef {
    name: "prop_sign",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
        "................\n",
        "..FFFFFFFF......\n",
        ".FYYYYYYYYF.....\n",
        ".FYFFFFFFYF.....\n",
        ".FYFFFFFFYF.....\n",
        ".FYYYYYYYYF.....\n",
        "..FFFFFFFF......\n",
        "....ZZ..........\n",
        "....ZZ..........\n",
        "....ZZ..........\n",
        "....ZZ..........\n",
        "................\n",
        "................\n",
        "................\n",
        "................\n",
        "................\n",
    ),
};

pub static PROP_CHEST: SpriteDef = SpriteDef {
    name: "prop_chest",
    w: 16,
    h: 16,
    frames: 2,
    grid: concat!(
        "................................\n",
        "..FFFFFFFF........FFFFFFFF......\n",
        ".FBBBBBBBBF......FBB....BBF.....\n",
        ".FBBBBBBBBF......FB......BF.....\n",
        ".FBBBFFBBBF......FBBBFFBBBF.....\n",
        ".FBBBBBBBBF......FBBBBBBBBF.....\n",
        ".FCCCCCCCCF......FCCCCCCCCF.....\n",
        ".FCCCCCCCCF......FCCCCCCCCF.....\n",
        "..FFFFFFFF........FFFFFFFF......\n",
        "................................\n",
        "................................\n",
        "................................\n",
        "................................\n",
        "................................\n",
        "................................\n",
        "................................\n",
    ),
};

pub static PROP_GEM: SpriteDef = SpriteDef {
    name: "prop_gem",
    w: 16,
    h: 16,
    frames: 2,
    grid: concat!(
        "................................\n",
        "......EE..............EE........\n",
        ".....Euue............EAAe.......\n",
        "....Euvvue..........EAAAae......\n",
        "...Euvvvvue........EAAAAAae.....\n",
        "....Euvvue..........EAAAae......\n",
        ".....Euue............EAAe.......\n",
        "......EE..............EE........\n",
        "......hh..............hh........\n",
        ".....hhhh............hhhh.......\n",
        "....hhhhhh..........hhhhhh......\n",
        "................................\n",
        "................................\n",
        "................................\n",
        "................................\n",
        "................................\n",
    ),
};

pub static PROP_PEDESTAL: SpriteDef = SpriteDef {
    name: "prop_pedestal",
    w: 16,
    h: 16,
    frames: 1,
    grid: concat!(
        "................\n",
        "....gggggg......\n",
        "...ghhhhhhg.....\n",
        "...gh....hg.....\n",
        "...ghhhhhhg.....\n",
        "....gjjjjg......\n",
        ".....jjjj.......\n",
        ".....jjjj.......\n",
        "....jjjjjj......\n",
        "...jjjjjjjj.....\n",
        "................\n",
        "................\n",
        "................\n",
        "................\n",
        "................\n",
        "................\n",
    ),
};

pub static POI_SHOP: SpriteDef = SpriteDef {
    name: "poi_shop",
    w: 6,
    h: 6,
    frames: 1,
    grid: concat!(".FFFF.\n", "FBBBBF\n", "FB..BF\n", "FBBBBF\n", ".FFFF.\n", "......\n",),
};

pub static POI_FOUNTAIN: SpriteDef = SpriteDef {
    name: "poi_fountain",
    w: 6,
    h: 6,
    frames: 1,
    grid: concat!("..uu..\n", ".uvvu.\n", "uvvvvu\n", ".uvvu.\n", "..uu..\n", "......\n",),
};

pub static POI_SHRINE: SpriteDef = SpriteDef {
    name: "poi_shrine",
    w: 6,
    h: 6,
    frames: 1,
    grid: concat!(".AAAA.\n", "A....A\n", "A.EE.A\n", "A....A\n", ".AAAA.\n", "......\n",),
};

pub static POI_GEM: SpriteDef = SpriteDef {
    name: "poi_gem",
    w: 6,
    h: 6,
    frames: 1,
    grid: concat!("..EE..\n", ".Euue.\n", "Euvvue\n", ".Euue.\n", "..EE..\n", "......\n",),
};

pub static POI_CHECK: SpriteDef = SpriteDef {
    name: "poi_check",
    w: 6,
    h: 6,
    frames: 1,
    grid: concat!("......\n", "....4.\n", "...44.\n", "4.44..\n", ".44...\n", "......\n",),
};

pub static POI_STAR: SpriteDef = SpriteDef {
    name: "poi_star",
    w: 6,
    h: 6,
    frames: 1,
    grid: concat!("..D...\n", ".DDD..\n", "DDDDDD\n", ".DDD..\n", "D...D.\n", "......\n",),
};

pub static POI_SECRET: SpriteDef = SpriteDef {
    name: "poi_secret",
    w: 6,
    h: 6,
    frames: 1,
    grid: concat!("..PP..\n", ".P..P.\n", "P....P\n", ".P..P.\n", "..PP..\n", "......\n",),
};
