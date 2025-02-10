//! Logging interface for `tvm_loader` crates.
//!
//! Provides an interface intended to output information to the user in a usable format during
//! `tvm` initialization.

use core::sync::atomic::{AtomicU8, Ordering};

unsafe extern "Rust" {
    static LOGGER: &'static dyn Log;
}

/// Global control for filtering unnecessary log messages.
static GLOBAL_FILTER: AtomicLevelFilter = AtomicLevelFilter::new(LevelFilter::Trace);

/// Defines the global [`Log`] impelementation.
#[macro_export]
macro_rules! unsafe_global_logger {
    ($log:expr) => {
        #[unsafe(no_mangle)]
        static LOGGER: &'static dyn $crate::logging::Log = &$log;
    };
}

/// Returns the globally defined [`Log`] implementation.
pub fn logger() -> &'static dyn Log {
    // SAFETY:
    //
    // All `tvm_loader` crates are required to implement [`Log`] correctly and define this symbol.
    unsafe { LOGGER }
}

/// Returns the currently active global [`LevelFilter`].
pub fn level_filter() -> LevelFilter {
    GLOBAL_FILTER.load(Ordering::Relaxed)
}

/// Sets the currently active [`LevelFilter`] to `filter`.
pub fn set_level_filter(filter: LevelFilter) {
    GLOBAL_FILTER.store(filter, Ordering::Relaxed)
}

/// Logging interface for `tvm_loader`.
pub trait Log: Sync {
    /// Logs the provided [`Message`].
    fn log(&self, message: &Message);
}

/// The context of a [`Log`] message.
pub struct Message<'a> {
    control: ControlData<'a>,
    arguments: core::fmt::Arguments<'a>,
}

impl<'a> Message<'a> {
    /// The name of the target of the log message.
    pub const fn target(&self) -> &'a str {
        self.control.target()
    }

    /// The [`Level`] of the [`Message`].
    pub const fn level(&self) -> Level {
        self.control.level()
    }

    /// The safely precompiled version of the log message and its arguments.
    pub const fn args(&self) -> core::fmt::Arguments<'a> {
        self.arguments
    }
}

/// Data that controls the intensity and target of the associated [`Message`].
pub struct ControlData<'a> {
    target: &'a str,
    level: Level,
}

impl<'a> ControlData<'a> {
    /// The name of the target of the [`ControlData`].
    pub const fn target(&self) -> &'a str {
        self.target
    }

    /// The [`Level`] of the [`ControlData`].
    pub const fn level(&self) -> Level {
        self.level
    }
}

/// An enum representing the various verbosity levels of a [`Log`] implementor.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// The most verbose logging level, these logs should contain very low priority information.
    Trace = 0,
    /// Designates low priority information.
    Debug = 1,
    /// Designates useful information.
    Info = 2,
    /// Designates potentially problematic occurrences.
    Warn = 3,
    /// Designates serious errors.
    Error = 4,
}

impl Level {
    /// Converts [`Level`] to its corresponding [`LevelFilter`], where a corresponding
    /// [`LevelFilter`] lets through logs of its level and higher.
    pub fn to_level_filter(&self) -> LevelFilter {
        AtomicLevelFilter::to_level_filter(*self as u8)
    }

    /// Returns the representation of [`Level`] as a [`str`].
    pub fn as_str(&self) -> &'static str {
        self.to_level_filter().as_str()
    }
}

impl PartialEq<LevelFilter> for Level {
    fn eq(&self, other: &LevelFilter) -> bool {
        *self as usize == *other as usize
    }
}

impl PartialOrd<LevelFilter> for Level {
    fn partial_cmp(&self, other: &LevelFilter) -> Option<core::cmp::Ordering> {
        Some((*self as usize).cmp(&(*other as usize)))
    }
}

impl core::fmt::Display for Level {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.pad(self.as_str())
    }
}

/// An enum representing the various verbosity filtering levels of a [`Log`] implementor.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LevelFilter {
    /// Filters all [`Level`]s lower than [`Level::Trace`].
    Trace = 0,
    /// Filters all [`Level`]s lower than [`Level::Debug`].
    Debug = 1,
    /// Filters all [`Level`]s lower than [`Level::Info`].
    Info = 2,
    /// Filters all [`Level`]s lower than [`Level::Warn`].
    Warn = 3,
    /// Filters all [`Level`]s lower than [`Level::Error`]
    Error = 4,
    /// Filters all [`Level`]s.
    Off = 5,
}

impl LevelFilter {
    /// Converts a [`LevelFilter`] to its corresponding [`Level`].
    ///
    /// Returns [`None`] if [`LevelFilter`] is [`LevelFilter::Off`].
    pub fn to_level(&self) -> Option<Level> {
        match self {
            LevelFilter::Trace => Some(Level::Trace),
            LevelFilter::Debug => Some(Level::Debug),
            LevelFilter::Info => Some(Level::Info),
            LevelFilter::Warn => Some(Level::Warn),
            LevelFilter::Error => Some(Level::Error),
            LevelFilter::Off => None,
        }
    }

    /// Returns the representation of [`LevelFilter`] as a [`str`].
    pub fn as_str(&self) -> &'static str {
        ["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "OFF"][*self as usize]
    }

    /// Returns whether the given [`Level`] should be logged if this [`LevelFilter`] is active.
    pub fn should_allow(&self, level: Level) -> bool {
        self.to_level()
            .map(|level_filter| level >= level_filter)
            .unwrap_or(false)
    }
}

impl core::fmt::Display for LevelFilter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.pad(self.as_str())
    }
}

/// A safe API for interacting with a [`LevelFilter`] that can be safely shared between threads and
/// mutated across threads.
pub struct AtomicLevelFilter(AtomicU8);

impl AtomicLevelFilter {
    /// Creates a new [`AtomicLevelFilter`].
    pub const fn new(level_filter: LevelFilter) -> Self {
        Self(AtomicU8::new(level_filter as u8))
    }

    /// Loads a [`LevelFilter`] from the [`AtomicLevelFilter`].
    ///
    /// [`load()`][sl] takes an [`Ordering`] argument which describes the memory ordering of this
    /// operation. Possible values are [`SeqCst`][sc], [`Acquire`][a], and [`Relaxed`][r].
    ///
    ///
    /// [sl]: Self::load
    /// [sc]: Ordering::SeqCst
    /// [a]: Ordering::Acquire
    /// [r]: Ordering::Relaxed
    pub fn load(&self, order: Ordering) -> LevelFilter {
        Self::to_level_filter(self.0.load(order))
    }

    /// Stores a [`LevelFilter`] into the [`AtomicLevelFilter`].
    ///
    /// [`store()`][s] takes an [`Ordering`] argument which describes the memory ordering of this
    /// operation. Possible values are [`SeqCst`][sc], [`Release`][rs], and [`Relaxed`][rx].
    ///
    ///
    /// [s]: Self::store
    /// [sc]: Ordering::SeqCst
    /// [rs]: Ordering::Release
    /// [rx]: Ordering::Relaxed
    pub fn store(&self, val: LevelFilter, order: Ordering) {
        self.0.store(val as u8, order)
    }

    #[inline(always)]
    fn to_level_filter(val: u8) -> LevelFilter {
        match val {
            0 => LevelFilter::Trace,
            1 => LevelFilter::Debug,
            2 => LevelFilter::Info,
            3 => LevelFilter::Warn,
            4 => LevelFilter::Error,
            5 => LevelFilter::Off,
            _ => unreachable!(),
        }
    }
}

#[doc(hidden)]
pub fn __log_impl(arguments: core::fmt::Arguments, level: Level, target: &str) {
    let control = ControlData { level, target };
    let record = Message { control, arguments };

    logger().log(&record)
}

/// Generic logging macro.
#[macro_export]
macro_rules! log {
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => ({
        let lvl = $lvl;
        if $crate::logging::level_filter().should_allow(lvl) {
            $crate::logging::__log_impl(::core::format_args!($($arg)+), lvl, $target)
        }
    });
    ($lvl:expr, $($arg:tt)+) => ($crate::log!(target: ::core::module_path!(), $lvl, $($arg)+));
}

/// Logs the provided message at [`Level::Trace`].
#[macro_export]
macro_rules! log_trace {
    (target: $target:expr, $($arg:tt)+) => ($crate::log!(target: $target, $crate::logging::Level::Trace, $($arg)+));
    ($($arg:tt)+) => ($crate::log!($crate::logging::Level::Trace, $($arg)+));
}

/// Logs the provided message at [`Level::Debug`].
#[macro_export]
macro_rules! log_debug {
    (target: $target:expr, $($arg:tt)+) => ($crate::log!(target: $target, $crate::logging::Level::Debug, $($arg)+));
    ($($arg:tt)+) => ($crate::log!($crate::logging::Level::Debug, $($arg)+));
}

/// Logs the provided message at [`Level::Info`].
#[macro_export]
macro_rules! log_info {
    (target: $target:expr, $($arg:tt)+) => ($crate::log!(target: $target, $crate::logging::Level::Info, $($arg)+));
    ($($arg:tt)+) => ($crate::log!($crate::logging::Level::Info, $($arg)+));
}

/// Logs the provided message at [`Level::Warn`].
#[macro_export]
macro_rules! log_warn {
    (target: $target:expr, $($arg:tt)+) => ($crate::log!(target: $target, $crate::logging::Level::Warn, $($arg)+));
    ($($arg:tt)+) => ($crate::log!($crate::logging::Level::Warn, $($arg)+));
}

/// Logs the provided message at [`Level::Error`].
#[macro_export]
macro_rules! log_error {
    (target: $target:expr, $($arg:tt)+) => ($crate::log!(target: $target, $crate::logging::Level::Error, $($arg)+));
    ($($arg:tt)+) => ($crate::log!($crate::logging::Level::Error, $($arg)+));
}
