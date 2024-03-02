use chrono::{NaiveTime, Timelike};

const SECONDS_IN_A_DAY: u32 = 24 * 60 * 60;

pub fn time_to_day_fraction(time: NaiveTime) -> f64 {
    f64::from(time.num_seconds_from_midnight()) / f64::from(SECONDS_IN_A_DAY)
}

pub fn day_fraction_to_time(day_fraction: f64) -> NaiveTime {
    assert!(day_fraction <= 1.0);
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let seconds_passed = (day_fraction * f64::from(SECONDS_IN_A_DAY)) as u32;
    NaiveTime::from_num_seconds_from_midnight_opt(seconds_passed, 0).unwrap()
}
