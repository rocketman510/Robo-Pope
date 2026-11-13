//! One representation for values that are static or resolved at render time.
//!
//! A display property accepts its ordinary Rust value or an asynchronous
//! callback returning that value. Developers do not need separate methods
//! such as add_static_text and add_dynamic_text.
//!
//! Conceptual representation:
//! Value<V, T, E> = Static(V) | Dynamic(async RenderCtx + Page data -> Result<V>)
//!
//! .add_text("Hello")
//! .add_text(dynamic_callback)
//!
//! Into conversions are central to the ergonomics. Strings, styles, URLs,
//! booleans, option labels/descriptions, and supported settings enter one
//! internal Value representation. Dynamic callbacks run during rendering,
//! receive RenderCtx and shared Page data, and do not create executions.
//!
//! Fixed slots add an Empty state. Each slot independently accepts a static
//! value, dynamic callback, or None. Section slots need separate generic
//! parameters so static and dynamic callback closure types can differ.
//! A single [impl Into<Slot<_>>; N] array has one element type and cannot by
//! itself express arbitrary heterogeneous Into inputs; do not design away
//! the mixed-slot use case.
//!
//! A small closure adapter may be necessary for Rust inference/coherence,
//! but it must not become separate public methods for static and dynamic
//! content. RED-phase compile-time tests must prove the chosen API supports
//! mixed slots and inferred None without awkward annotations.
//!
//! Intended type shape (illustrative):
//! \`\`\`rust,ignore
//! enum Value<V, T, E> {
//!     Static(V),
//!     Dynamic(DynamicValue<V, T, E>),
//! }
//! enum Slot<V, T, E> { Empty, Value(Value<V, T, E>) }
//! \`\`\`
//!
//! TODO: Define Value<V, T, E>, Slot<V, T, E>, and async callback types.
//! TODO: Define Into conversions for static values and dynamic callback
//!       adapters, including independently typed fixed slots.
//! TODO: Avoid requiring Clone on T solely to invoke a dynamic callback.
//! TODO: Keep render-time resolution separate from execution registration.
