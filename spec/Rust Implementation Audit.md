# Rust Implementation Audit

## Scope

- Repository: `rocketman510/Robo-Pope`
- Branch: `rewrite/rust-red-tests`
- Contract: `spec/Page API.md`
- Method: Static review of the Rust source tree, cross-checked against the focused FIRST suites in `src/tests/`.
- This document is an implementation truth table, not a GREEN completion claim.

## Current state

The branch has working Page/Modal definition registries, static Components V2 rendering, MemoryStore insert/resolve behavior, validation, and a basic Frame planner.

The end-to-end interaction path remains incomplete: Page and Modal registration do not populate ExecutionRegistry, Ctx response methods are not wired to Discord HTTP, Modal non-Text inputs are incomplete, pagination controls and persisted navigation state are missing, nested paginated Frames fail, and no production durable Store exists. Database and Store ownership have been separated at the contract boundary. Direct closure conversion is now accepted by the value conversion layer.

## Finding-to-test map

| Finding | Required contract | RED coverage |
|---|---|---|
| T-01 callback registration | Every Page/Modal execution is indexed before routing | `execution_wiring.rs` |
| T-02 Ctx HTTP | Each response method sends its distinct Discord operation | `ctx_http.rs` |
| T-03 gateway handler | InteractionCreate reaches App dispatch | `execution_wiring.rs` |
| T-04 / G-01 Database vs Store | Host DB and framework Store are independent | `database_contract.rs` |
| T-05 durability | Production Store survives process recreation | `database_contract.rs`, `store.rs` |
| T-06 Modal rendering/transaction | All declared inputs render; failed render leaves no record | `modal_contract.rs` |
| T-07 typed Modal values | Entity IDs, Mentionables, and file metadata are preserved | `modal_contract.rs` |
| T-08 navigation | Controls, state persistence, atomic updates, and bounds work | `pagination_contract.rs`, `store.rs` |
| T-09 nested Frames | Nested Frame splits in-place within Container limits | `pagination_contract.rs` |
| T-10 dynamic budget | Resolved dynamic values are included in planning | `pagination_contract.rs` |
| T-11 nested keys | Every interactive descendant has stable identity | `pagination_contract.rs` |
| T-12 closure ergonomics | Direct async closures need no `dynamic(...)` wrapper | `value.rs` |
| T-13 disabled options | Unsupported per-option disabled state is rejected | `selection_contract.rs` |
| T-14 Modal limits | Text ranges and option counts are enforced | `modal_contract.rs` |
| T-15 global IDs | Page and Modal IDs share one namespace | `execution_wiring.rs` |

## Contract decisions

The former open questions are now explicit in `Page API.md`:

1. Registration indexes all Page/Modal callbacks in a process-local execution registry; the gateway handler routes InteractionCreate through App dispatch.
2. Modal rendering persists its submit record only after successful resolution, validation, and payload construction.
3. Host database access uses `Database::get::<T>()`; framework persistence is a separate Store dependency.
4. Production Store durability is required; MemoryStore is test/development only.
5. String Select per-option disabled state is rejected because Discord does not support it; whole-menu disabled remains supported.
6. Direct asynchronous property closures are accepted; `dynamic(...)` is optional.
7. One paginated Frame may appear at the top level or inside a Container. Nested pagination preserves the Container and its static siblings. Multiple paginated Frames in one Page tree are rejected.
8. Frame navigation uses reserved Previous/Next execution IDs and atomically compare-and-sets persisted state.
9. Modal Serde mapping for each supported submitted value category is specified.

## Test discipline

Run `cargo test --no-run` first. Fix compilation and fixture defects before interpreting runtime failures. Then run `cargo test --all-targets` and retain the failing output as the RED baseline.

Do not weaken assertions to make the suite pass. Address one finding per GREEN change, rerun the complete suite, and repeat the not-implemented/cockroach sweep.
