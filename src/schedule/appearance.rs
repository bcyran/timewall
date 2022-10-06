use crate::{cli::Appearance, properties::PropertiesAppearance};
use anyhow::Result;

pub fn current_image_index_appearance(
    properties: &PropertiesAppearance,
    user_appearance: Option<Appearance>,
) -> Result<usize> {
    match user_appearance {
        Some(Appearance::Light) | None => Ok(properties.light as usize),
        Some(Appearance::Dark) => Ok(properties.dark as usize),
    }
}

pub fn get_image_index_order_appearance(properties: &PropertiesAppearance) -> Vec<usize> {
    vec![properties.light as usize, properties.dark as usize]
}
