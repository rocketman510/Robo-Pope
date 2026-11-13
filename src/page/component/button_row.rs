//! Fixed five-slot button Action Row.
//!
//! Discord Action Rows support one to five Buttons. Make the maximum explicit
//! with five fixed slots, not an arbitrary Vec followed by runtime count
//! checks. Unused slots are explicit. A Button owns its dynamic label/style/
//! disabled values and callback; the row must not reject a Button because a
//! property is dynamic.
//!
//! Slot conversion and None inference are compile-time contracts. The
//! prototype used tuple slots; the final API may use a fixed array only if
//! its element type remains ergonomic. RED-phase compile tests must prove
//! intended call sites compile without awkward annotations. Do not optimize
//! away heterogeneous Into conversion ergonomics.
//!
//! Empty rows are invalid. Each Button has its own execution identity.
//! Link Buttons may be used if Discord permits them in this row, but never
//! create execution records.
//!
//! TODO: Define ButtonRow<T, E> with exactly five slots.
//! TODO: Settle tuple-vs-array API using inference tests.
//! TODO: Resolve each Button normally and validate one-to-five populated slots.
