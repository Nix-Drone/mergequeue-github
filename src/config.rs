// trunk-ignore-all(trunk-toolbox/do-not-land)
use confique::toml::{self, FormatOptions};
use confique::Config;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    SingleQueue,
    ParallelQueue,
}

#[derive(Config)]
pub struct Conf {
    #[config(default = "singlequeue")]
    pub mode: Mode,

    #[config(default = 0.1)]
    pub flake_rate: f32,
}

impl Conf {
    pub fn print_default() {
        let default_config = toml::template::<Conf>(FormatOptions::default());
        println!("{}", default_config);
    }

    pub fn is_valid(&self) -> Result<(), &'static str> {
        if self.flake_rate >= 0.0 && self.flake_rate <= 1.0 {
            Ok(())
        } else {
            Err("flake_rate must be between 0 and 1")
        }
    }
}
