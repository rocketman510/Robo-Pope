//! Application database facade: framework persistence plus bot-owned services.
//!
//! The framework needs a small persistence interface for execution routing,
//! but it must not prescribe how the bot stores users, levels, tickets, or
//! other domain data. Database<E> combines the framework Store with an
//! application extension E. This keeps Page callbacks ergonomic:
//!
//!     ctx.app.db.users.query(user_id).await?
//!
//! Store persists framework execution references. E contains the
//! application's own repositories/services. Deref (or an equally ergonomic
//! accessor) may expose E without making framework code depend on its types.
//!
//! The in-memory Store is a development/test default only. The App constructor
//! used in production should accept a durable Store implementation.
//!
//! TODO: Define Database<E> { store: Arc<dyn Store>, extension: E }.
//! TODO: Expose E through Deref or a clear accessor without leaking Store.
//! TODO: Keep domain-specific database traits outside this framework module.
