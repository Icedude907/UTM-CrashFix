use core::f64::consts::PI;

use smash::app::{self, lua_bind::*};
use smash::Vector2f;

use crate::common::consts::*;
use crate::common::*;
use crate::training::directional_influence;
use training_mod_sync::*;

static COUNTER: RwLock<u32> = RwLock::new(0);
static DIRECTION: RwLock<Direction> = RwLock::new(Direction::NEUTRAL);

// TODO! Bug - we only roll a new direction when loading a save state or on LRA reset
pub fn roll_direction() {
    assign(&COUNTER, 0);
    assign(&DIRECTION, read(&MENU).sdi_state.get_random());
}

unsafe fn get_sdi_direction() -> Option<f64> {
    let direction = read(&DIRECTION);
    direction.into_angle().map(|angle| {
        if directional_influence::should_reverse_angle(direction) {
            PI - angle
        } else {
            angle
        }
    })
}

#[skyline::hook(replace = FighterControlModuleImpl::check_hit_stop_delay_command)]
pub unsafe fn check_hit_stop_delay_command(
    module_accessor: &mut app::BattleObjectModuleAccessor,
    sdi_direction: *mut Vector2f,
) -> u64 {
    // Function returns 1 if there is an SDI input, 0 is there is not

    if !is_training_mode() || !is_operation_cpu(module_accessor) {
        return original!()(module_accessor, sdi_direction);
    }
    let repeat = read(&MENU).sdi_strength.into_u32();
    let mut counter_lock = lock_write(&COUNTER);
    *counter_lock = (*counter_lock + 1) % repeat;
    if *counter_lock == repeat - 1 {
        if let Some(angle) = get_sdi_direction() {
            // If there is a non-neutral direction picked,
            // modify the SDI angle Vector2f as a side-effect
            // and return 1 so the CPU knows that an SDI input occurred
            (*sdi_direction).x = angle.cos() as f32;
            (*sdi_direction).y = angle.sin() as f32;
            return 1;
        }
    }
    0
}
