#[macro_export]
macro_rules! not_nan {
    ( $l:expr ) => {
        ordered_float::NotNan::new($l).unwrap()
    };
}
