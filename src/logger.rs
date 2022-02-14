use crate::sync::SpinMutex;
use core::ops::Deref;
use core::fmt::Arguments;

static LOG_LEVEL: SpinMutex<LogLevel> = SpinMutex::new(LogLevel::Error);

pub fn set_log_level(level: LogLevel) {
    LOG_LEVEL.lock().set(level);
}

#[derive(Clone, Copy)]
pub enum LogLevel {
    Error = 0,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    fn set(&mut self, level: LogLevel) {
        *self = level;
    }
}

#[macro_export]
macro_rules! error {
    ($($args:tt)*) => (
        use $crate::logger::LogLevel;
        $crate::logger::_log(LogLevel::Error, file!(), line!(), format_args!($($args)*));
    );
}

#[macro_export]
macro_rules! warn {
    ($($args:tt)*) => (
        use $crate::logger::LogLevel;
        $crate::logger::_log(LogLevel::Warn, file!(), line!(), format_args!($($args)*));
    );
}

#[macro_export]
macro_rules! info {
    ($($args:tt)*) => (
        use $crate::logger::LogLevel;
        $crate::logger::_log(LogLevel::Info, file!(), line!(), format_args!($($args)*));
    );
}

#[macro_export]
macro_rules! debug {
    ($($args:tt)*) => (
        use $crate::logger::LogLevel;
        $crate::logger::_log(LogLevel::Debug, file!(), line!(), format_args!($($args)*));
    );
}

#[macro_export]
macro_rules! trace {
    ($($args:tt)*) => (
        use $crate::logger::LogLevel;
        $crate::logger::_log(LogLevel::Trace, file!(), line!(), format_args!($($args)*));
    );
}

pub fn _log(level: LogLevel, file: &str, line: u32, args: Arguments) {
    let global_level = *LOG_LEVEL.lock().deref() as u8;
    if level as u8 > global_level {
        return
    }

    let level = match level {
        LogLevel::Error => {"ERROR"},
        LogLevel::Warn => {"WARN"},
        LogLevel::Info => {"INFO"},
        LogLevel::Debug => {"DEBUG"},
        LogLevel::Trace => {"TRACE"},
    };
    use crate::kprintln;
    kprintln!("[{} ({}:{})] {}", level, file, line, args);
}










