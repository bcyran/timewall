use anyhow::{anyhow, bail, Context, Result};
use libheif_rs::HeifContext;
use log::debug;
use xml::{
    attribute::OwnedAttribute,
    name::OwnedName,
    reader::{EventReader, XmlEvent},
};

use crate::heif;

const APPLE_DESKTOP_PREFIX: &str = "apple_desktop";

/// `AppleDesktop` XMP metadata attribute.
#[derive(PartialEq, Eq, Debug)]
pub enum AppleDesktop {
    /// H24 variant - time based wallpaper.
    H24(String),
    /// Solar variant - sun position baed wallpaper.
    Solar(String),
    /// Appearance variant - light and dark mode wallpaper.
    Apr(String),
}

impl AppleDesktop {
    /// Extract attribute from HEIF image.
    pub fn from_heif(heif_ctx: &HeifContext) -> Result<Self> {
        get_apple_desktop_metadata_from_heif(heif_ctx)
    }

    fn new(type_name: &str, value: &str) -> Result<Self> {
        match type_name {
            "h24" => Ok(Self::H24(value.to_owned())),
            "solar" => Ok(Self::Solar(value.to_owned())),
            "apr" => Ok(Self::Apr(value.to_owned())),
            _ => bail!("invalid {APPLE_DESKTOP_PREFIX} metadata type: {type_name}"),
        }
    }

    /// Get a new `AppleDesktop` instance with the value replaced.
    const fn with_replaced_value(&self, value: String) -> Self {
        match self {
            Self::H24(_) => Self::H24(value),
            Self::Solar(_) => Self::Solar(value),
            Self::Apr(_) => Self::Apr(value),
        }
    }
}

/// Extract `apple_desktop` metadata from HEIF image.
pub fn get_apple_desktop_metadata_from_heif(heif_ctx: &HeifContext) -> Result<AppleDesktop> {
    let xmp_metadata = heif::get_xmp_metadata(heif_ctx).context("couldn't read XMP metadata")?;
    get_apple_desktop_metadata_from_xmp(&xmp_metadata)
}

/// Extract `apple_desktop` metadata from XMP metadata bytes.
pub fn get_apple_desktop_metadata_from_xmp(xmp_metadata: &[u8]) -> Result<AppleDesktop> {
    // Try to extract from XML attribute first
    if let Some(metadata) = get_apple_desktop_metadata_from_xml_attribute(xmp_metadata)? {
        return Ok(metadata);
    }

    // If that fails, try to extract from XML element
    if let Some(metadata) = get_apple_desktop_metadata_from_xml_element(xmp_metadata)? {
        return Ok(metadata);
    }

    Err(anyhow!(
        "{APPLE_DESKTOP_PREFIX} metadata not found in XMP metadata"
    ))
}

fn get_apple_desktop_metadata_from_xml_element(xml_content: &[u8]) -> Result<Option<AppleDesktop>> {
    let xmp_reader = EventReader::new(xml_content);
    let mut maybe_metadata_type: Option<AppleDesktop> = None;

    for event in xmp_reader {
        match event {
            Ok(XmlEvent::StartElement {
                name:
                    OwnedName {
                        prefix: Some(ref prefix),
                        ref local_name,
                        ..
                    },
                ..
            }) if prefix == APPLE_DESKTOP_PREFIX => {
                debug!("{APPLE_DESKTOP_PREFIX}:{local_name} element found");
                maybe_metadata_type = Some(AppleDesktop::new(local_name, "")?);
            }
            Ok(XmlEvent::Characters(text)) => {
                if let Some(ref metada_type) = maybe_metadata_type {
                    return Ok(Some(metada_type.with_replaced_value(text)));
                }
            }
            _ => (),
        }
    }

    debug!("{APPLE_DESKTOP_PREFIX} element not found");
    Ok(None)
}

fn get_apple_desktop_metadata_from_xml_attribute(
    xmp_metadata: &[u8],
) -> Result<Option<AppleDesktop>> {
    let mut xmp_reader = EventReader::new(xmp_metadata);
    let rdf_description = get_rdf_description_element(&mut xmp_reader);

    if let Some(XmlEvent::StartElement { ref attributes, .. }) = rdf_description {
        return get_apple_desktop_attribute(attributes);
    }

    Ok(None)
}

/// Find `<rdf:Description ... />` element using XML event reader.
fn get_rdf_description_element(reader: &mut EventReader<&[u8]>) -> Option<XmlEvent> {
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
                return Some(element);
            }
            XmlEvent::EndDocument => break,
            _ => (),
        }
    }

    None
}

/// Find `apple_desktop:{h24,solar}` attribute in list of XML attributes.
fn get_apple_desktop_attribute(attributes: &[OwnedAttribute]) -> Result<Option<AppleDesktop>> {
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
            } if prefix == APPLE_DESKTOP_PREFIX => {
                debug!("{APPLE_DESKTOP_PREFIX}:{local_name} attribute found");
                return Ok(Some(AppleDesktop::new(local_name, value)?));
            }
            _ => (),
        }
    }

    debug!("{APPLE_DESKTOP_PREFIX} attribute not found");
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    fn build_xmp_metadata_string_with_attribute(name: &str, value: &str) -> String {
        format!(
            r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
            <x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="XMP Core 6.0.0">
                <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
                    <rdf:Description rdf:about=""
                        xmlns:apple_desktop="http://ns.apple.com/namespace/1.0/"
                        {name}="{value}" />
                </rdf:RDF>
            </x:xmpmeta><?xpacket end="w"?>"#
        )
    }

    fn build_xmp_metadata_string_with_element(name: &str, value: &str) -> String {
        format!(
            r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
            <x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="XMP Core 6.0.0">
            <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
                <rdf:Description rdf:about=""
                        xmlns:apple_desktop="http://ns.apple.com/namespace/1.0/">
                    <{name}>{value}</{name}>
                </rdf:Description>
            </rdf:RDF>
            </x:xmpmeta><?xpacket end="w"?>"#
        )
    }

    const DUMMY_VALUE: &str = "dummy_value";

    #[rstest]
    #[case("apple_desktop:h24", AppleDesktop::H24(String::from(DUMMY_VALUE)))]
    #[case("apple_desktop:solar", AppleDesktop::Solar(String::from(DUMMY_VALUE)))]
    #[case("apple_desktop:apr", AppleDesktop::Apr(String::from(DUMMY_VALUE)))]
    fn test_get_h24_metadata_from_xmp_attribute(
        #[case] attribute_name: &str,
        #[case] expected_value: AppleDesktop,
    ) {
        let xmp = build_xmp_metadata_string_with_attribute(attribute_name, DUMMY_VALUE);
        let result = get_apple_desktop_metadata_from_xmp(xmp.as_bytes()).unwrap();
        assert_eq!(result, expected_value);
    }

    #[rstest]
    #[case("apple_desktop:h24", AppleDesktop::H24(String::from(DUMMY_VALUE)))]
    #[case("apple_desktop:solar", AppleDesktop::Solar(String::from(DUMMY_VALUE)))]
    #[case("apple_desktop:apr", AppleDesktop::Apr(String::from(DUMMY_VALUE)))]
    fn test_get_metadata_from_xmp_element(
        #[case] element_name: &str,
        #[case] expected_value: AppleDesktop,
    ) {
        let xmp = build_xmp_metadata_string_with_element(element_name, DUMMY_VALUE);
        let result = get_apple_desktop_metadata_from_xmp(xmp.as_bytes()).unwrap();
        assert_eq!(result, expected_value);
    }

    #[rstest]
    #[case("apple_desktop:invalid")]
    #[case("what")]
    fn test_get_metadata_from_xmp_invalid_attribute(#[case] attribute_name: &str) {
        let xmp = build_xmp_metadata_string_with_attribute(attribute_name, "whatever");
        let result = get_apple_desktop_metadata_from_xmp(xmp.as_bytes());
        assert!(result.is_err());
    }

    #[rstest]
    #[case("apple_desktop:invalid")]
    #[case("what")]
    fn test_get_metadata_from_xmp_invalid_element(#[case] element_name: &str) {
        let xmp = build_xmp_metadata_string_with_element(element_name, "whatever");
        let result = get_apple_desktop_metadata_from_xmp(xmp.as_bytes());
        assert!(result.is_err());
    }

    #[test]
    fn test_get_metadata_from_xmp_missing() {
        let xmp = r#"
            <?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
            <x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="XMP Core 6.0.0">
                <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
                </rdf:RDF>
            </x:xmpmeta><?xpacket end="w"?>"#
            .as_bytes();

        let result = get_apple_desktop_metadata_from_xmp(xmp);

        assert!(result.is_err());
    }
}
