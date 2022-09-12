use anyhow::{anyhow, Result};
use libheif_rs::{HeifContext, ItemId};
use log::debug;
use xml::{
    attribute::OwnedAttribute,
    name::OwnedName,
    reader::{EventReader, XmlEvent},
};

#[derive(PartialEq, Debug)]
/// AppleDesktop XMP metadata attribute
pub enum AppleDesktop {
    /// H24 variant - time based wallpaper
    H24(String),
    // Solar variant - sun position baed wallpaper
    Solar(String),
}

impl AppleDesktop {
    /// Extract attribute from HEIF image
    pub fn from_heif(image_ctx: &HeifContext) -> Result<AppleDesktop> {
        return get_apple_desktop_metadata_from_heif(image_ctx);
    }
}

/// Extract apple_desktop attribute from HEIF image
pub fn get_apple_desktop_metadata_from_heif(image_ctx: &HeifContext) -> Result<AppleDesktop> {
    let mut xmp_metadata = get_xmp_metadata(image_ctx)?;
    xmp_metadata.pop();
    get_apple_desktop_metadata_from_xmp(xmp_metadata)
}

/// Extract apple_desktop attribute from XMP metadata string
pub fn get_apple_desktop_metadata_from_xmp(xmp_metadata: String) -> Result<AppleDesktop> {
    let xmp_reader = EventReader::from_str(&xmp_metadata);
    let rdf_description = get_rdf_description_element(xmp_reader)?;
    match rdf_description {
        XmlEvent::StartElement { attributes, .. } => get_apple_desktop_attribute(attributes),
        _ => Err(anyhow!("invalid rdf:Description element type")),
    }
}

/// Extract XMP metadata string from HEIF image
fn get_xmp_metadata(image_ctx: &HeifContext) -> Result<String> {
    let primary_image_handle = image_ctx.primary_image_handle()?;

    let mut metadata_ids: [ItemId; 1] = [0];
    primary_image_handle.metadata_block_ids("mime", &mut metadata_ids);
    let xmp_metadata_id = metadata_ids[0];
    debug!("XMP metadata ID: {xmp_metadata_id}");

    let raw_metadata = primary_image_handle.metadata(xmp_metadata_id)?;
    let metadata_string = String::from_utf8_lossy(&raw_metadata).into_owned();

    debug!("XMP metadata read");
    Ok(metadata_string)
}

/// Find `<rdf:Description ... />` element using XML event reader
fn get_rdf_description_element(mut reader: EventReader<&[u8]>) -> Result<XmlEvent> {
    loop {
        let element = reader.next()?;
        match element {
            XmlEvent::StartElement {
                name:
                    OwnedName {
                        prefix: Some(ref prefix),
                        ref local_name,
                        ..
                    },
                ..
            } if prefix.as_str() == "rdf" && local_name.as_str() == "Description" => {
                debug!("rdf:Description element found");
                return Ok(element);
            }
            XmlEvent::EndDocument => return Err(anyhow!("missing rdf:Description element")),
            _ => continue,
        }
    }
}

/// Find `apple_desktop:{h24,solar}` attribute in list of XML attributes
fn get_apple_desktop_attribute(attributes: Vec<OwnedAttribute>) -> Result<AppleDesktop> {
    for attribute in attributes {
        if attribute.name.prefix != Some(String::from("apple_desktop")) {
            continue;
        }
        let attribute_value = String::from(&attribute.value);
        debug!(
            "apple_desktop:{} attribute found: {:?}",
            attribute.name.local_name, attribute_value
        );
        return match attribute.name.local_name.as_str() {
            "solar" => Ok(AppleDesktop::Solar(attribute_value)),
            "h24" => Ok(AppleDesktop::H24(attribute_value)),
            _ => Err(anyhow!("invalid apple_desktop attribute")),
        };
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
    fn test_get_h24_metadata_from_xmp() -> Result<()> {
        let expected_value = "dummy_h24_value";
        let xmp = build_xmp_metadata_string("apple_desktop:h24", expected_value);

        let result = get_apple_desktop_metadata_from_xmp(xmp)?;

        assert_eq!(result, AppleDesktop::H24(expected_value.to_string()));
        Ok(())
    }

    #[test]
    fn test_get_solar_metadata_from_xmp() -> Result<()> {
        let expected_value = "dummy_h24_value";
        let xmp = build_xmp_metadata_string("apple_desktop:solar", expected_value);

        let result = get_apple_desktop_metadata_from_xmp(xmp)?;

        assert_eq!(result, AppleDesktop::Solar(expected_value.to_string()));
        Ok(())
    }

    #[test]
    fn test_get_metadata_from_xmp_invalid_attribute() {
        let xmp = build_xmp_metadata_string("apple_desktop:invalid", "whatever");

        let result = get_apple_desktop_metadata_from_xmp(xmp);

        assert!(result.is_err())
    }

    #[test]
    fn test_get_metadata_from_xmp_missing_attribute() {
        let xmp = build_xmp_metadata_string("what", "is this");

        let result = get_apple_desktop_metadata_from_xmp(xmp);

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
            .to_string();

        let result = get_apple_desktop_metadata_from_xmp(xmp);

        assert!(result.is_err())
    }
}
