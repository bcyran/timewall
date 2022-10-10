use std::{fs::File, io::Read, path::Path};

use anyhow::{anyhow, Context, Ok, Result};
use libheif_rs::{
    check_file_type, ColorSpace, FileTypeResult, HeifContext, HeifError, Image, ItemId, RgbChroma,
};
use log::debug;

/// Check whether file at a given path is HEIF and is supported.
pub fn validate_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let mut file = File::open(path)?;
    let mut first_bytes = [0; 12];
    file.read_exact(&mut first_bytes)?;
    match check_file_type(&first_bytes) {
        FileTypeResult::Supported => Ok(()),
        FileTypeResult::No => Err(anyhow!("only HEIF files are supported")),
        FileTypeResult::Unsupported | FileTypeResult::MayBe => {
            Err(anyhow!("this HEIF is not supported"))
        }
    }
}

/// Extract XMP metadata bytes from HEIF image.
pub fn get_xmp_metadata(heif_ctx: &HeifContext) -> Result<Box<[u8]>> {
    let primary_image_handle = heif_ctx.primary_image_handle()?;

    let mut metadata_ids: [ItemId; 1] = [0];
    primary_image_handle.metadata_block_ids("mime", &mut metadata_ids);
    let xmp_metadata_id = metadata_ids[0];
    debug!("XMP metadata ID: {xmp_metadata_id}");

    let xmp_metadata = primary_image_handle.metadata(xmp_metadata_id)?;

    debug!("XMP metadata read");
    Ok(xmp_metadata.into_boxed_slice())
}

/// Get all available top level image handles from HEIF.
pub fn get_images(heif_ctx: &HeifContext) -> Result<Vec<Image>> {
    let number_of_images = heif_ctx.number_of_top_level_images();
    debug!("found {number_of_images} images");
    let mut image_ids = vec![0 as ItemId; number_of_images];
    heif_ctx.top_level_image_ids(&mut image_ids);
    image_ids
        .iter()
        .map(|image_id| heif_ctx.image_handle(*image_id))
        .collect::<Result<Vec<_>, HeifError>>()?
        .iter()
        .map(|image_handle| image_handle.decode(ColorSpace::Rgb(RgbChroma::Rgb), false))
        .collect::<Result<Vec<_>, HeifError>>()
        .context("couldn't extract some images from HEIF")
}
