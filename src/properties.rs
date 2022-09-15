use anyhow::{Context, Result};
use plist;
use serde::Deserialize;

/// Property List for the time based wallpaper.
#[derive(Deserialize, PartialEq, Debug)]
pub struct WallpaperPlistH24 {
    // Theme appearance details.
    #[serde(rename = "ap")]
    pub appearance: Appearance,
    // Info about the image sequence.
    #[serde(rename = "ti")]
    pub time_info: Vec<TimeItem>,
}

/// Wallpaper appearance depending on the theme.
#[derive(Deserialize, PartialEq, Debug)]
pub struct Appearance {
    // Index of the image to use for a dark theme.
    #[serde(rename = "d")]
    pub dark: i32,
    // Index of the image to use for a light theme.
    #[serde(rename = "l")]
    pub light: i32,
}

/// Single image sequence item of the time based wallpaper.
#[derive(Deserialize, PartialEq, Debug)]
pub struct TimeItem {
    // Index of the image in the sequence.
    #[serde(rename = "i")]
    pub index: usize,
    // Point in time.
    #[serde(rename = "t")]
    pub time: f32,
}

/// Property List for the sun based wallpaper.
#[derive(Deserialize, PartialEq, Debug)]
pub struct WallpaperPlistSolar {
    // Theme appearance details.
    #[serde(rename = "ap")]
    pub appearance: Appearance,
    // Info about the image sequence.
    #[serde(rename = "si")]
    pub solar_info: Vec<SolarItem>,
}

/// Single image sequence item of the sun based wallpaper.
#[derive(Deserialize, PartialEq, Debug)]
pub struct SolarItem {
    // Index of the image in the sequence.
    #[serde(rename = "i")]
    pub index: usize,
    // Sun altitude.
    #[serde(rename = "a")]
    pub altitude: f32,
    // Sun azimuth.
    #[serde(rename = "z")]
    pub azimuth: f32,
}

impl WallpaperPlistH24 {
    /// Parse base64 encoded `plist`.
    pub fn from_base64(base64_value: String) -> Result<WallpaperPlistH24> {
        plist::from_bytes(&decode_base64(base64_value)?)
            .with_context(|| format!("could not parse plist bytes"))
    }
}

impl WallpaperPlistSolar {
    /// Parse base64 encoded `plist`.
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
                    index: 0,
                    time: 0.29166666,
                },
                TimeItem {
                    index: 1,
                    time: 0.0,
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
                    index: 0,
                    altitude: 15.0,
                    azimuth: 130.0,
                },
                SolarItem {
                    index: 1,
                    altitude: -70.0,
                    azimuth: 54.0,
                },
            ],
        };

        let result = WallpaperPlistSolar::from_base64(SOLAR_PLIST_BASE64.to_string()).unwrap();

        assert_eq!(result, expected);
    }
}
