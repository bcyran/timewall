use chrono::{NaiveTime, Timelike};
use sun::Position;

use crate::properties::{WallpaperPropertiesH24, WallpaperPropertiesSolar};

const SECONDS_IN_A_DAY: u32 = 24 * 60 * 60;

pub fn select_image_h24(properties: &WallpaperPropertiesH24, time: &NaiveTime) -> usize {
    let day_progress = time.num_seconds_from_midnight() as f32 / SECONDS_IN_A_DAY as f32;
    let mut sorted_time_items = properties.time_info.clone();
    sorted_time_items.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    // item with greatest time value lower than current day progress
    let matching_today_item = sorted_time_items
        .iter()
        .filter(|item| item.time.partial_cmp(&day_progress).unwrap().is_le())
        .last();
    // if missing then get the item with greatest time value overall
    // (last from the "previous" day)
    let curent_item = matching_today_item.unwrap_or(sorted_time_items.last().unwrap());
    curent_item.index
}

pub fn select_image_solar(properties: &WallpaperPropertiesSolar, sun_pos: &Position) -> usize {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::properties::TimeItem;
    use rstest::*;

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
}
