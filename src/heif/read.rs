use std::{fs::File, io::Read, path::Path};

use anyhow::{anyhow, Result};
use libheif_rs::{
    check_file_type, ColorSpace, FileTypeResult, HeifContext, HeifError, Image, ImageHandle,
    ItemId, RgbChroma,
};

/// Check whether file at a given path is HEIF and is supported.
pub fn validate_heif_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let mut file = File::open(path)?;
    let mut first_bytes = [0; 64];
    file.read_exact(&mut first_bytes)?;
    match check_file_type(&first_bytes) {
        FileTypeResult::Supported => Ok(()),
        FileTypeResult::No => Err(anyhow!("only HEIC/HEIF files are supported")),
        FileTypeResult::Unsupported | FileTypeResult::MayBe => {
            Err(anyhow!("this HEIF is not supported"))
        }
    }
}

/// Get all available top level image handles from HEIC.
pub fn get_image_handles(image_ctx: &HeifContext) -> Vec<ImageHandle> {
    let number_of_images = image_ctx.number_of_top_level_images();
    let mut image_ids = vec![0 as ItemId; number_of_images];
    image_ctx.top_level_image_ids(&mut image_ids);
    image_ids
        .iter()
        .flat_map(|image_id| image_ctx.image_handle(*image_id))
        .collect()
}

/// Decode image from HEIF handle.
pub fn decode_image_from_handle(image_handle: &ImageHandle) -> Result<Image, HeifError> {
    image_handle.decode(ColorSpace::Rgb(RgbChroma::Rgb), false)
}
