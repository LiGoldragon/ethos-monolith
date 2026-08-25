//! Reads signal Ethos text and emits `signal.rs` for wire consumers.
//!
//! Consumers commit the emitted Rust; they never depend on this crate at
//! runtime. The generator is invoked by an explicit update step that writes
//! the checked signal artifact before the consumer build checks it.

pub mod build;
pub mod fixture;
pub mod generate;
