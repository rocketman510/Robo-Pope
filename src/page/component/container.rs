//! Container: ordered child composition with accent color and spoiler state.
//!
//! A Container groups permitted Components V2 children. It has an optional
//! accent color and spoiler behavior, and may contain a Frame. Accent color
//! is a display property and follows the static-or-dynamic Value contract.
//! Discord limits direct Container children to ten.
//!
//! Children are ordered. A Frame declared between two static children must
//! resolve between those same children. Never gather all static children
//! first and append Frame output afterward.
//!
//! Container Frames fully participate in stable execution identity,
//! persistence, and routing. Interactive descendants must not be rendered
//! without registering their callbacks. Duplicate keys/IDs are errors, not
//! silent HashMap overwrites.
//!
//! Container and Page pagination share one component-budget model. If static
//! children consume all ten direct slots, return a clear error or use a
//! deliberate supported representation; never bypass the limit when
//! computed Frame capacity is zero.
//!
//! TODO: Define Container<T, E> and ordered ContainerChild.
//! TODO: Support permitted children, Frames, dynamic accent color, spoiler.
//! TODO: Preserve child order during resolution and pagination.
//! TODO: Count direct children separately from nested total components.
//! TODO: Register Frame-generated interactive executions recursively.
