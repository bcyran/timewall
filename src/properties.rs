use std::path::Path;

use anyhow::{anyhow, Context, Result};
use ordered_float::NotNan;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::metadata::AppleDesktop;

/// Property List for the time based wallpaper.
#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct WallpaperPropertiesH24 {
    // Theme appearance details.
    #[serde(rename = "ap", default)]
    pub appearance: Option<Appearance>,
    // Info about the image sequence.
    #[serde(rename = "ti")]
    pub time_info: Vec<TimeItem>,
}

/// Wallpaper appearance depending on the theme.
#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Appearance {
    // Index of the image to use for a dark theme.
    #[serde(rename = "d")]
    pub dark: i32,
    // Index of the image to use for a light theme.
    #[serde(rename = "l")]
    pub light: i32,
}

/// Single image sequence item of the time based wallpaper.
#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub struct TimeItem {
    // Index of the image in the sequence.
    #[serde(rename = "i")]
    pub index: usize,
    // Point in time.
    #[serde(rename = "t")]
    pub time: NotNan<f64>,
}

/// Property List for the sun based wallpaper.
#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct WallpaperPropertiesSolar {
    // Theme appearance details.
    #[serde(rename = "ap", default)]
    pub appearance: Option<Appearance>,
    // Info about the image sequence.
    #[serde(rename = "si")]
    pub solar_info: Vec<SolarItem>,
}

/// Single image sequence item of the sun based wallpaper.
#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub struct SolarItem {
    // Index of the image in the sequence.
    #[serde(rename = "i")]
    pub index: usize,
    // Sun altitude.
    #[serde(rename = "a")]
    pub altitude: NotNan<f64>,
    // Sun azimuth.
    #[serde(rename = "z")]
    pub azimuth: NotNan<f64>,
}

pub trait Plist: DeserializeOwned + Serialize {
    /// Parse base64 encoded `plist`.
    fn from_base64(base64_value: &[u8]) -> Result<Self> {
        let decoded = base64::decode(base64_value)
            .with_context(|| format!("could not decode plist base64"))?;
        plist::from_bytes(&decoded).with_context(|| format!("could not parse plist bytes"))
    }

    /// Deserialize `plist` from XML file.
    fn from_xml_file<T: AsRef<Path>>(path: T) -> Result<Self> {
        plist::from_file(path).with_context(|| format!("could not read plist from XML file"))
    }

    /// Serialize `plist` as XML and write to a file.
    fn to_xml_file<T: AsRef<Path>>(&self, path: T) -> Result<()> {
        plist::to_file_xml(path, &self)
            .with_context(|| format!("could not write plist to XML file"))
    }
}

impl Plist for WallpaperPropertiesH24 {}
impl Plist for WallpaperPropertiesSolar {}

// Wallpaper properties describing either time-based or sun-based schedule
#[derive(Debug)]
pub enum WallpaperProperties {
    // Time-based schedule
    H24(WallpaperPropertiesH24),
    // Sun-based schedule
    Solar(WallpaperPropertiesSolar),
}

impl WallpaperProperties {
    /// Create an instance from apple desktop metadata.
    pub fn from_apple_desktop(apple_desktop: &AppleDesktop) -> Result<Self> {
        let properties = match apple_desktop {
            AppleDesktop::H24(value) => {
                WallpaperProperties::H24(WallpaperPropertiesH24::from_base64(value.as_bytes())?)
            }
            AppleDesktop::Solar(value) => {
                WallpaperProperties::Solar(WallpaperPropertiesSolar::from_base64(value.as_bytes())?)
            }
        };
        Ok(properties)
    }

    /// Load from XML file.
    pub fn from_xml_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        if let Ok(properties_h24) = WallpaperPropertiesH24::from_xml_file(&path) {
            return Ok(Self::H24(properties_h24));
        }
        if let Ok(properties_solar) = WallpaperPropertiesSolar::from_xml_file(&path) {
            return Ok(Self::Solar(properties_solar));
        }
        Err(anyhow!(
            "invalid properties file {}",
            path.as_ref().display()
        ))
    }

    /// Save the properties as a XML file.
    pub fn to_xml_file<P: AsRef<Path>>(&self, dest_path: P) -> Result<()> {
        match self {
            WallpaperProperties::H24(properties) => properties.to_xml_file(dest_path),
            WallpaperProperties::Solar(properties) => properties.to_xml_file(dest_path),
        }
    }

    /// Get number of images defined by those properties.
    pub fn num_images(&self) -> usize {
        // We can't just count time / solar items because they can repeat the same image
        // for different times!
        let max_index = match self {
            WallpaperProperties::H24(properties) => {
                properties.time_info.iter().map(|item| item.index).max()
            }
            WallpaperProperties::Solar(properties) => {
                properties.solar_info.iter().map(|item| item.index).max()
            }
        };
        max_index.unwrap() + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const H24_PLIST_BASE64: &'static str = "YnBsaXN0MDDSAQIDBFJhcFJ0adIFBgcIUWRRbBAFEAKiCQrSCwwNDlF0UWkjP9KqqqAAAAAQANILDA8QIwAAAAAAAAAAEAEIDRATIBgaHB4jNygqLDU8RQAAAAAAAAEBAAAAAAAAABEAAAAAAAAAAAAAAAAAAABH";
    const SOLAR_PLIST_BASE64: &'static str = "YnBsaXN0MDDSAQIDBFJhcFJzadIFBgcIUWRRbBABEACiCQrTCwwNDggPUWFRaVF6I0AuAAAAAAAAI0BgQAAAAAAA0wsMDRAHESPAUYAAAAAAACNASwAAAAAAAAgNEBMgGBocHiNCKiwuMDlJUgAAAAAAAAEBAAAAAAAAABIAAAAAAAAAAAAAAAAAAABb";

    #[test]
    fn test_wallpaper_plist_h24_from_base64() {
        let expected = WallpaperPropertiesH24 {
            appearance: Some(Appearance { dark: 5, light: 2 }),
            time_info: vec![
                TimeItem {
                    index: 0,
                    time: not_nan!(0.2916666567325592),
                },
                TimeItem {
                    index: 1,
                    time: not_nan!(0.0),
                },
            ],
        };

        let result = WallpaperPropertiesH24::from_base64(H24_PLIST_BASE64.as_bytes()).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_wallpaper_plist_solar_from_base64() {
        let expected = WallpaperPropertiesSolar {
            appearance: Some(Appearance { dark: 1, light: 0 }),
            solar_info: vec![
                SolarItem {
                    index: 0,
                    altitude: not_nan!(15.0),
                    azimuth: not_nan!(130.0),
                },
                SolarItem {
                    index: 1,
                    altitude: not_nan!(-70.0),
                    azimuth: not_nan!(54.0),
                },
            ],
        };

        let result = WallpaperPropertiesSolar::from_base64(SOLAR_PLIST_BASE64.as_bytes()).unwrap();

        assert_eq!(result, expected);
    }
}
