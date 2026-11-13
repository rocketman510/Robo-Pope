//! Interactive Button.
//!
//! A Button has a developer execution ID, label, style, disabled state, and
//! explicit execution callback. Label/style/disabled are display properties
//! and accept static or dynamic values. The callback is always explicit.
//!
//! Rendering derives a custom ID from definition identity, execution
//! identity, serialized instance data, and optional Frame element identity.
//! The render pipeline collects and persists the reference. A Button inside
//! a Section, Container, or Frame follows identical routing rules.
//!
//! Validate label length after dynamic resolution. Disabled state does not
//! erase the execution definition. Authorization remains the callback's job.
//!
//! TODO: Define Button<T, E>, ButtonStyle mapping, and Exec<T, E>.
//! TODO: Support dynamic label, style, disabled state.
//! TODO: Collect execution identity without serializing closures.
