//! Planned implementation sequence after the API skeleton and RED phase.
//!
//! 1. Page builder, IDs, typed registration, immutable RegisteredPage.
//! 2. Static/dynamic Value resolution and fixed-slot conversion ergonomics.
//! 3. Components V2 composition, validation, and Serenity rendering.
//! 4. Button execution IDs, anyhow context chains, Store persistence, routing.
//! 5. String/entity selection menus and typed interaction payloads.
//! 6. Modal registration, inputs, submission routing, and typed Serde forms.
//! 7. Heterogeneous Frame generation and stable developer keys.
//! 8. Component-budget planning, pagination state, automatic navigation.
//! 9. Additional Frame data-source pagination and optimizations.
//!
//! This file defines order only. RED tests should verify the API before
//! behavior is implemented. Discord limits, identity rules, and anyhow
//! error-context contracts must not be postponed until the green phase.
//! Multiple/nested Frame pagination policy and Section's bare-None inference
//! must be settled by explicit compile-time and behavioral tests.
