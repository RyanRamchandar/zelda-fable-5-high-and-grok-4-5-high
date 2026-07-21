//! Player energy meter.

use crate::combat::style;
use crate::combat::tuning;
use crate::world::entity::PlayerData;

pub fn refund(pd: &mut PlayerData, amount: f32) {
    pd.energy = (pd.energy + amount).min(tuning::ENERGY_MAX);
}

pub fn tick(pd: &mut PlayerData) {
    if pd.energy_deny_flash > 0 {
        pd.energy_deny_flash -= 1;
    }
    pd.ticks_since_spend = pd.ticks_since_spend.saturating_add(1);
    if pd.ticks_since_spend >= tuning::ENERGY_REGEN_DELAY && pd.energy < tuning::ENERGY_MAX {
        let rate = tuning::ENERGY_REGEN_PER_TICK * style::energy_regen_mult(pd);
        pd.energy = (pd.energy + rate).min(tuning::ENERGY_MAX);
    }
}

pub fn fill(pd: &mut PlayerData, amount: f32) {
    pd.energy = (pd.energy + amount).min(tuning::ENERGY_MAX);
}
