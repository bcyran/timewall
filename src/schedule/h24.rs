use std::cmp::min;

use anyhow::{anyhow, Result};
use chrono::NaiveTime;
use itertools::Itertools;
use ordered_float::NotNan;

use super::time::time_to_day_fraction;
use crate::wallpaper::properties::TimeItem;

/// Get the image index from item which should be displayed at the given time.
pub fn current_image_index_h24(time_items: &[TimeItem], time: NaiveTime) -> Result<usize> {
    Ok(current_item_h24(time_items, time)?.index)
}

/// Get the time item which should be displayed at the given time.
fn current_item_h24(time_items: &[TimeItem], time: NaiveTime) -> Result<&TimeItem> {
    let current_time_fraction = not_nan!(time_to_day_fraction(time));
    time_items
        .iter()
        .min_by_key(|item| times_distance(item.time, current_time_fraction))
        .ok_or_else(|| anyhow!("no time items to choose from"))
}

/// Calculate distance between two times expressed as day fraction between 0 and 1.
/// This distance includes "wrapping" around the clock.
/// E.g. distance between 21:00 (0.875) and 3:00 (0.125) is 6 hours (0.25).
fn times_distance(a: NotNan<f64>, b: NotNan<f64>) -> NotNan<f64> {
    let first_distance = not_nan!((a - b).abs());
    let second_distance = not_nan!(1.0) - first_distance;
    min(first_distance, second_distance)
}

/// Get indices of images in appearance order.
pub fn get_image_index_order_h24(time_items: &[TimeItem]) -> Vec<usize> {
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

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    const HOUR_VALUE: f64 = 1.0 / 24.0;

    #[fixture]
    #[rustfmt::skip]
    fn time_items_1() -> Vec<TimeItem> {
        // 1:00, 4:00, 5:00, 12:00, 22:00, intentionally unordered
        vec![
            TimeItem { index: 2, time: not_nan!(05.0 * HOUR_VALUE) },
            TimeItem { index: 0, time: not_nan!(01.0 * HOUR_VALUE) },
            TimeItem { index: 1, time: not_nan!(04.0 * HOUR_VALUE) },
            TimeItem { index: 4, time: not_nan!(22.0 * HOUR_VALUE) },
            TimeItem { index: 3, time: not_nan!(12.0 * HOUR_VALUE) },
        ]
    }

    #[fixture]
    #[rustfmt::skip]
    fn time_items_2() -> Vec<TimeItem> {
        vec![
            TimeItem { index: 0, time: not_nan!(06.0 * HOUR_VALUE) },
            TimeItem { index: 1, time: not_nan!(23.0 * HOUR_VALUE) },
        ]
    }

    #[rstest]
    #[case("00:00:00", 0)]
    #[case("00:20:00", 0)]
    #[case("03:30:00", 1)]
    #[case("04:00:00", 1)]
    #[case("04:29:00", 1)]
    #[case("04:31:00", 2)]
    #[case("05:00:00", 2)]
    #[case("05:30:00", 2)]
    #[case("05:30:00", 2)]
    #[case("20:00:00", 4)]
    #[case("23:00:00", 4)]
    #[case("23:59:00", 0)] // wrap to after midnight
    fn test_current_image_index_h24_1(
        time_items_1: Vec<TimeItem>,
        #[case] time: NaiveTime,
        #[case] expected_index: usize,
    ) {
        let result = current_image_index_h24(&time_items_1, time);
        assert_eq!(result.unwrap(), expected_index);
    }

    #[rstest]
    #[case("01:00:00", 1)] // wrap to before midnight
    #[case("03:00:00", 0)]
    #[case("12:00:00", 0)]
    fn test_current_image_index_h24_2(
        time_items_2: Vec<TimeItem>,
        #[case] time: NaiveTime,
        #[case] expected_index: usize,
    ) {
        let result = current_image_index_h24(&time_items_2, time);
        assert_eq!(result.unwrap(), expected_index);
    }

    #[rstest]
    fn test_get_image_index_order_h24(time_items_1: Vec<TimeItem>) {
        let result = get_image_index_order_h24(&time_items_1);
        assert_eq!(result, &[0, 1, 2, 3, 4]);
    }
}
