//! Text Display component.
//!
//! Text Display presents Markdown. Each Page::add_text or
//! Container::add_text call creates a separate component; text is never
//! silently concatenated.
//!
//! Content is static or an asynchronous dynamic value. Validate resolved
//! content against the per-Text-Display 4,000-character maximum and the
//! message-wide 4,000-character Text Display budget. Count using one
//! documented convention. Descriptions and unrelated fields do not consume
//! this Text Display-only budget.
//!
//! An oversized Text Display is an error unless a deliberate splitting
//! strategy is introduced. Pagination may split between complete Text
//! Displays, never silently truncate one.
//!
//! TODO: Define Text<T, E> with Value<String, T, E> content.
//! TODO: Resolve with RenderCtx and Page data.
//! TODO: Contribute correct component and Text Display character costs.
