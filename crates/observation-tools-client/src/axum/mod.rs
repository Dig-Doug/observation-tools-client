//! Axum middleware layers for observation-tools integration
//!
//! This module provides Tower layers for integrating observation-tools
//! with Axum web applications.
//!
//! # Example
//!
//! ```rust,ignore
//! use axum::{routing::get, Router};
//! use observation_tools_client::{Client, ClientBuilder};
//! use observation_tools_client::axum::{ExecutionLayer, RequestObserverLayer};
//!
//! let client = ClientBuilder::new()
//!     .base_url("http://localhost:3000")
//!     .build()
//!     .unwrap();
//!
//! let app = Router::new()
//!     .route("/", get(handler))
//!     // RequestObserverLayer must come before ExecutionLayer (as an inner layer)
//!     // because it depends on the execution context that ExecutionLayer creates
//!     .layer(RequestObserverLayer::new())
//!     .layer(ExecutionLayer::new(client));
//! ```

mod execution_layer;
mod request_observer;

pub use execution_layer::ExecutionLayer;
pub use execution_layer::ExecutionService;
pub use request_observer::RequestObserverConfig;
pub use request_observer::RequestObserverLayer;
pub use request_observer::RequestObserverService;
