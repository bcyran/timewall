use chrono::NaiveTime;

const SECONDS_IN_A_DAY: u32 = 24 * 60 * 60;

pub fn day_fraction_to_time(day_fraction: f64) -> NaiveTime {
    assert!(day_fraction <= 1.0);
    let seconds_passed = (day_fraction * SECONDS_IN_A_DAY as f64) as u32;
    NaiveTime::from_num_seconds_from_midnight(seconds_passed, 0)
}
