//! Robo-Pope framework library.
//!
//! The public API is split into App ownership/routing and declarative Page UI.
//! Keeping the library separate from the bot binary allows integration tests
//! and other applications to use the framework without starting a gateway.
pub mod app;
pub mod page;

#[cfg(test)]
mod tests;
