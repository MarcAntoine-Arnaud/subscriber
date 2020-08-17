use std::{fmt, io, time::Instant};
use tracing::{
    field::{Field, Visit},
    Event, Level, Subscriber,
};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

#[derive(Debug, Default)]
struct FmtEventVisitor {
    message: String,
}

impl Visit for FmtEventVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        match field.name() {
            "message" => self.message = format!("{:?}", value),
            _ => {}
        }
    }
}

enum StandardOutput {
    Out(io::Stdout),
    Err(io::Stderr),
}

impl StandardOutput {
    fn new(level: &Level) -> Self {
        match *level {
            Level::ERROR | Level::WARN => Self::Err(io::stderr()),
            _ => Self::Out(io::stdout()),
        }
    }

    fn get_dyn_ref(&mut self) -> &mut dyn io::Write {
        match self {
            Self::Out(out) => out,
            Self::Err(err) => err,
        }
    }
}

/// Output messages to standard streams.
///
/// ERROR/WARN go to stderr.
/// All others to go to stdout.
pub struct FmtLayer {
    start: Instant,
}

impl FmtLayer {
    pub fn new() -> Self {
        FmtLayer {
            start: Instant::now(),
        }
    }
}

impl<S> Layer<S> for FmtLayer
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let now = Instant::now();
        let time = now - self.start;

        let mut visitor = FmtEventVisitor::default();
        event.record(&mut visitor);

        let mut span_string = String::new();
        for span in ctx.scope() {
            if !span_string.is_empty() {
                span_string.push_str(" | ");
            }
            span_string.push_str(span.name());
        }

        let metadata = event.metadata();
        let level = match *metadata.level() {
            Level::ERROR => "ERROR",
            Level::WARN => "WARN",
            Level::INFO => "INFO",
            Level::DEBUG => "DEBUG",
            Level::TRACE => "TRACE",
        };

        let module = metadata.module_path().unwrap_or("no module");

        let mut output = StandardOutput::new(metadata.level());
        let output_ref = output.get_dyn_ref();

        writeln!(
            output_ref,
            "[{:.6} {}]({})({}): {}",
            time.as_secs_f64(),
            level,
            span_string,
            module,
            visitor.message,
        )
        .unwrap();
    }
}
