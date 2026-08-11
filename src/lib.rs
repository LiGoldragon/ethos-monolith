//! Reads Ethos text, emits Rust (signal.rs, nexus.rs, sema.rs) per component.
//!
//! Consumers commit the emitted Rust; they never depend on this crate at
//! runtime. The generator is invoked by an explicit update step that writes
//! all three artifacts atomically before the component build checks them.

pub mod build;
pub mod generate;
