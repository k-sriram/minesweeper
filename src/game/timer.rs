use std::time::{Duration, Instant};

pub struct Timer {
    start: Option<Instant>,
    excess: Duration,
}

impl Timer {
    pub const fn new() -> Self {
        Timer {
            start: None,
            excess: Duration::ZERO,
        }
    }

    pub fn start(&mut self) {
        self.excess = Duration::ZERO;
        self.resume();
    }

    pub fn stop(&mut self) {
        if let Some(start) = self.start {
            self.excess += start.elapsed();
            self.start = None;
        }
    }

    pub fn reset(&mut self) {
        self.excess = Duration::ZERO;
        self.start = None;
    }

    pub fn resume(&mut self) {
        self.start = Some(Instant::now());
    }

    pub fn time(&self) -> Duration {
        self.start.map_or(Duration::ZERO, |start| start.elapsed()) + self.excess
    }

    pub fn time_f64(&self) -> f64 {
        self.time().as_secs_f64()
    }

    pub fn time_as_secs(&self) -> u64 {
        self.time().as_secs()
    }

    pub fn is_running(&self) -> bool {
        self.start.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer() {
        let mut timer = Timer::new();
        assert_eq!(timer.time(), Duration::ZERO);
        assert_eq!(timer.time_f64(), 0.0);
        assert_eq!(timer.time_as_secs(), 0);
        assert!(!timer.is_running());

        timer.start();
        assert!(timer.is_running());

        std::thread::sleep(Duration::from_micros(1));
        timer.stop();
        assert!(!timer.is_running());
        let time = timer.time();
        assert!(time > Duration::ZERO);
        std::thread::sleep(Duration::from_micros(1));
        assert_eq!(timer.time(), time);

        timer.resume();
        assert!(timer.is_running());
        std::thread::sleep(Duration::from_micros(1));
        assert!(timer.time() > time);

        timer.stop();
        assert!(!timer.is_running());

        timer.reset();
        assert!(!timer.is_running());
        assert_eq!(timer.time(), Duration::ZERO);
        assert_eq!(timer.time_f64(), 0.0);
        assert_eq!(timer.time_as_secs(), 0);
    }
}
