use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use libheif_rs::{HeifContext, ItemId};
use log::debug;
use xml::{
    attribute::OwnedAttribute,
    name::OwnedName,
    reader::{EventReader, XmlEvent},
};

/// AppleDesktop XMP metadata attribute.
#[derive(PartialEq, Debug)]
pub enum AppleDesktop {
    /// H24 variant - time based wallpaper.
    H24(String),
    // Solar variant - sun position baed wallpaper.
    Solar(String),
}

impl AppleDesktop {
    /// Extract attribute from HEIF image.
    pub fn from_heif(image_ctx: &HeifContext) -> Result<AppleDesktop> {
        get_apple_desktop_metadata_from_heif(image_ctx)
    }
}

#[derive(PartialEq, Debug)]
pub struct ImageInfo {
    file: PathBuf,
    size: u64,
    width: u32,
    height: u32,
    images: usize,
    metadata: AppleDesktop,
}

impl ImageInfo {
    pub fn from_image<P: AsRef<Path>>(image_path: P) -> Result<ImageInfo> {
        let image_path = image_path.as_ref();
        let heif_ctx = HeifContext::read_from_file(image_path.to_str().unwrap())?;
        let primary_handle = heif_ctx.primary_image_handle()?;

        Ok(ImageInfo {
            file: image_path.canonicalize()?,
            size: fs::metadata(image_path)?.len(),
            width: primary_handle.width(),
            height: primary_handle.height(),
            images: heif_ctx.number_of_top_level_images(),
            metadata: get_apple_desktop_metadata_from_heif(&heif_ctx)?,
        })
    }
}

impl Display for ImageInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_str = match self.metadata {
            AppleDesktop::H24 { .. } => "time",
            AppleDesktop::Solar { .. } => "solar",
        };
        write!(
            f,
            "File: {}\nSize: {}B\nDimensions: {}x{}\nType: {}\nImages: {}",
            self.file.display(),
            self.size,
            self.width,
            self.height,
            type_str,
            self.images
        )
    }
}

/// Extract apple_desktop attribute from HEIF image.
pub fn get_apple_desktop_metadata_from_heif(image_ctx: &HeifContext) -> Result<AppleDesktop> {
    let xmp_metadata = get_xmp_metadata(image_ctx)?;
    get_apple_desktop_metadata_from_xmp(&xmp_metadata)
}

/// Extract apple_desktop attribute from XMP metadata bytes
pub fn get_apple_desktop_metadata_from_xmp(xmp_metadata: &[u8]) -> Result<AppleDesktop> {
    let mut xmp_reader = EventReader::new(xmp_metadata);
    let rdf_description = get_rdf_description_element(&mut xmp_reader)?;
    if let XmlEvent::StartElement { ref attributes, .. } = rdf_description {
        return get_apple_desktop_attribute(attributes);
    }
    panic!("unexpected XML event")
}

/// Extract XMP metadata bytes from HEIF image.
fn get_xmp_metadata(image_ctx: &HeifContext) -> Result<Box<[u8]>> {
    let primary_image_handle = image_ctx.primary_image_handle()?;

    let mut metadata_ids: [ItemId; 1] = [0];
    primary_image_handle.metadata_block_ids("mime", &mut metadata_ids);
    let xmp_metadata_id = metadata_ids[0];
    debug!("XMP metadata ID: {xmp_metadata_id}");

    let xmp_metadata = primary_image_handle.metadata(xmp_metadata_id)?;

    debug!("XMP metadata read");
    Ok(xmp_metadata.into_boxed_slice())
}

/// Find `<rdf:Description ... />` element using XML event reader.
fn get_rdf_description_element(reader: &mut EventReader<&[u8]>) -> Result<XmlEvent> {
    while let Ok(element) = reader.next() {
        match element {
            XmlEvent::StartElement {
                name:
                    OwnedName {
                        prefix: Some(ref prefix),
                        ref local_name,
                        ..
                    },
                ..
            } if prefix == "rdf" && local_name == "Description" => {
                debug!("rdf:Description element found");
                return Ok(element);
            }
            XmlEvent::EndDocument => break,
            _ => continue,
        }
    }
    Err(anyhow!("missing rdf:Description element"))
}

/// Find `apple_desktop:{h24,solar}` attribute in list of XML attributes.
fn get_apple_desktop_attribute(attributes: &[OwnedAttribute]) -> Result<AppleDesktop> {
    for attribute in attributes {
        match attribute {
            OwnedAttribute {
                name:
                    OwnedName {
                        prefix: Some(prefix),
                        local_name,
                        ..
                    },
                value,
            } if prefix == "apple_desktop" => {
                debug!("apple_desktop:{} attribute found", local_name);
                return match local_name.as_str() {
                    "solar" => Ok(AppleDesktop::Solar(value.to_owned())),
                    "h24" => Ok(AppleDesktop::H24(value.to_owned())),
                    _ => Err(anyhow!("invalid apple_desktop attribute")),
                };
            }
            _ => continue,
        }
    }
    Err(anyhow!("missing apple_desktop attribute"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_xmp_metadata_string(attribute_name: &str, attribute_value: &str) -> String {
        format!(
            r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
            <x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="XMP Core 6.0.0">
                <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
                    <rdf:Description rdf:about=""
                        xmlns:apple_desktop="http://ns.apple.com/namespace/1.0/"
                        {attribute_name}="{attribute_value}" />
                </rdf:RDF>
            </x:xmpmeta><?xpacket end="w"?>"#
        )
    }

    #[test]
    fn test_get_h24_metadata_from_xmp() {
        let expected_value = String::from("dummy_h24_value");
        let xmp = build_xmp_metadata_string("apple_desktop:h24", &expected_value);

        let result = get_apple_desktop_metadata_from_xmp(xmp.as_bytes()).unwrap();

        assert_eq!(result, AppleDesktop::H24(expected_value));
    }

    #[test]
    fn test_get_solar_metadata_from_xmp() {
        let expected_value = String::from("dummy_h24_value");
        let xmp = build_xmp_metadata_string("apple_desktop:solar", &expected_value);

        let result = get_apple_desktop_metadata_from_xmp(xmp.as_bytes()).unwrap();

        assert_eq!(result, AppleDesktop::Solar(expected_value));
    }

    #[test]
    fn test_get_metadata_from_xmp_invalid_attribute() {
        let xmp = build_xmp_metadata_string("apple_desktop:invalid", "whatever");

        let result = get_apple_desktop_metadata_from_xmp(xmp.as_bytes());

        assert!(result.is_err())
    }

    #[test]
    fn test_get_metadata_from_xmp_missing_attribute() {
        let xmp = build_xmp_metadata_string("what", "is this");

        let result = get_apple_desktop_metadata_from_xmp(xmp.as_bytes());

        assert!(result.is_err())
    }

    #[test]
    fn test_get_metadata_from_xmp_missing_element() {
        let xmp = r#"
            <?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
            <x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="XMP Core 6.0.0">
                <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
                </rdf:RDF>
            </x:xmpmeta><?xpacket end="w"?>"#
            .as_bytes();

        let result = get_apple_desktop_metadata_from_xmp(xmp);

        assert!(result.is_err())
    }
}
