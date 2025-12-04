//! Web UI handlers

mod execution_detail;
mod executions_list;
mod index;
mod observation_detail;
mod templates;

pub use execution_detail::execution_detail_log;
pub use execution_detail::execution_detail_payload;
pub use executions_list::list_executions;
pub use index::index;
pub use observation_detail::observation_detail;
pub use templates::init_templates;
