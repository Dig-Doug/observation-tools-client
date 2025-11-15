use crate::context;
use crate::observation::ObservationBuilder;
use log::{Level, Log, Metadata, Record};
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
    if metadata.target().starts_with("observation_tools_client::") {
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

    let mut builder = ObservationBuilder::new("ObservationLogger")
      .observation_type(ObservationType::LogEntry)
      .log_level(record.level().into())
      .label(format!("log/{}", record.target()))
      .payload(&format!("{}", record.args()));
    if let (Some(file), Some(line)) = (record.file(), record.line()) {
      builder = builder.source(file, line);
    }
    let _ = builder.build();
  }

  fn flush(&self) {
    // No buffering, so nothing to flush
  }
}
