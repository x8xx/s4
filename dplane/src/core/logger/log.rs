macro_rules! log {
    ($fmt:expr) => {
        crate::core::logger::lib::write::write_log!($fmt);
    };
    ($fmt:expr, $($arg:tt)*) => {
        crate::core::logger::lib::write::write_log!($fmt, $($arg)*);
    };
}

macro_rules! debug_log {
    ($fmt:expr) => {
        #[cfg(feature="log-level-debug")]
        log!($fmt);
    };
    ($fmt:expr, $($arg:tt)*) => {
        #[cfg(feature="log-level-debug")]
        log!($fmt, $($arg)*);
    };
}

macro_rules! err_log {
    ($fmt:expr) => {
        write_err_log!!($fmt);
    };
    ($fmt:expr, $($arg:tt)*) => {
        write_err_log!!($fmt, $($arg)*);
    };
}

pub(crate) use log;
pub(crate) use debug_log;
pub(crate) use err_log;
