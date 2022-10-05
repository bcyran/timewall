use serde::{Deserialize, Serialize};

pub enum Hemisphere {
    Northern,
    Southern,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Coords {
    pub lat: f64,
    pub lon: f64,
}

impl Coords {
    pub fn hemisphere(&self) -> Hemisphere {
        if self.lat >= 0.0 {
            Hemisphere::Northern
        } else {
            Hemisphere::Southern
        }
    }
}
