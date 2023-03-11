use crate::{
    arch::timer::{get_time, set_timer},
    config::{CLOCK_FREQ, INTR_PER_SEC},
};

pub fn set_next_trigger() {
    set_timer((get_time() + CLOCK_FREQ / INTR_PER_SEC).try_into().unwrap());
}
