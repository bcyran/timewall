use std::path::{Path, PathBuf};

use crate::heic;
use crate::metadata::AppleDesktop;
use crate::properties::WallpaperProperties;
use anyhow::{anyhow, Result};
use libheif_rs::HeifContext;
use log::debug;

const PROPERTIES_NAME: &str = "properties.xml";

fn image_name(index: usize) -> String {
    format!("{index}.png")
}

/// Unpacked wallpaper laying somewhere in the filesystem.
#[derive(Debug)]
pub struct Wallpaper {
    /// Paths of extracted images.
    images: Vec<PathBuf>,
    /// Wallpaper properties.
    properties: WallpaperProperties,
}

impl Wallpaper {
    /// Load wallpaper from a directory it was unpacked to.
    pub fn load<P: AsRef<Path>>(dir_path: P) -> Result<Self> {
        let dir_path = dir_path.as_ref();

        let properties = WallpaperProperties::from_xml_file(dir_path.join(PROPERTIES_NAME))?;
        let mut images: Vec<PathBuf> = Vec::with_capacity(properties.num_images());

        for i in 0..properties.num_images() {
            let image_path = dir_path.join(image_name(i)).canonicalize()?;
            if !image_path.exists() {
                return Err(anyhow!("image {i} present in properties but not in dir"));
            }
            images.push(image_path);
        }

        Ok(Wallpaper { images, properties })
    }
}

/// Unpack wallpaper images and properties from HEIC into a directory.
pub fn unpack_heic<IP: AsRef<Path>, DP: AsRef<Path>>(image_path: IP, dest_path: DP) -> Result<()> {
    let image_path = image_path.as_ref();
    let dest_path = dest_path.as_ref();

    if !dest_path.is_dir() {
        return Err(anyhow!("{} is not a directory", dest_path.display()));
    }

    let heif_ctx = HeifContext::read_from_file(image_path.to_str().unwrap())?;
    let apple_desktop_meta = AppleDesktop::from_heif(&heif_ctx)?;
    let properties = WallpaperProperties::from_apple_desktop(&apple_desktop_meta)?;
    let image_handles = heic::get_image_handles(&heif_ctx);
    debug!("found {} images", image_handles.len());

    for (i, image_handle) in image_handles.iter().enumerate() {
        let unpacked_image_path = dest_path.join(image_name(i));
        debug!("writing image to {}", unpacked_image_path.display());
        heic::write_as_png(image_handle, &unpacked_image_path)?;
    }

    let properties_path = dest_path.join(PROPERTIES_NAME);
    debug!("writing properties to {}", properties_path.display());
    properties.to_xml_file(&properties_path)?;

    Ok(())
}
