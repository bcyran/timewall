use anyhow::{Context, Result};
use plist;
use serde::Deserialize;

#[derive(Deserialize, PartialEq, Debug)]
pub struct WallpaperPlistH24 {
    #[serde(rename = "ap")]
    pub appearance: Appearance,
    #[serde(rename = "ti")]
    pub time_info: Vec<TimeItem>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Appearance {
    #[serde(rename = "d")]
    pub dark: i32,
    #[serde(rename = "l")]
    pub light: i32,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct TimeItem {
    #[serde(rename = "t")]
    pub time: f32,
    #[serde(rename = "i")]
    pub index: usize,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct WallpaperPlistSolar {
    #[serde(rename = "ap")]
    pub appearance: Appearance,
    #[serde(rename = "si")]
    pub solar_info: Vec<SolarItem>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct SolarItem {
    #[serde(rename = "a")]
    pub altitude: f32,
    #[serde(rename = "i")]
    pub index: usize,
    #[serde(rename = "z")]
    pub azimuth: f32,
}

impl WallpaperPlistH24 {
    pub fn from_base64(base64_value: String) -> Result<WallpaperPlistH24> {
        plist::from_bytes(&decode_base64(base64_value)?)
            .with_context(|| format!("could not parse plist bytes"))
    }
}

impl WallpaperPlistSolar {
    pub fn from_base64(base64_value: String) -> Result<WallpaperPlistSolar> {
        plist::from_bytes(&decode_base64(base64_value)?)
            .with_context(|| format!("could not parse plist bytes"))
    }
}

fn decode_base64(base64_value: String) -> Result<Vec<u8>> {
    base64::decode(base64_value).with_context(|| format!("could not decode plist base64"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const H24_PLIST_BASE64: &'static str = "YnBsaXN0MDDSAQIDBFJhcFJ0adIFBgcIUWRRbBAFEAKiCQrSCwwNDlF0UWkjP9KqqqAAAAAQANILDA8QIwAAAAAAAAAAEAEIDRATIBgaHB4jNygqLDU8RQAAAAAAAAEBAAAAAAAAABEAAAAAAAAAAAAAAAAAAABH";
    const SOLAR_PLIST_BASE64: &'static str = "YnBsaXN0MDDSAQIDBFJhcFJzadIFBgcIUWRRbBABEACiCQrTCwwNDggPUWFRaVF6I0AuAAAAAAAAI0BgQAAAAAAA0wsMDRAHESPAUYAAAAAAACNASwAAAAAAAAgNEBMgGBocHiNCKiwuMDlJUgAAAAAAAAEBAAAAAAAAABIAAAAAAAAAAAAAAAAAAABb";

    #[test]
    fn test_wallpaper_plist_h24_from_base64() {
        let expected = WallpaperPlistH24 {
            appearance: Appearance { dark: 5, light: 2 },
            time_info: vec![
                TimeItem {
                    time: 0.29166666,
                    index: 0,
                },
                TimeItem {
                    time: 0.0,
                    index: 1,
                },
            ],
        };

        let result = WallpaperPlistH24::from_base64(H24_PLIST_BASE64.to_string()).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_wallpaper_plist_solar_from_base64() {
        let expected = WallpaperPlistSolar {
            appearance: Appearance { dark: 1, light: 0 },
            solar_info: vec![
                SolarItem {
                    altitude: 15.0,
                    index: 0,
                    azimuth: 130.0,
                },
                SolarItem {
                    altitude: -70.0,
                    index: 1,
                    azimuth: 54.0,
                },
            ],
        };

        let result = WallpaperPlistSolar::from_base64(SOLAR_PLIST_BASE64.to_string()).unwrap();

        assert_eq!(result, expected);
    }
}
