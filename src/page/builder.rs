//! Page blueprint, builder methods, registration, and registered handle.
//!
//! Page<T> describes message structure, not a particular sent message. It
//! holds an ordered sequence of top-level components and Frame regions plus
//! the one generic data type shared by dynamic values and executions.
//!
//! Intended lifecycle:
//! Page<T>::new(id)
//!     -> add_text / add_container / add_button_row / add_frame / ...
//!     -> register(&App)
//!     -> RegisteredPage<T>
//!     -> render(&App, data)
//!     -> RenderedMessage (or a paginated set)
//!
//! Registration consumes Page<T>, validates statically knowable invariants
//! (unique execution IDs, reserved IDs, fixed structure), and produces
//! RegisteredPage<T>. Dynamic values and Frame contents are validated at
//! render time because their resolved values do not exist during registration.
//! Rendering is unavailable on the unregistered builder.
//!
//! Builder calls append distinct components in order. Each add_text call
//! adds its own Text Display; adjacent text is not concatenated. Containers
//! compose supported child components and Frames. One Page method accepts a
//! selection-menu abstraction; the menu type determines its Discord kind.
//!
//! Fixed-size counts belong in the public API. A button row has five slots;
//! a Section has three Text Display slots. Do not replace fixed slots with
//! arbitrary Vec values followed by avoidable runtime checks. Slot
//! conversions must permit static values, dynamic values where supported,
//! and absence. Each independently typed slot must remain independently
//! generic so static and dynamic closure types can coexist in one call.
//!
//! Prefer borrowing Page data in render callbacks instead of imposing Clone
//! on every T. T must still satisfy the serialization and concurrency bounds
//! needed by asynchronous rendering.
//!
//! Intended type shape (illustrative):
//! \`\`\`rust,ignore
//! pub struct Page<T, E = ()> {
//!     id: PageId,
//!     items: Vec<PageItem<T, E>>,
//!     pagination: PaginationPolicy,
//! }
//! enum PageItem<T, E> {
//!     Component(Component<T, E>),
//!     Frame(Frame<T, E>),
//! }
//! pub struct RegisteredPage<T, E = ()> {
//!     definition: Arc<PageDefinition<T, E>>,
//! }
//! \`\`\`
//!
//! TODO: Define Page<T>, PageItem<T>, PageData<T>, and RegisteredPage<T>.
//! TODO: Add methods for each component, select menu, Frame, and pagination.
//! TODO: Make registration consume Page<T>; registered definitions immutable.
//! TODO: Preserve builder order through rendering and pagination.
//! TODO: Decide and document whether an empty Page is valid.
