//! Enemy 16×16 (+ rock 8×8) sheets.

use super::SpriteDef;


pub static SLIME: SpriteDef = SpriteDef {
    name: "slime",
    w: 16,
    h: 16,
    frames: 5,
    grid: concat!(
    "................................................................................\n",
    "................................................................................\n",
    ".....................................FFFFFF.....................................\n",
    "....................................FIIIIIIF....................................\n",
    "...................................F99999999F.......FFFFFFFF....................\n",
    "...................................F99999999F......FIIIIIIIIF...................\n",
    "....FFFFFFFF.......................F99U99U99F.....F9999999999F..................\n",
    "...FIIIIIIIIF.......FFFFFFFF.......F99V99V99F.....F9999999999F..................\n",
    "..F9999999999F.....FIIIIIIIIF......F99999999F.....F9999999999F.....FFFFFFFFFF...\n",
    "..F999U99U999F....F999U99U999F.....F9HHHHHH9F.....F999U99U999F....FIIIIIIIIIIF..\n",
    "..F999V99V999F...F9999V99V9999F....F9HHHHHH9F.....F999V99V999F...F9999U99U9999F.\n",
    "..F9999999999F...F999999999999F.....FFFFFFFF......F9HHHHHHHH9F..F99999V99V99999F\n",
    "..F9HHHHHHHH9F...F99HHHHHHHH99F...................F9HHHHHHHH9F..F99999999999999F\n",
    "..F9HHHHHHHH9F....F9HHHHHHHH9F.....................FFFFFFFFFF....F9HHHHHHHHHH9F.\n",
    "...FFFFFFFFFF......FFFFFFFFFF.....................................FFFFFFFFFFFF..\n",
    "................................................................................\n",
    ),
};

pub static BAT: SpriteDef = SpriteDef {
    name: "bat",
    w: 16,
    h: 16,
    frames: 4,
    grid: concat!(
    "................................................................\n",
    "................................................................\n",
    "................................................................\n",
    "................................................................\n",
    ".......FF.........FFFF.FF.FFFF.........FF..............FF.......\n",
    "..F...FJJF...F...FKKKKFJJFKKKKF.......FJJF......FFFFFFFJJFFFFFFF\n",
    ".FKFFFJJJJFFFKF..FKKKKJJJJKKKKF......FJJJJF.....KKKKKKJJJJKKKKKK\n",
    "FKKKKKJUpJKKKKKF.FKKKKJUpJKKKKF..FFFFFJUpJFFFFF.KKKKKKJUpJKKKKKK\n",
    "FKKKKKJJJJKKKKKF..FFFFJJJJFFFF..FKKKKKJJJJKKKKKFFFFFFFJJJJFFFFFF\n",
    ".FFFFFJJJJFFFFF......FJJJJF.....FKKKKKJJJJKKKKKF.....FJJJJF.....\n",
    ".....FJJJJF..........FJJJJF.....FKKKKKJJJJKKKKKF.....FJJJJF.....\n",
    "......FFFF............FFFF.......FFFFFFFFFFFFFF......FJJJJF.....\n",
    ".....................................................FJJJJF.....\n",
    "......................................................FFFF......\n",
    "................................................................\n",
    "................................................................\n",
    ),
};

pub static OCTOROK: SpriteDef = SpriteDef {
    name: "octorok",
    w: 16,
    h: 16,
    frames: 6,
    grid: concat!(
    "................................................................................................\n",
    ".......................................................................................FF.......\n",
    "......................................................................................FNNF......\n",
    ".....FFFFFF..........FFFFFF..........FFFFFF..........FFFFFF..........FFFFFF..........FFNNFF.....\n",
    "....FLLLLLLF........FLLLLLLF........FLLLLLLF........FLLLLLLF........FLLLLLLF........FLLNNLLF....\n",
    "...FLLLLLLLLF......FooooooooF......FLLLLLLLLF......FLLLLLLLLF......FLLLLLLLLF......FLLLNNLLLF...\n",
    "...FLLLLLLLLF......FLLLLLLLLF......FLLLLLLLLF......FLLLLLLLLF......FLLLLLLLLF......FLLLLLLLLF...\n",
    "...FLLULLULLF......FLLULLULLF......FLLLLLLLLF......FLLLLLLLLF......FLLULLULLF......FLLULLULLF...\n",
    "...FLLVLLVLLF......FLLVLLVLLF.....FMMLLLLLLMMF....FMMLLLLLLMMF.....FLLpLLpLLF......FLLpLLpLLF...\n",
    "...FLLLMMLLLF......FLLLMMLLLF.....FMMMMMMMMMMF....FMiiiiiiiiMF.....FLLLggLLLF......FLLLggLLLF...\n",
    "...FLLLMMLLLF......FLLLMMLLLF.....FMMMMMMMMMMF....FMiiiiiiiiMF.....FLLLggLLLF......FLLLggLLLF...\n",
    "...FLMMMMMMLF......FLMMMMMMLF.....FMMMMMMMMMMF....FMiiiiiiiiMF.....FLMMggMMLF......FLMMggMMLF...\n",
    "...FLMMMMMMLF......FLMMMMMMLF.....FMMMMMMMMMMF....FMiiiiiiiiMF.....FLMMMMMMLF......FLMMMMMMLF...\n",
    "...FMMFMMFMMF......FMMFMMFMMF.....FMMMMMMMMMMF....FMMMMMMMMMMF.....FMMFMMFMMF......FMMFMMFMMF...\n",
    "...FMMFMMFMMF......FMMFMMFMMF......FMMFMMFMMF......FMMFMMFMMF......FMMFMMFMMF......FMMFMMFMMF...\n",
    "....FF.FF.FF........FF.FF.FF........FF.FF.FF........FF.FF.FF........FF.FF.FF........FF.FF.FF....\n",
    ),
};

pub static OCTOROK_ROCK: SpriteDef = SpriteDef {
    name: "octorok_rock",
    w: 8,
    h: 8,
    frames: 2,
    grid: concat!(
    "................\n",
    "..FFFF....FFFF..\n",
    ".FONNNF..FONgNF.\n",
    ".FNgNNF..FNONNF.\n",
    ".FNNNNF..FNNNNF.\n",
    ".FNNNOF..FNNNOF.\n",
    "..FFFF....FFFF..\n",
    "................\n",
    ),
};
