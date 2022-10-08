use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use libheif_rs::HeifContext;

use crate::{
    metadata::get_apple_desktop_metadata_from_heif,
    properties::{Properties, PropertiesAppearance, PropertiesH24, PropertiesSolar},
    schedule::{sort_solar_items, sort_time_items},
    time::day_fraction_to_time,
};

#[derive(Debug)]
pub struct ImageInfo {
    file: PathBuf,
    size: u64,
    width: u32,
    height: u32,
    images: usize,
    properties: Properties,
}

impl ImageInfo {
    pub fn from_image<P: AsRef<Path>>(image_path: P) -> Result<ImageInfo> {
        let image_path = image_path.as_ref();
        let heif_ctx = HeifContext::read_from_file(image_path.to_str().unwrap())?;
        let primary_handle = heif_ctx.primary_image_handle()?;
        let metadata = get_apple_desktop_metadata_from_heif(&heif_ctx)?;

        Ok(ImageInfo {
            file: image_path.canonicalize()?,
            size: fs::metadata(image_path)?.len(),
            width: primary_handle.width(),
            height: primary_handle.height(),
            images: heif_ctx.number_of_top_level_images(),
            properties: Properties::from_apple_desktop(&metadata)?,
        })
    }

    pub fn schedule_type(&self) -> &str {
        match self.properties {
            Properties::H24(..) => "time",
            Properties::Solar(..) => "solar",
            Properties::Appearance(..) => "appearance",
        }
    }
}

impl Display for ImageInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "File: {}", self.file.display())?;
        writeln!(f, "Size: {}B", self.size)?;
        writeln!(f, "Resolution: {}x{}px", self.width, self.height)?;
        writeln!(f, "Schedule type: {}", self.schedule_type())?;
        writeln!(f, "Number of images: {}", self.images)?;
        writeln!(f, "Number of frames: {}", self.properties.num_frames())?;
        match self.properties {
            Properties::H24(ref props) => {
                writeln!(f, "Schedule:")?;
                fmt_schedule_h24(f, props)?;
            }
            Properties::Solar(ref props) => {
                writeln!(f, "Schedule:")?;
                fmt_schedule_solar(f, props)?;
            }
            _ => (),
        };
        if let Some(appearance_props) = self.properties.appearance() {
            writeln!(f, "Appearance:")?;
            fmt_schedule_appearance(f, appearance_props)?
        }

        Ok(())
    }
}

fn fmt_schedule_h24(f: &mut std::fmt::Formatter, properties: &PropertiesH24) -> std::fmt::Result {
    let sorted_time_items = sort_time_items(&properties.time_info);
    writeln!(f, "Frame Image Time")?;
    for (idx, item) in sorted_time_items.iter().enumerate() {
        let time = day_fraction_to_time(item.time.into_inner());
        writeln!(f, "{:<5} {:<5} {}", idx + 1, item.index, time)?;
    }
    Ok(())
}

fn fmt_schedule_solar(
    f: &mut std::fmt::Formatter,
    properties: &PropertiesSolar,
) -> std::fmt::Result {
    let sorted_solar_items = sort_solar_items(&properties.solar_info);
    writeln!(f, "Frame Image Azimuth Altitude")?;
    for (idx, item) in sorted_solar_items.iter().enumerate() {
        writeln!(
            f,
            "{:<5} {:<5} {:<7.1} {:.1}",
            idx + 1,
            item.index,
            item.azimuth,
            item.altitude,
        )?;
    }
    Ok(())
}

fn fmt_schedule_appearance(
    f: &mut std::fmt::Formatter,
    properties: &PropertiesAppearance,
) -> std::fmt::Result {
    writeln!(f, "Light: {}", properties.light)?;
    writeln!(f, "Dark: {}", properties.dark)?;
    Ok(())
}
