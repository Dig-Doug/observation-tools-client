//! Execution context propagation

use crate::error::Result;
use crate::execution::ExecutionHandle;
use crate::Error;
use std::cell::RefCell;

thread_local! {
    static EXECUTION_CONTEXT: RefCell<Option<ExecutionHandle>> = RefCell::new(None);
}

/// Set the global execution for the current thread
pub(crate) fn set_global_execution(execution: ExecutionHandle) -> Result<()> {
    EXECUTION_CONTEXT.with(|ctx| {
        let mut ctx = ctx.borrow_mut();
        if ctx.is_some() {
            return Err(Error::GlobalExecutionAlreadyRegistered);
        }
        *ctx = Some(execution);
        Ok(())
    })
}

/// Get the current execution from context
pub(crate) fn get_current_execution() -> Option<ExecutionHandle> {
    EXECUTION_CONTEXT.with(|ctx| ctx.borrow().clone())
}
