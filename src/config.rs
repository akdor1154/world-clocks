use cosmic::cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, CosmicConfigEntry, PartialEq, Eq)]
#[version = 1]
pub struct WorldClocksConfig {
    pub timezones: Vec<Tz>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Tz {
    pub name: String,
    pub display_name: String,
}

impl Default for WorldClocksConfig {
    fn default() -> Self {
        Self {
            timezones: vec![
                Tz {
                    name: "Etc/UTC".into(),
                    display_name: "UTC".into(),
                },
                Tz {
                    name: "Europe/London".into(),
                    display_name: "London".into(),
                },
                Tz {
                    name: "Australia/Perth".into(),
                    display_name: "Perth".into(),
                },
            ],
        }
    }
}
