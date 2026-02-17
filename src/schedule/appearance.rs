use crate::{appearance::Appearance, wallpaper::properties::PropertiesAppearance};

#[allow(clippy::cast_sign_loss)]
pub const fn current_image_index_appearance(
    properties: &PropertiesAppearance,
    appearance: Appearance,
) -> usize {
    match appearance {
        Appearance::Light => properties.light as usize,
        Appearance::Dark => properties.dark as usize,
    }
}

#[allow(clippy::cast_sign_loss)]
pub fn get_image_index_order_appearance(properties: &PropertiesAppearance) -> Vec<usize> {
    vec![properties.light as usize, properties.dark as usize]
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    #[fixture]
    fn properties() -> PropertiesAppearance {
        PropertiesAppearance {
            dark: 69,
            light: 420,
        }
    }

    #[rstest]
    #[case(Appearance::Light, 420)]
    #[case(Appearance::Dark, 69)]
    fn test_current_image_appearance(
        properties: PropertiesAppearance,
        #[case] appearance: Appearance,
        #[case] expected_index: usize,
    ) {
        let result = current_image_index_appearance(&properties, appearance);
        assert_eq!(result, expected_index);
    }

    #[rstest]
    fn test_get_image_index_order_h24(properties: PropertiesAppearance) {
        let result = get_image_index_order_appearance(&properties);
        assert_eq!(result, &[420, 69]);
    }
}
