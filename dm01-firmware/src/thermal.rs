use crate::config::MotorConfig;

/// Thermal protection state machine.
/// Monitors controller temperature and enforces the documented 30% rollback above MaxTemp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThermalMonitor {
    pub is_throttling: bool,
}

impl ThermalMonitor {
    pub const fn new() -> Self {
        Self {
            is_throttling: false,
        }
    }

    /// Returns a current scaling factor in permille (300..=1000) based on controller temperature.
    pub fn update_scale_permille(&mut self, controller_temp_c: i16, config: &MotorConfig) -> u16 {
        if controller_temp_c >= config.temp_max_c {
            self.is_throttling = true;
            // Above MaxTemp (default 100C), roll back output power to 30%
            return 300;
        }

        // Hysteresis recovery: clear hard thermal rollback once temp drops 3C below MaxTemp
        if self.is_throttling {
            if controller_temp_c <= config.temp_max_c.saturating_sub(3) {
                self.is_throttling = false;
            } else {
                return 300;
            }
        }

        // Progressive linear derating between MinTemp (70C) and MaxTemp (100C) if desired,
        // tapering from 100% (1000) at MinTemp down to 70% (700) just before MaxTemp cutoff.
        if controller_temp_c > config.temp_min_c && config.temp_max_c > config.temp_min_c {
            let range = (config.temp_max_c - config.temp_min_c) as i32;
            let excess = (controller_temp_c - config.temp_min_c) as i32;
            let reduction = (excess * 300) / range;
            return (1000 - reduction).clamp(300, 1000) as u16;
        }

        1000
    }
}

