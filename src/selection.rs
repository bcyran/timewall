use chrono::{DateTime, Local, NaiveTime, Timelike};
use itertools::Itertools;
use sun::Position;

use crate::{
    geo::{Coords, Hemisphere},
    properties::{SolarItem, WallpaperPropertiesH24, WallpaperPropertiesSolar},
};

const SECONDS_IN_A_DAY: u32 = 24 * 60 * 60;

/// Select index of image corresponding with the given time.
pub fn select_image_h24(properties: &WallpaperPropertiesH24, time: &NaiveTime) -> usize {
    let day_progress = time.num_seconds_from_midnight() as f64 / SECONDS_IN_A_DAY as f64;
    let sorted_time_items = properties
        .time_info
        .iter()
        .sorted_by(|a, b| a.time.partial_cmp(&b.time).unwrap())
        .collect_vec();
    // Find the last item with time lower or equal to current.
    // If no such item, fall back to the overall last item.
    // This represents the situation when the first item doesn't have 00:00 timestamp,
    // we want to use the last one from the "previous" day.
    sorted_time_items
        .iter()
        .rfind(|item| item.time <= day_progress)
        .unwrap_or(sorted_time_items.last().unwrap())
        .index
}

/// Select index of image corresponding with the sun position for given datetime and coordinates.
pub fn select_image_solar(
    properties: &WallpaperPropertiesSolar,
    datetime: &DateTime<Local>,
    coords: &Coords,
) -> usize {
    let sun_pos = sun::pos(datetime.timestamp_millis(), coords.lat, coords.lon);
    // Both values are supposed to be in radians but it looks like only azimuth actually is?
    // Let's ensure both are degrees before passing the position further.
    let sun_pos = Position {
        azimuth: sun_pos.azimuth.to_degrees(),
        altitude: sun_pos.altitude,
    };
    select_image_solar_from_sun_pos(properties, &sun_pos, &coords.hemishphere())
}

/// Select index of image corresponding with sun position on the specified hemisphere.
fn select_image_solar_from_sun_pos(
    properties: &WallpaperPropertiesSolar,
    sun_pos: &Position,
    hemishphere: &Hemisphere,
) -> usize {
    let (rising_items, setting_items) = sort_solar_items(&properties.solar_info, hemishphere);
    let current_item = if is_rising(sun_pos.azimuth, hemishphere) {
        // If the sun is currently rising, get last item with altitude lower than the current.
        // If there's no such items, fall back to the last one from setting items.
        rising_items
            .iter()
            .rfind(|item| item.altitude <= sun_pos.altitude)
            .unwrap_or(setting_items.last().unwrap())
    } else {
        // If the sun is currently setting, get last item with altitude higher than the current.
        // If there's no such items, fall back to the last one from rising items.
        setting_items
            .iter()
            .rfind(|item| item.altitude >= sun_pos.altitude)
            .unwrap_or(rising_items.last().unwrap())
    };
    current_item.index
}

/// Split collection of solar items (sun positions) into rising and setting items.
/// Sort both collections in the natural occurrence order.
/// Sun altitude increases while it rises and decreases when it's setting.
fn sort_solar_items<'i>(
    items: &'i Vec<SolarItem>,
    hemishphere: &Hemisphere,
) -> (Vec<&'i SolarItem>, Vec<&'i SolarItem>) {
    let (mut rising_items, mut setting_items): (Vec<_>, Vec<_>) = items
        .iter()
        .partition(|item| is_rising(item.azimuth, hemishphere));
    rising_items.sort_by(|a, b| a.altitude.partial_cmp(&b.altitude).unwrap());
    setting_items.sort_by(|a, b| a.altitude.partial_cmp(&b.altitude).unwrap().reverse());
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
    fn props_24h() -> WallpaperPropertiesH24 {
        WallpaperPropertiesH24 {
            appearance: None,
            time_info: vec![
                // intentionally unordered
                TimeItem {
                    index: 0,
                    time: 0.25,
                },
                TimeItem {
                    index: 2,
                    time: 0.75,
                },
                TimeItem {
                    index: 1,
                    time: 0.5,
                },
            ],
        }
    }

    #[rstest]
    #[case(NaiveTime::from_hms(0, 0, 0), 2)]
    #[case(NaiveTime::from_hms(0, 20, 0), 2)]
    #[case(NaiveTime::from_hms(5, 59, 59), 2)]
    #[case(NaiveTime::from_hms(6, 00, 00), 0)]
    #[case(NaiveTime::from_hms(11, 59, 00), 0)]
    #[case(NaiveTime::from_hms(12, 00, 00), 1)]
    #[case(NaiveTime::from_hms(15, 00, 00), 1)]
    #[case(NaiveTime::from_hms(18, 00, 00), 2)]
    #[case(NaiveTime::from_hms(23, 59, 59), 2)]
    fn test_select_image_h24(
        props_24h: WallpaperPropertiesH24,
        #[case] time: NaiveTime,
        #[case] expected_result: usize,
    ) {
        let result = select_image_h24(&props_24h, &time);
        assert_eq!(result, expected_result);
    }

    #[fixture]
    fn props_solar() -> WallpaperPropertiesSolar {
        WallpaperPropertiesSolar {
            appearance: None,
            solar_info: vec![
                // intentionally unordered
                SolarItem {
                    index: 0,
                    azimuth: 45.0,
                    altitude: -0.58,
                },
                SolarItem {
                    index: 4,
                    azimuth: 300.0,
                    altitude: -0.45,
                },
                SolarItem {
                    index: 1,
                    azimuth: 100.0,
                    altitude: 0.01,
                },
                SolarItem {
                    index: 2,
                    azimuth: 170.0,
                    altitude: 0.65,
                },
                SolarItem {
                    index: 3,
                    azimuth: 250.0,
                    altitude: 0.01,
                },
            ],
        }
    }

    #[rstest]
    #[case(Position { azimuth: 30.0, altitude: -0.68 }, 4)]
    #[case(Position { azimuth: 50.0, altitude: -0.50 }, 0)]
    #[case(Position { azimuth: 99.0, altitude: 0.0 }, 0)]
    #[case(Position { azimuth: 101.0, altitude: 0.02 }, 1)]
    #[case(Position { azimuth: 171.0, altitude: 0.66 }, 2)]
    #[case(Position { azimuth: 200.0, altitude: 0.55 }, 2)]
    #[case(Position { azimuth: 251.0, altitude: 0.0 }, 3)]
    #[case(Position { azimuth: 301.0, altitude: -0.50 }, 4)]
    fn test_select_image_solar_from_sun_pos_northern_hemi(
        props_solar: WallpaperPropertiesSolar,
        #[case] sun_pos: Position,
        #[case] expected_result: usize,
    ) {
        let result = select_image_solar_from_sun_pos(&props_solar, &sun_pos, &Hemisphere::Northern);
        assert_eq!(result, expected_result);
    }
}
