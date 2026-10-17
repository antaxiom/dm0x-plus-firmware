use crate::config::{MotorConfig, OverTempMode};

/// Thermal protection monitor for both Controller (`CTL`) and Motor (`MOT`) temperature sensors
/// matching Page 14 of the T24 Display Manual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThermalMonitor {
    pub ctl_overheated: bool,
    pub mot_overheated: bool,
}

impl ThermalMonitor {
    pub const fn new() -> Self {
        Self {
            ctl_overheated: false,
            mot_overheated: false,
        }
    }

    /// Evaluates controller (`CTL`) and motor (`MOT`) temperatures and returns the active
    /// current limit scaling factor in permille (`300`, `500`, `700`, or `1000`).
    pub fn update_scale_permille(
        &mut self,
        controller_temp_c: i16,
        motor_temp_c: i16,
        config: &MotorConfig,
    ) -> u16 {
        let mut scale_permille = 1000u16;

        // 1. Controller temperature sensor (CTL)
        if config.temp_ctl_enabled {
            if controller_temp_c >= config.temp_ctl_max_c {
                self.ctl_overheated = true;
            } else if self.ctl_overheated && controller_temp_c <= config.temp_ctl_min_c {
                self.ctl_overheated = false;
            }

            if self.ctl_overheated {
                scale_permille = scale_permille.min(mode_to_permille(config.temp_ctl_over_mode));
            }
        } else {
            self.ctl_overheated = false;
        }

        // 2. Motor temperature sensor (MOT, optional hardware)
        if config.temp_mot_enabled {
            if motor_temp_c >= config.temp_mot_max_c {
                self.mot_overheated = true;
            } else if self.mot_overheated && motor_temp_c <= config.temp_mot_min_c {
                self.mot_overheated = false;
            }

            if self.mot_overheated {
                scale_permille = scale_permille.min(mode_to_permille(config.temp_mot_over_mode));
            }
        } else {
            self.mot_overheated = false;
        }

        scale_permille
    }
}

const fn mode_to_permille(mode: OverTempMode) -> u16 {
    match mode {
        OverTempMode::Skip => 1000,
        OverTempMode::Power30Percent => 300,
        OverTempMode::Power50Percent => 500,
        OverTempMode::Power70Percent => 700,
    }
}
