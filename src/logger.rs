macro_rules! debug_log {
    ($($arg:tt)*) => (
        if cfg!(debug_assertions) {
            log::info!($($arg)*)
        }
    )
}
