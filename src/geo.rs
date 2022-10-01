pub enum Hemisphere {
    Northern,
    Southern,
}

pub struct Coords {
    pub lat: f64,
    pub lon: f64,
}

impl Coords {
    pub fn hemishphere(&self) -> Hemisphere {
        if self.lat >= 0.0 {
            Hemisphere::Northern
        } else {
            Hemisphere::Southern
        }
    }
}
