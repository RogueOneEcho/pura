use rogue_logging::Verbosity::*;
use rogue_logging::{Logger, LoggerBuilder};
use std::sync::Arc;

#[must_use]
pub fn init_logging() -> Arc<Logger> {
    LoggerBuilder::new()
        .with_exclude_filter("reqwest".to_owned())
        .with_exclude_filter("cookie".to_owned())
        .with_exclude_filter("html5ever".to_owned())
        .with_exclude_filter("lofty".to_owned())
        .with_exclude_filter("selectors".to_owned())
        .with_verbosity(Debug)
        .create()
}
