//! Heterogeneous Frame elements and stable developer-defined keys.
//!
//! Element is the common abstraction returned by a Frame callback. It lets
//! application code return different component types without converting to
//! Serenity builders.
//!
//! Interactive elements require stable keys chosen from application data,
//! such as a user ID or record key. The key follows the logical item when it
//! moves to another position or pagination slice. Framework identity combines
//! definition ID, Frame ID, element key, and component execution ID.
//!
//! Positional index is not stable identity. Duplicate keys must produce a
//! descriptive error rather than overwrite an earlier callback.
//!
//! Intended shape (illustrative):
//! \`\`\`rust,ignore
//! struct FrameElement<T, E> {
//!     stable_key: ElementKey,
//!     component: Component<T, E>,
//! }
//! enum Element<T, E> { /* ergonomic constructors for Component variants */ }
//! \`\`\`
//!
//! TODO: Define Element<T, E> constructors for supported Component types.
//! TODO: Define keyed FrameElement wrapper or equivalent API.
//! TODO: Recursively collect interactive descendants before persistence.
//! TODO: Define missing/duplicate key behavior.
