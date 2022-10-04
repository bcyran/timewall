use std::cmp::Reverse;

use anyhow::{Context, Ok, Result};
use chrono::{DateTime, Local, NaiveTime, Timelike};
use itertools::Itertools;
use log::debug;
use sun::Position;

use crate::{
    geo::{Coords, Hemisphere},
    properties::{SolarItem, TimeItem},
};

const SECONDS_IN_A_DAY: u32 = 24 * 60 * 60;

/// Select index of image corresponding with the given time.
pub fn select_image_h24(time_items: &[TimeItem], time: &NaiveTime) -> Result<usize> {
    let day_progress = time.num_seconds_from_midnight() as f64 / SECONDS_IN_A_DAY as f64;
    let sorted_time_items = sort_time_items(time_items);
    // Find the last item with time lower or equal to current.
    // If no such item, fall back to the overall last item.
    // This represents the situation when the first item doesn't have 00:00 timestamp,
    // we want to use the last one from the "previous" day.
    let maybe_current_item = sorted_time_items
        .iter()
        .rfind(|item| f64::from(item.time) <= day_progress)
        .or(sorted_time_items.last());
    let current_item =
        maybe_current_item.with_context(|| format!("no time items to choose from"))?;
    Ok(current_item.index)
}

/// Get indices of images in their occurrence order for time items.
pub fn get_image_order_h24(time_items: &[TimeItem]) -> Vec<usize> {
    sort_time_items(time_items)
        .iter()
        .map(|item| item.index)
        .collect_vec()
}

/// Sort time items by their time of occurrence.
pub fn sort_time_items(time_items: &[TimeItem]) -> Vec<&TimeItem> {
    time_items
        .iter()
        .sorted_by_key(|item| item.time)
        .collect_vec()
}

/// Select index of image corresponding with the sun position for given datetime and coordinates.
pub fn select_image_solar(
    solar_items: &[SolarItem],
    datetime: &DateTime<Local>,
    coords: &Coords,
) -> Result<usize> {
    let sun_pos = sun::pos(datetime.timestamp_millis(), coords.lat, coords.lon);
    let sun_pos = Position {
        azimuth: sun_pos.azimuth.to_degrees(),
        altitude: sun_pos.altitude.to_degrees(),
    };
    debug!("sun pos: {sun_pos:?}");
    select_image_solar_from_sun_pos(solar_items, &sun_pos, &coords.hemishphere())
}

/// Select index of image corresponding with sun position on the specified hemisphere.
fn select_image_solar_from_sun_pos(
    solar_items: &[SolarItem],
    sun_pos: &Position,
    hemishphere: &Hemisphere,
) -> Result<usize> {
    let (rising_items, setting_items) = sort_solar_items(solar_items);
    let maybe_current_item = if is_rising(sun_pos.azimuth, hemishphere) {
        // If the sun is currently rising, get last item with altitude lower than the current.
        // If there's no such items, fall back to the last one from setting items.
        // If no setting items, fall back to the last of rising items.
        rising_items
            .iter()
            .rfind(|item| f64::from(item.altitude) <= sun_pos.altitude)
            .or(setting_items.last())
            .or(rising_items.last())
    } else {
        // If the sun is currently setting, get last item with altitude higher than the current.
        // If there's no such items, fall back to the last one from rising items.
        // If no rising items, fall back to the last of setting items.
        setting_items
            .iter()
            .rfind(|item| f64::from(item.altitude) >= sun_pos.altitude)
            .or(rising_items.last())
            .or(setting_items.last())
    };
    let current_item =
        maybe_current_item.with_context(|| format!("no solar items to choose from"))?;
    Ok(current_item.index)
}

/// Get indices of images in their occurrence order for solar items.
pub fn get_image_order_solar(solar_items: &[SolarItem]) -> Vec<usize> {
    let (rising_items, setting_items) = sort_solar_items(solar_items);
    rising_items
        .iter()
        .chain(setting_items.iter())
        .map(|item| item.index)
        .collect_vec()
}

/// Split collection of solar items (sun positions) into rising and setting items.
/// Sort both collections in the natural occurrence order.
/// Sun altitude increases while it rises and decreases when it's setting.
/// We assume Northen hemisphere for sun coordinates from metadata.
pub fn sort_solar_items<'i>(items: &'i [SolarItem]) -> (Vec<&'i SolarItem>, Vec<&'i SolarItem>) {
    // XXX: Should the hemisphere (coordinates) be taken from image EXIF?
    let (mut rising_items, mut setting_items): (Vec<_>, Vec<_>) = items
        .iter()
        .partition(|item| is_rising(f64::from(item.azimuth), &Hemisphere::Northern));
    rising_items.sort_by_key(|item| item.altitude);
    setting_items.sort_by_key(|item| Reverse(item.altitude));
    (rising_items, setting_items)
}

/// Check whether given sun azimuth corresponds with rising or setting sun position
/// on given hemisphere.
fn is_rising(azimuth: f64, hemishphere: &Hemisphere) -> bool {
    match hemishphere {
        Hemisphere::Northern => azimuth <= 180.0,
        Hemisphere::Southern => azimuth > 180.0,
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;
    use crate::properties::{SolarItem, TimeItem};

    #[fixture]
    #[rustfmt::skip]
    fn time_items() -> Vec<TimeItem> {
        // intentionally unordered
        vec![
            TimeItem { index: 0, time: not_nan!(0.25) },
            TimeItem { index: 2, time: not_nan!(0.75) },
            TimeItem { index: 1, time: not_nan!(0.5) },
        ]
    }

    #[fixture]
    #[rustfmt::skip]
    fn solar_items_rising() -> Vec<SolarItem> {
        vec![
            SolarItem { index: 1, azimuth: not_nan!(100.0), altitude: not_nan!(01.0) },
            SolarItem { index: 0, azimuth: not_nan!(45.0), altitude: not_nan!(-58.0) },
            SolarItem { index: 2, azimuth: not_nan!(170.0), altitude: not_nan!(65.0) },
        ]
    }

    #[fixture]
    #[rustfmt::skip]
    fn solar_items_setting() -> Vec<SolarItem> {
        vec![
            SolarItem { index: 4, azimuth: not_nan!(300.0), altitude: not_nan!(-45.0) },
            SolarItem { index: 3, azimuth: not_nan!(250.0), altitude: not_nan!(01.0) },
        ]
    }

    #[fixture]
    fn solar_items(
        solar_items_rising: Vec<SolarItem>,
        solar_items_setting: Vec<SolarItem>,
    ) -> Vec<SolarItem> {
        [solar_items_rising, solar_items_setting].concat()
    }

    #[rstest]
    #[case("00:00:00", 2)]
    #[case("00:20:00", 2)]
    #[case("05:59:59", 2)]
    #[case("06:00:00", 0)]
    #[case("11:59:00", 0)]
    #[case("12:00:00", 1)]
    #[case("15:00:00", 1)]
    #[case("18:00:00", 2)]
    #[case("23:59:59", 2)]
    fn test_select_image_h24(
        time_items: Vec<TimeItem>,
        #[case] time: NaiveTime,
        #[case] expected_result: usize,
    ) {
        let result = select_image_h24(&time_items, &time);
        assert_eq!(result.unwrap(), expected_result);
    }

    #[rstest]
    fn test_get_image_order_h24(time_items: Vec<TimeItem>) {
        let result = get_image_order_h24(&time_items);
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[rstest]
    #[case(Position { azimuth: 30.0, altitude: -68.0 }, 4)]
    #[case(Position { azimuth: 50.0, altitude: -50.0 }, 0)]
    #[case(Position { azimuth: 99.0, altitude: 00.0 }, 0)]
    #[case(Position { azimuth: 101.0, altitude: 02.0 }, 1)]
    #[case(Position { azimuth: 171.0, altitude: 66.0 }, 2)]
    #[case(Position { azimuth: 200.0, altitude: 55.0 }, 2)]
    #[case(Position { azimuth: 251.0, altitude: 00.0 }, 3)]
    #[case(Position { azimuth: 301.0, altitude: -50.0 }, 4)]
    fn test_select_image_solar_from_sun_pos_northern_hemi(
        solar_items: Vec<SolarItem>,
        #[case] sun_pos: Position,
        #[case] expected_result: usize,
    ) {
        let result = select_image_solar_from_sun_pos(&solar_items, &sun_pos, &Hemisphere::Northern);
        assert_eq!(result.unwrap(), expected_result);
    }

    #[rstest]
    #[case(Position { azimuth: 30.0, altitude: -68.0 }, 2)]
    #[case(Position { azimuth: 50.0, altitude: -50.0 }, 0)]
    #[case(Position { azimuth: 101.0, altitude: 02.0 }, 1)]
    #[case(Position { azimuth: 171.0, altitude: 66.0 }, 2)]
    #[case(Position { azimuth: 200.0, altitude: 55.0 }, 2)]
    #[case(Position { azimuth: 251.0, altitude: 00.0 }, 2)]
    #[case(Position { azimuth: 301.0, altitude: -50.0 }, 2)]
    fn test_select_image_solar_from_sun_pos_no_setting(
        solar_items_rising: Vec<SolarItem>,
        #[case] sun_pos: Position,
        #[case] expected_result: usize,
    ) {
        let result =
            select_image_solar_from_sun_pos(&solar_items_rising, &sun_pos, &Hemisphere::Northern);
        assert_eq!(result.unwrap(), expected_result);
    }

    #[rstest]
    fn test_get_image_order_solar(solar_items: Vec<SolarItem>) {
        let result = get_image_order_solar(&solar_items);
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }
}
