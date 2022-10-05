use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Local};
use itertools::Itertools;
use log::debug;
use sun::Position;

use crate::{
    geo::{Coords, Hemisphere},
    properties::SolarItem,
};

/// Get the index of the image which should be displayed for given datetime and location.
pub fn current_image_index_solar(
    solar_items: &[SolarItem],
    datetime: &DateTime<Local>,
    coords: &Coords,
) -> Result<usize> {
    let sun_pos = sun::pos(datetime.timestamp_millis(), coords.lat, coords.lon);
    let sun_pos_degrees = Position {
        azimuth: sun_pos.azimuth.to_degrees(),
        altitude: sun_pos.altitude.to_degrees(),
    };
    debug!("sun position: {:?}", sun_pos_degrees);
    current_image_index_from_sun_pos(solar_items, &sun_pos_degrees, &coords.hemisphere())
}

/// Get the index of image which should be displayed for a given sun position.
fn current_image_index_from_sun_pos(
    solar_items: &[SolarItem],
    sun_pos: &Position,
    hemisphere: &Hemisphere,
) -> Result<usize> {
    Ok(current_item_solar_from_sun_pos(solar_items, &sun_pos, hemisphere)?.index)
}

/// Get the solar item which should be displayed for a given sun position.
/// Sun position is expected in degrees!
fn current_item_solar_from_sun_pos<'i>(
    solar_items: &'i [SolarItem],
    sun_pos: &Position,
    hemisphere: &Hemisphere,
) -> Result<&'i SolarItem> {
    let (min_alt_item, max_alt_item) = get_minmax_alt_items(solar_items)?;
    let sorted_items = sort_solar_items(solar_items);
    let current_phase_items: Vec<&SolarItem> = match is_rising(sun_pos.azimuth, hemisphere) {
        true => get_items_between(&sorted_items, min_alt_item, max_alt_item),
        false => get_items_between(&sorted_items, max_alt_item, min_alt_item),
    };
    let current_altitude = not_nan!(sun_pos.altitude);
    let current_item = current_phase_items
        .iter()
        .min_by_key(|item| not_nan!((current_altitude - item.altitude).abs()))
        .with_context(|| format!("no solar items to choose from"))?;
    Ok(current_item)
}

/// Get items with lowest and highest altitude.
fn get_minmax_alt_items(solar_items: &[SolarItem]) -> Result<(&SolarItem, &SolarItem)> {
    match solar_items.iter().minmax_by_key(|item| item.altitude) {
        itertools::MinMaxResult::MinMax(min, max) => Ok((min, max)),
        _ => Err(anyhow!("no solar items to choose from")),
    }
}

/// Get all items between 'first' and 'last', inclusive.
/// Items are cycled so we can wrap around the end and start from the beginning again.
fn get_items_between<'i>(
    solar_items: &[&'i SolarItem],
    first: &SolarItem,
    last: &SolarItem,
) -> Vec<&'i SolarItem> {
    let mut starting_from_first = solar_items
        .iter()
        .cycle()
        .skip_while(|item| ***item != *first)
        .peekable();
    let mut items_between = starting_from_first
        .peeking_take_while(|item| ***item != *last)
        .map(|item| *item)
        .collect_vec();
    items_between.push(*starting_from_first.next().unwrap());
    items_between
}

/// Check whether given sun azimuth corresponds with rising or setting sun position
/// on given hemisphere.
fn is_rising(azimuth: f64, hemishphere: &Hemisphere) -> bool {
    match hemishphere {
        Hemisphere::Northern => azimuth <= 180.0,
        Hemisphere::Southern => azimuth > 180.0,
    }
}

/// Get indices of images in appearance order.
pub fn get_image_order_solar(solar_items: &[SolarItem]) -> Vec<usize> {
    sort_solar_items(solar_items)
        .iter()
        .map(|item| item.index)
        .collect_vec()
}

/// Sort solar items by their occurrence order in a day.
pub fn sort_solar_items(solar_items: &[SolarItem]) -> Vec<&SolarItem> {
    // We assume Northern Hemisphere and just sort by azimuth value.
    // There is no localization metadata in images so I don't see other option.
    solar_items
        .iter()
        .sorted_by_key(|item| item.azimuth)
        .collect_vec()
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    #[fixture]
    #[rustfmt::skip]
    fn solar_items_1() -> Vec<SolarItem> {
        // -50, -10, 10, 80, 30, -60, intentionally unordered
        vec![
            SolarItem { index: 2, azimuth: not_nan!(100.0), altitude: not_nan!(10.0) },
            SolarItem { index: 0, azimuth: not_nan!(30.0), altitude: not_nan!(-50.0) },
            SolarItem { index: 1, azimuth: not_nan!(50.0), altitude: not_nan!(-10.0) },
            SolarItem { index: 3, azimuth: not_nan!(190.0), altitude: not_nan!(80.0) },
            SolarItem { index: 5, azimuth: not_nan!(350.0), altitude: not_nan!(-60.0) },
            SolarItem { index: 4, azimuth: not_nan!(250.0), altitude: not_nan!(30.0) },
        ]
    }

    #[fixture]
    #[rustfmt::skip]
    fn solar_items_2() -> Vec<SolarItem> {
        vec![
            SolarItem { index: 0, azimuth: not_nan!(100.0), altitude: not_nan!(-50.0) },
            SolarItem { index: 1, azimuth: not_nan!(250.0), altitude: not_nan!(-44.0) },
        ]
    }

    #[rstest]
    #[case(Position { azimuth: 100.0, altitude: -70.0 }, 5)] // wrap around to last item
    #[case(Position { azimuth: 100.0, altitude: -58.0 }, 5)] // wrap around to last item
    #[case(Position { azimuth: 100.0, altitude: -54.0 }, 0)]
    #[case(Position { azimuth: 100.0, altitude: -45.0 }, 0)]
    #[case(Position { azimuth: 100.0, altitude: -31.0 }, 0)]
    #[case(Position { azimuth: 100.0, altitude: -29.0 }, 1)]
    #[case(Position { azimuth: 100.0, altitude: -10.0 }, 1)]
    #[case(Position { azimuth: 100.0, altitude: 01.0 }, 2)]
    #[case(Position { azimuth: 100.0, altitude: 70.0 }, 3)]
    #[case(Position { azimuth: 170.0, altitude: 80.0 }, 3)] // peak value before noon
    #[case(Position { azimuth: 200.0, altitude: 80.0 }, 3)] // peak value after noon
    #[case(Position { azimuth: 250.0, altitude: 70.0 }, 3)]
    #[case(Position { azimuth: 250.0, altitude: 40.0 }, 4)]
    #[case(Position { azimuth: 250.0, altitude: 00.0 }, 4)]
    #[case(Position { azimuth: 250.0, altitude: -50.0 }, 5)]
    #[case(Position { azimuth: 250.0, altitude: -70.0 }, 5)]
    fn test_current_item_solar_from_sun_pos_1(
        solar_items_1: Vec<SolarItem>,
        #[case] sun_pos: Position,
        #[case] expected_index: usize,
    ) {
        let result =
            current_image_index_from_sun_pos(&solar_items_1, &sun_pos, &Hemisphere::Northern);
        assert_eq!(result.unwrap(), expected_index);
    }

    #[rstest]
    #[case(Position { azimuth: 100.0, altitude: -60.0 }, 0)]
    #[case(Position { azimuth: 100.0, altitude: -40.0 }, 1)]
    #[case(Position { azimuth: 250.0, altitude: -40.0 }, 1)]
    #[case(Position { azimuth: 250.0, altitude: -60.0 }, 0)] // wrap around to first item
    fn test_current_item_solar_from_sun_pos_2(
        solar_items_2: Vec<SolarItem>,
        #[case] sun_pos: Position,
        #[case] expected_index: usize,
    ) {
        let result =
            current_image_index_from_sun_pos(&solar_items_2, &sun_pos, &Hemisphere::Northern);
        assert_eq!(result.unwrap(), expected_index);
    }

    #[rstest]
    fn test_sort_solar_items(solar_items_1: Vec<SolarItem>) {
        let result = get_image_order_solar(&solar_items_1);
        assert_eq!(result, vec![0, 1, 2, 3, 4, 5]);
    }
}
