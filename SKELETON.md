# Components V2 framework skeleton

Target repository: `rocketman510/Robo-Pope`
Target ref resolved from the supplied URL: `rewrite/rust`
Target source directories: `src/app/` and `src/page/`

This is the structure/documentation phase only. No RED-phase tests,
implementation bodies, Cargo changes, or bot integration are included.
The Rust files contain contracts, intended type shapes, and TODOs.

## Layout

- `src/app/`: App ownership, contexts, typed registries, interaction routing,
  persistence, and shared errors.
- `src/page/`: Page builder/registration, static/dynamic values, execution
  callbacks, rendering, validation, and public module organization.
- `src/page/component/`: Text, Section, Container, Separator, Thumbnail,
  Media Gallery, File, Button, Link Button, and button rows.
- `src/page/select/`: String and entity selection menus.
- `src/page/frame/`: heterogeneous Elements, stable Frame keys, and pagination.
- `src/page/modal/`: Modal inputs and typed submission handling.
- `src/page/limits.rs`: the exact numerical limits in the supplied spec.

## Prototype lessons carried into the contracts

- Mixed static/dynamic fixed slots must be expressible in one API.
- Dynamic Button properties must not be rejected by ButtonRow.
- Frame identities use stable logical keys, never current vector indices.
- Duplicate generated executions must be rejected, not silently overwritten.
- Container Frame output retains its declared position among static children.
- Nested component limits and Text Display character limits are counted
  separately and semantically.
- File display and upload are separate operations with an explicit handoff.
- In-memory Store is for development/testing; restart persistence requires a
  durable backend.
- Invalid IDs and invalid submissions produce errors, never fabricated
  placeholder values or panics.
- Rust compile-time ergonomics (heterogeneous `Into` slots and inferred
  `None`) are explicit RED-phase test targets.

The next step is RED TDD: write tests that prove the API contracts before
filling in implementation bodies.
