//! X API client foundation
//!
//! This module provides common HTTP client infrastructure for X API communication,
//! including base URL configuration, authentication header handling, and response
//! error classification.

mod client;
mod error;

pub use client::{HttpXApiClient, XApiClient};
pub use error::{classify_response_error, parse_retry_after};
