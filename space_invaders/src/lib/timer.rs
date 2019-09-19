use std::time::{SystemTime, UNIX_EPOCH};

pub struct Timer {
    last_trigger: usize,
    last_check: usize,
    interval: f64,
}

impl Timer {
    pub(crate) fn new(interval: f64) -> Timer {
        let ms = Timer::get_millis();
        Timer {
            last_check: ms,
            last_trigger: ms,
            interval,
        }
    }

    pub(crate) fn reset(&mut self) {
        let ms = Timer::get_millis();
        self.last_check = ms;
        self.last_trigger = ms;
    }

    pub fn update_last_check(&mut self) -> usize {
        let new_time = Timer::get_millis();
        let elapsed = new_time - self.last_check;
        self.last_check = new_time;
        elapsed
    }

    pub(crate) fn should_trigger(&mut self) -> bool {
        let ms = Timer::get_millis();
        let should = (ms as f64 - self.last_trigger as f64) > self.interval;
        if should {
            self.last_trigger = ms;
        }
        should
    }

    pub fn reset_preserving_intervals(&mut self) {
        let new_time = Timer::get_millis();
        self.last_trigger = new_time - (self.last_check - self.last_trigger);
        self.last_check = new_time;
    }

    fn get_millis() -> usize {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        (since_the_epoch.as_secs() * 1000) as usize
            + since_the_epoch.subsec_nanos() as usize / 1_000_000
    }
}
