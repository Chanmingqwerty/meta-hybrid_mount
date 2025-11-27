use std::path::Path;
use anyhow::Result;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::prelude::*;
use tracing_subscriber::fmt::FormatFields;

struct SimpleFormatter;

impl<S, N> fmt::FormatEvent<S, N> for SimpleFormatter
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> fmt::FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &fmt::FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let metadata = event.metadata();
        let level = metadata.level();
        let target = metadata.target();

        // Match original log format: [LEVEL] [Target] Message
        write!(writer, "[{}] [{}] ", level, target)?;

        ctx.format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

pub fn init(verbose: bool, log_path: &Path) -> Result<()> {
    let level = if verbose { "debug" } else { "info" };
    
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Using standard blocking file append to ensure logs persist on exit
    let file_appender = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .event_format(SimpleFormatter);

    let filter = EnvFilter::new(level);

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .init();

    // Capture standard log crate macros
    tracing_log::LogTracer::init()?;

    Ok(())
}
