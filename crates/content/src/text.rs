//! All player-facing dialogue / sign copy. No string literals in game code.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextId {
    VillageWelcome,
    ShopSign,
    WaypostShrine,
    FountainLore,
    ElderIntro,
    ElderReminder,
    ShopkeeperStub,
    VillagerA,
    VillagerB,
    VillagerC,
    KidHint,
    ChimeSign,
    RuinsTablet,
    ShrineLore,
    SealHolds0,
    SealHolds1,
    SealHolds2,
    SealOpens,
    ShrineLobby,
    HollowWall,
    SummitVista,
    WatchtowerView,
    TwinFlames,
    PowerChestLocked,
    CourageGemHold,
    PowerGemHold,
    WisdomGemHold,
    SecretFound,
    MeadowFairy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpcId {
    Elder,
    Shopkeeper,
    VillagerA,
    VillagerB,
    VillagerC,
    Kid,
}

pub fn npc_lines(npc: NpcId) -> TextId {
    match npc {
        NpcId::Elder => TextId::ElderIntro,
        NpcId::Shopkeeper => TextId::ShopkeeperStub,
        NpcId::VillagerA => TextId::VillagerA,
        NpcId::VillagerB => TextId::VillagerB,
        NpcId::VillagerC => TextId::VillagerC,
        NpcId::Kid => TextId::KidHint,
    }
}

pub fn npc_sprite(npc: NpcId) -> &'static str {
    match npc {
        NpcId::Elder => "npc_elder",
        NpcId::Shopkeeper => "npc_shop",
        NpcId::VillagerA => "npc_villager_a",
        NpcId::VillagerB => "npc_villager_b",
        NpcId::VillagerC => "npc_villager_c",
        NpcId::Kid => "npc_kid",
    }
}

/// Pages of ≤3 lines × ~38 chars.
pub fn text(id: TextId) -> &'static [&'static str] {
    match id {
        TextId::VillageWelcome => &[
            "Welcome to Mosslight Village.",
            "Rest by the fountain—danger waits",
            "beyond the hedges.",
        ],
        TextId::ShopSign => &[
            "Mosslight Goods",
            "Open when stock arrives.",
        ],
        TextId::WaypostShrine => &[
            "NORTH — Triforce Shrine",
            "Three gems open the seal.",
        ],
        TextId::FountainLore => &[
            "The fountain remembers kindness.",
            "Fairies mend what blades cannot.",
        ],
        TextId::ElderIntro => &[
            "Traveler—listen well.",
            "Recover the three virtue gems:",
            "Courage, Power, and Wisdom.",
            "Only then will the shrine open.",
        ],
        TextId::ElderReminder => &[
            "The gems still wait.",
            "Grove, camp, ruins—then shrine.",
        ],
        TextId::ShopkeeperStub => &[
            "Stock arrives soon, friend.",
            "Come back when the shelves fill.",
        ],
        TextId::VillagerA => &[
            "Behind the shop hedge, coin glints",
            "if you watch your step.",
        ],
        TextId::VillagerB => &[
            "The river hides a path when you",
            "look down from the broken bridge.",
        ],
        TextId::VillagerC => &[
            "In the grove, a lonely pale tree",
            "hides a glade of soft light.",
        ],
        TextId::KidHint => &[
            "There's a wall that sounds hollow!",
            "Also—stand in the flower ring",
            "in the meadow and wait…",
        ],
        TextId::ChimeSign => &[
            "The chimes answer a single gale…",
            "(puzzle sealed—return later)",
        ],
        TextId::RuinsTablet => &[
            "Three gems, three virtues…",
            "Courage. Power. Wisdom.",
            "Together they unbind the seal.",
        ],
        TextId::ShrineLore => &[
            "The seal holds until three gems",
            "rest upon their pedestals once",
            "more—in spirit, if not in stone.",
        ],
        TextId::SealHolds0 => &["The seal holds… 0 of 3 gems."],
        TextId::SealHolds1 => &["The seal holds… 1 of 3 gems."],
        TextId::SealHolds2 => &["The seal holds… 2 of 3 gems."],
        TextId::SealOpens => &[
            "The seal yields!",
            "The way into the lobby opens.",
        ],
        TextId::ShrineLobby => &[
            "The dungeon lies beyond.",
            "(Act 1 dungeon — Phase 3)",
        ],
        TextId::HollowWall => &["It sounds hollow…"],
        TextId::SummitVista => &[
            "The shrine watches over the valley.",
            "A scenic reward clinks nearby.",
        ],
        TextId::WatchtowerView => &[
            "From the tower, the camp sprawls.",
            "Raiders keep a war-chest guarded.",
        ],
        TextId::TwinFlames => &[
            "Twin flames answer together.",
            "Light both braziers as one.",
        ],
        TextId::PowerChestLocked => &[
            "Guards still linger nearby.",
            "Clear them to claim the prize.",
        ],
        TextId::CourageGemHold => &["You hold aloft the Courage Gem!"],
        TextId::PowerGemHold => &["You hold aloft the Power Gem!"],
        TextId::WisdomGemHold => &["You hold aloft the Wisdom Gem!"],
        TextId::SecretFound => &["A secret discovered!"],
        TextId::MeadowFairy => &[
            "A fairy answers the flower ring!",
            "Energy floods your spirit.",
        ],
    }
}
