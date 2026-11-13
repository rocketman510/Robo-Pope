//! Frame pagination state, budget calculation, and automatic navigation.
//!
//! Framework-owned pagination means developers provide Frame data and opt in;
//! they do not author ordinary Previous/Next callbacks or custom IDs.
//!
//! Reserve budget for all static Page/Container content and navigation
//! controls before filling a page. Account for nested total components,
//! Text Display characters, Container direct children, Action Row limits,
//! Section structure, Gallery items, and every Discord restriction.
//!
//! Split only between complete components. A component too large for an
//! otherwise empty page returns an error unless it has an explicit splitting
//! strategy. Never truncate a Text Display or split an interactive component.
//!
//! Static components before and after a Frame remain on every generated page.
//! Container children retain declared order even when a Frame expands.
//! Persist original Page data and current Frame position separately from the
//! Frame collection/source. Initial implementation may regenerate the
//! collection during navigation; document that underlying data can change.
//! A later paginated data source should avoid loading huge collections.
//!
//! Navigation is framework-generated. Disable Previous/Next at boundaries
//! and reject out-of-range or stale navigation even if interaction arrives.
//!
//! TODO: Define PaginationState, planner, and Frame source/snapshot boundary.
//! TODO: Reserve static cost and compute semantic nested component budgets.
//! TODO: Persist/recover page index and validate navigation.
//! TODO: Define behavior for multiple and nested Frame regions before coding.
