#[macro_export]
macro_rules! not_nan {
    ( $l:literal ) => {
        ordered_float::NotNan::new($l).unwrap()
    };
}
