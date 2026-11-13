//! Suggested implementation sequence after the RED phase.
//!
//! The order follows the supplied specification and limits how many moving
//! parts need to exist before a contract can be exercised:
//!
//! 1. Page builder, IDs, typed registration, immutable RegisteredPage.
//! 2. Static/dynamic Value resolution and mixed-slot conversion ergonomics.
//! 3. Components V2 composition and fixed-size Section/ButtonRow slots.
//! 4. Button execution identity, Store persistence, and component routing.
//! 5. String/entity selection menus and typed selection routing.
//! 6. Modal registration, input rendering, and submission routing.
//! 7. Heterogeneous Frame generation and stable generated-element identity.
//! 8. Frame budget planning, pagination, and automatic navigation.
//! 9. Typed Modal submission deserialization through Serde.
//! 10. Later: streaming/paginated Frame data sources and large-data
//!     pagination optimizations.
//!
//! This sequence is not an excuse to postpone validation contracts. Discord
//! limits, error behavior, and identity invariants should be represented in
//! RED tests before their implementation phase begins.
//!
//! The immediate next step is test-only: add API/compile-time tests and
//! behavioral tests that fail against these unimplemented contracts. Do not
//! implement the green phase in the same change as the RED suite.
//!
//! TODO: Keep this sequence aligned with the test suite and agreed decisions.
