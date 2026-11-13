//! String Select and its options.
//!
//! A String Select has execution ID, placeholder, options, selection bounds,
//! disabled state, and explicit callback. Option labels/descriptions and the
//! placeholder may be dynamic; submitted option values are stable data.
//! Support at most 25 options.
//!
//! Selected values are returned as Vec<String> or typed Selection<Vec<String>>
//! and are not added to serialized Page data. Validate option count,
//! labels/descriptions, placeholder, and min/max after dynamic resolution.
//!
//! TODO: Define StringSelect<T, E> and SelectOption<T, E>.
//! TODO: Resolve dynamic option presentation and route selected values.
