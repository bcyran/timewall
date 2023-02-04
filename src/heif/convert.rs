use std::{
    cmp::min,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::Result;
use libheif_rs::{HeifContext, Image};
use log::debug;
use threadpool::ThreadPool;

use super::read;

const CHANNELS: usize = 3;

/// Unpack all images from given HEIF into PNG files in specified directory.
/// Unpacked images will be named by their indices, starting from 0: '0.png', '1.png'...
pub fn unpack_images<P: AsRef<Path>>(heif_ctx: &HeifContext, dest_dir_path: P) -> Result<()> {
    let dest_dir_path = dest_dir_path.as_ref();
    let images = read::get_images(heif_ctx)?;

    let n_threads = min(num_cpus::get(), images.len());
    let thread_pool = ThreadPool::new(n_threads);
    debug!("unpacking using {n_threads} threads");

    for (i, image) in images.into_iter().enumerate() {
        let unpacked_image_path = dest_dir_path.join(format!("{i}.png"));
        thread_pool.execute(move || {
            debug!("writing image to {}", unpacked_image_path.display());
            write_image_as_png(&image, &unpacked_image_path).unwrap();
        });
    }
    thread_pool.join();

    Ok(())
}

/// Write HEIF image as PNG at the specified path.
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
}
