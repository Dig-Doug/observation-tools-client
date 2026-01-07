use crate::context;
use crate::observation::ObservationBuilder;
use log::Level;
use log::Log;
use log::Metadata;
use log::Record;
use observation_tools_shared::ObservationType;

pub struct ObservationLogger;

impl ObservationLogger {
  pub fn init() -> Result<(), log::SetLoggerError> {
    Self::init_with_level(Level::Info)
  }

  pub fn init_with_level(max_level: Level) -> Result<(), log::SetLoggerError> {
    static LOGGER: ObservationLogger = ObservationLogger;
    log::set_logger(&LOGGER)?;
    log::set_max_level(max_level.to_level_filter());
    Ok(())
  }
}

impl Log for ObservationLogger {
  fn enabled(&self, metadata: &Metadata) -> bool {
    if metadata.target().starts_with("observation_tools::") {
      return false;
    }
    metadata.level() <= log::max_level()
  }

  fn log(&self, record: &Record) {
    if !self.enabled(record.metadata()) {
      return;
    }
    if context::get_current_execution().is_none() {
      return;
    }

    let builder = ObservationBuilder::new("ObservationLogger")
      .observation_type(ObservationType::LogEntry)
      .log_level(record.level().into())
      .label(format!("log/{}", record.target()));
    let builder = if let (Some(file), Some(line)) = (record.file(), record.line()) {
      builder.source(file, line)
    } else {
      builder
    };
    let _ = builder.payload(format!("{}", record.args())).build();
  }

  fn flush(&self) {
    // No buffering, so nothing to flush
  }
}
