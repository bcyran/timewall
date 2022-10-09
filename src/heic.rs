use std::{
    fs::File,
    io::{BufWriter, Read, Write},
    path::Path,
};

use anyhow::{anyhow, Result};
use libheif_rs::{
    check_file_type, Channel, ColorSpace, FileTypeResult, HeifContext, HeifError, Image,
    ImageHandle, ItemId, RgbChroma,
};
use log::debug;

const CHANNELS: usize = 3;

/// Check whether file at a given path is HEIC and is supported.
pub fn validate_heic_file<P: AsRef<Path>>(path: P) -> Result<()> {
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

/// Decode image from HEIC handle.
pub fn decode_image_from_handle(image_handle: &ImageHandle) -> Result<Image, HeifError> {
    image_handle.decode(ColorSpace::Rgb(RgbChroma::Rgb), false)
}

/// Write HEIC image as PNG at the specified path.
pub fn write_image_as_png<P: AsRef<Path>>(image: &Image, path: P) -> Result<()> {
    let output = File::create(&path)?;
    let output_writer = BufWriter::new(output);

    let image_plane = image.planes().interleaved.unwrap();
    let line_length = image_plane.width as usize * CHANNELS;

    let mut png_encoder = png::Encoder::new(output_writer, image_plane.width, image_plane.height);
    png_encoder.set_color(png::ColorType::Rgb);
    png_encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = png_encoder.write_header()?.into_stream_writer()?;

    write_from_padded_data(
        &mut png_writer,
        image_plane.data,
        image_plane.stride,
        line_length,
    )?;

    Ok(())
}

/// Write potentially padded image data, removing line padding if it's present.
/// Read up on "image stride" if you don't get what's going on here.
fn write_from_padded_data<W: Write>(
    writer: &mut W,
    data: &[u8],
    data_stride: usize,
    line_length: usize,
) -> Result<()> {
    if data_stride == line_length {
        // If stride is equal to line line length, then there's no padding.
        // We can just write everything.
        debug!("image lines not padded");
        writer.write_all(data)?;
    } else {
        // Otherwise, we have to write line by line, removing the padding in the process.
        debug!("image lines padded");
        for data_line in data.chunks(data_stride) {
            writer.write_all(&data_line[..line_length])?;
        }
    }

    Ok(())
}

/// Read HEIC `Image` from PNG file at the specified path.
pub fn read_image_from_png<P: AsRef<Path>>(path: P) -> Result<Image> {
    let file = File::open(path)?;
    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info()?;
    let mut png_data = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut png_data)?;
    png_data.truncate(info.buffer_size());
    png_data.shrink_to_fit();

    let mut image = Image::new(info.width, info.height, ColorSpace::Rgb(RgbChroma::Rgb))?;
    image.create_plane(Channel::Interleaved, info.width, info.height, 8)?;
    let mut image_plane = image.planes_mut().interleaved.unwrap();

    write_to_padded_data(
        &mut image_plane.data,
        &png_data,
        image_plane.stride,
        info.line_size,
    )?;

    Ok(image)
}

/// Write unpadded data into potentially padded container, adding the padding if necessary.
fn write_to_padded_data<W: Write>(
    writer: &mut W,
    data: &[u8],
    data_stride: usize,
    line_length: usize,
) -> Result<()> {
    let padding_length = data_stride - line_length;
    if padding_length == 0 {
        debug!("image lines not padded");
        writer.write_all(data)?;
    } else {
        debug!("image lines padded");
        let padding = vec![0; padding_length];
        for data_line in data.chunks(line_length) {
            writer.write_all(&data_line)?;
            writer.write_all(&padding)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_write_from_padded_data_no_padding() {
        #[rustfmt::skip]
        let data: &[u8] = &[
            1, 1, 1, 2, 2, 2,
            3, 3, 3, 4, 4, 4,
            5, 5, 5, 6, 6, 6,
        ];
        let data_stride = 6;
        let line_length = 6;

        let mut result: Vec<u8> = Vec::new();
        write_from_padded_data(&mut result, data, data_stride, line_length).unwrap();

        assert_eq!(result, data);
    }

    #[test]
    pub fn test_write_from_padded_data_with_padding() {
        #[rustfmt::skip]
        let data: &[u8] = &[
            1, 1, 1, 2, 2, 2, 0, 0, 0,
            3, 3, 3, 4, 4, 4, 0, 0, 0,
            5, 5, 5, 6, 6, 6, 0, 0, 0,
        ];
        let data_stride = 9;
        let line_length = 6;
        #[rustfmt::skip]
        let expected: &[u8] = &[
            1, 1, 1, 2, 2, 2,
            3, 3, 3, 4, 4, 4,
            5, 5, 5, 6, 6, 6,
        ];

        let mut result: Vec<u8> = Vec::new();
        write_from_padded_data(&mut result, data, data_stride, line_length).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_write_to_padded_data_no_padding() {
        #[rustfmt::skip]
        let data = &[
            1, 1, 1,
            2, 2, 2,
            3, 3, 3
        ];
        let data_stride = 3;
        let line_length = 3;

        let mut result: Vec<u8> = Vec::new();
        write_to_padded_data(&mut result, data, data_stride, line_length).unwrap();

        assert_eq!(result, data);
    }

    #[test]
    fn test_write_to_padded_data_with_padding() {
        #[rustfmt::skip]
        let data = &[
            1, 1, 1,
            2, 2, 2,
            3, 3, 3
        ];
        let data_stride = 6;
        let line_length = 3;
        #[rustfmt::skip]
        let expected = &[
            1, 1, 1, 0, 0, 0,
            2, 2, 2, 0, 0, 0,
            3, 3, 3, 0, 0, 0,
        ];

        let mut result: Vec<u8> = Vec::new();
        write_to_padded_data(&mut result, data, data_stride, line_length).unwrap();

        assert_eq!(result, expected);
    }
}
