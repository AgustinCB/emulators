use std::time::{SystemTime, UNIX_EPOCH};

pub struct Timer {
    last_trigger: usize,
    last_check: usize,
    interval: f64,
}

impl Timer {
    pub fn new(interval: f64) -> Timer {
        let ms = Timer::get_millis();
        Timer {
            interval,
            last_check: ms,
            last_trigger: ms,
        }
    }

    pub fn update_last_check(&mut self) {
        self.last_check =  Timer::get_millis();
    }

    pub fn should_trigger(&mut self) -> bool {
        let ms = Timer::get_millis();
        let should = (ms as f64 - self.last_trigger as f64) > self.interval;
        if should {
            self.last_trigger = ms;
        }
        should
    }

    fn get_millis() -> usize {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        (since_the_epoch.as_secs() * 1000) as usize +
            since_the_epoch.subsec_nanos() as usize / 1_000_000
    }
}