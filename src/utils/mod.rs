//! Logging and metadata helpers that the rest of the crate leans on.
//!
//! [`info_logger`] sets up console logging, [`activity_log`] records what the
//! system did to a rotating file on disk, and [`metadata`] decodes a transform's
//! untyped metadata map into a typed struct.

pub mod activity_log;
pub mod info_logger;
pub mod metadata;
