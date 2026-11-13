//! Explicit interactive executions attached to components.
//!
//! Display and behavior are separate. Dynamic values determine what a
//! component looks like during rendering. Execution callbacks determine what
//! happens after interaction. Only executions are registered/persisted.
//!
//! Button and selection callbacks receive Ctx plus original Page data.
//! Selection callbacks additionally receive typed interaction data:
//! String values or resolved user/role/channel/mentionable entities. These
//! values are not merged into serialized Page data.
//!
//! Every execution ID is unique within its definition. Registration rejects
//! duplicates and reserves a framework-only namespace for pagination
//! controls. Frame elements additionally require stable developer-defined
//! keys. Identity must not depend on temporary vector position.
//!
//! Closures are never serialized. Registered definitions own callbacks;
//! Store records contain only stable references and serialized data.
//!
//! TODO: Define Exec<T, E, Selection> and callback adapter contracts.
//! TODO: Keep callbacks explicit in Button, Select, and Modal APIs.
//! TODO: Validate duplicate IDs at registration and dynamic Frame resolution.
//! TODO: Define Selection<S> and entity-choice structures.
//! TODO: A valid custom ID never grants authorization.
