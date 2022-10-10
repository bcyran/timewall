pub mod metadata;
pub mod properties;

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use libheif_rs::HeifContext;
use log::debug;
use metadata::AppleDesktop;
use properties::Properties;

use crate::heif;

const PROPERTIES_NAME: &str = "properties.xml";

/// Unpacked wallpaper laying somewhere in the filesystem.
#[derive(Debug)]
pub struct Wallpaper {
    /// Paths of extracted images.
    pub images: Vec<PathBuf>,
    /// Wallpaper properties.
    pub properties: Properties,
}

impl Wallpaper {
    /// Load wallpaper from a directory it was unpacked to.
    pub fn load<P: AsRef<Path>>(dir_path: P) -> Result<Self> {
        let dir_path = dir_path.as_ref();

        let properties = Properties::from_xml_file(dir_path.join(PROPERTIES_NAME))?;
        let mut images: Vec<PathBuf> = Vec::with_capacity(properties.num_images());

        for i in 0..properties.num_images() {
            let image_path = dir_path.join(format!("{i}.png")).canonicalize()?;
            if !image_path.exists() {
                return Err(anyhow!("image {i} present in properties but not in dir"));
            }
            images.push(image_path);
        }

        Ok(Wallpaper { images, properties })
    }
}

/// Unpack wallpaper images and properties from HEIF into a directory.
pub fn unpack<IP: AsRef<Path>, DP: AsRef<Path>>(wall_path: IP, dest_dir_path: DP) -> Result<()> {
    let image_path = wall_path.as_ref();
    let dest_dir_path = dest_dir_path.as_ref();

    if !dest_dir_path.is_dir() {
        return Err(anyhow!("{} is not a directory", dest_dir_path.display()));
    }

    let heif_ctx = HeifContext::read_from_file(image_path.to_str().unwrap())?;
    heif::unpack_images(&heif_ctx, dest_dir_path)?;
    unpack_properties(&heif_ctx, dest_dir_path.join(PROPERTIES_NAME))?;

    Ok(())
}

fn unpack_properties<P: AsRef<Path>>(heif_ctx: &HeifContext, dest_path: P) -> Result<()> {
    let dest_path = dest_path.as_ref();
    let apple_desktop_meta = AppleDesktop::from_heif(heif_ctx)?;
    let properties = Properties::from_apple_desktop(&apple_desktop_meta)?;
    debug!("writing properties to {}", dest_path.display());
    properties.to_xml_file(&dest_path)
}
