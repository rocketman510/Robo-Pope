# Rust Implementation Audit (revised)

## Scope

- **Repository:** rocketman510/Robo-Pope
- **Branch:** \`rewrite/rust-red-tests\` (evolved from \`rewrite/rust\`)
- **Contract source:** \`spec/Page API.md\`
- **Method:** Static review of every \`src/**/\*.rs\` file, cross-checked against the cockroach sweep (not-implemented markers, thin files, register vs execution storage, Database vs Store) and against the in-tree \`contract_audit\` RED suite.
- **What this document is:** An implementation truth table. It is not a GREEN plan and not a claim that GREEN is done.

Finding labels:

| Label | Meaning |
|-------|---------|
| **[TEST]** | A FIRST-style test in-tree (or specified below) fails today and proves the gap. |
| **[REVIEW]** | Needs code reading; a test can only sample the property. |
| **[GAP]** | Page API does not decide the behavior; needs a spec decision before GREEN. |
| **[SKELETON-OK]** | Thin file is intentional (constants, re-exports, tiny types). Not a cockroach. |
| **[COCKROACH]** | Looks like structure but the behavior path is empty, stubbed, or never wired. |

---

## Current state in one paragraph

There is real builder, registry, value, MemoryStore, validation, and Frame planner code. Registration of Page/Modal *definitions* works. Render of many static Components V2 surfaces works. **The end-to-end interaction path does not work:** callbacks accepted by builders are never inserted into \`ExecutionRegistry\` at \`register()\`, so \`dispatch_interaction\` cannot invoke them. Every \`Ctx\` Discord response method still returns "not yet wired." \`Database\` is still only a wrapper around the framework \`Store\`. Modal non-Text inputs still bail at render. That is why a bot can compile, a page can render, a button can show up, and a click does nothing useful.

GREEN work on this branch added: \`ExecutionRegistry\`, \`InteractionRouter\` that resolves Store + looks up the registry, \`App::dispatch_interaction\`, and handler wiring that calls dispatch. **It did not close the loop:** \`Page::register\` / \`Modal::register\` still do not call \`executions.insert(...)\`.

---

## Executive checklist

| Area | Status | Label |
|------|--------|-------|
| Page/Modal definition registries | Working | — |
| MemoryStore insert/resolve | Working (process-local only) | **[TEST]** T-05 for durability |
| Page render (static / many components) | Mostly working | — |
| Frame planner (simple text) | Working | — |
| \`Page::register\` → \`ExecutionRegistry::insert\` | **Missing** | **[COCKROACH][TEST]** |
| \`Modal::register\` → execution storage | **Missing** | **[COCKROACH][TEST]** |
| \`AppHandler\` → \`dispatch_interaction\` | Present on this branch | was T-03 |
| Router Store resolve + registry get | Present; get always misses | **[TEST]** T-01 |
| \`Ctx::{reply,defer,edit_*,followup,delete,open_modal}\` | Explicit not-wired \`bail!\` | **[COCKROACH][TEST]** T-02 |
| \`Database\` as host application DB | Name only; holds \`Store\` | **[COCKROACH][TEST]** T-04 / G-01 |
| Durable Store beyond MemoryStore | Absent | **[TEST]** T-05 |
| Modal non-Text render | Explicit not-implemented | **[TEST]** T-06 |
| Modal orphan Store on failed render | Persist-before-success | **[TEST]** T-06 orphan |
| Pagination nav controls + state persist | Not done | **[TEST]** T-08 |
| Nested Frame in Container split | Fails / over-limit path | **[TEST]** T-09 |
| Interactive key for Section/Container in Frame | Incomplete | **[TEST]** T-11 |
| Direct closure \`IntoValue\` ergonomics | Spec examples not matched | **[GAP]/[TEST]** T-12 |
| Disabled select option | Render fails | **[TEST]** T-13 |
| Modal max length / max options enforced as contract | Partially / wrong error | **[TEST]** T-14 |
| Cross-registry Page/Modal ID uniqueness | Not enforced | **[TEST]** T-15 |

---

## Thin files — classified

| File | Verdict |
|------|---------|
| \`src/lib.rs\`, \`src/app/error.rs\`, \`src/page/limits.rs\`, \`src/page/implementation_order.rs\` | **[SKELETON-OK]** |
| Thin component modules (\`text\`, \`file\`, \`thumbnail\`, etc.) | **[REVIEW]** small types, not automatically cockroaches |
| **\`src/app/database.rs\`** | **[COCKROACH]** Store wrapper only |
| \`src/app/handler.rs\` | **[REVIEW]** must keep dispatch wired |

\`Ok(())\` in tests and Store idempotent insert are mostly red herrings. Signal is explicit \`not yet\` markers and missing \`executions.insert\` on register.

---

## Critical path: why click does nothing

\`\`\`
Discord InteractionCreate
  → AppHandler
  → App::dispatch_interaction
  → InteractionRouter
  → Store.resolve(custom_id)          // works if render persisted a record
  → ExecutionRegistry.get(key)        // ALWAYS None today
  → Err("No process-local execution…")
\`\`\`

Root cause: \`Page::register\` / \`Modal::register\` validate and insert definitions but **never** call \`app.executions.insert(...)\` with type-erased developer closures.

**Deterministic FIRST proof:** After register, \`executions.contains(key)\` is false for each interactive id; dispatch cannot run the callback.

---

## Finding catalog (summary)

### T-01 — register does not populate ExecutionRegistry  **[COCKROACH][TEST]**
Files: \`builder.rs\` register, \`modal/mod.rs\` register, \`executions.rs\`, \`router.rs\`.

### T-02 — Ctx HTTP not wired  **[COCKROACH][TEST]**
All response methods \`bail!("…not yet wired…")\` in \`context.rs\`.

### T-03 — Handler dispatch  **[REVIEW][TEST]**
Must call \`dispatch_interaction\` (present on this branch; protect with test).

### T-04 / G-01 — Database is only Store  **[COCKROACH][TEST]**
\`database.rs\` is ~19 lines wrapping \`Arc<dyn Store>\`. No host repositories. Page API examples imply app DB.

### T-05 — MemoryStore only  **[TEST]**
No durable backend.

### T-06 — Modal non-Text + orphan Store  **[TEST]**
Explicit not-implemented; orphan record risk on persist-before-success.

### T-07 — Typed modal entity/file deserialize  **[TEST]**

### T-08 / T-09 — Pagination nav, state, nested Frame  **[TEST]**

### T-10 — Dynamic budget  **[TEST]**

### T-11 — Nested interactive keys  **[TEST]**

### T-12 — Direct closure ergonomics  **[GAP][TEST]**

### T-13 — Disabled SelectOption  **[TEST]**

### T-14 — Modal limits enforcement  **[TEST]**
Constants in \`limits.rs\` are fine; enforcement is not.

### T-15 — Cross-registry ID uniqueness  **[TEST]**

---

## Missing contracts (need spec decisions)

1. Host DB shape vs framework Store
2. Whether durable Store is required for v1
3. Disabled select options: allowed or reject at build
4. Direct closure IntoValue vs dynamic(...)
5. Nested Frame pagination policy inside Containers

---

## Recommended FIRST test modules

| Concern | Module | Property |
|---------|--------|----------|
| database.rs | \`tests/database_contract.rs\` | Store ≠ host app DB claim |
| register → executions | \`tests/execution_wiring.rs\` | contains(key) after register |
| context.rs | \`tests/ctx_http.rs\` | Http test double records calls |
| handler | \`tests/handler_dispatch.rs\` | InteractionCreate → dispatch |

---

## GREEN order (only after RED is correct)

1. register → executions.insert (fixes click does nothing)
2. Ctx + Http test double
3. Database/Store naming and host extension surface
4. Modal non-Text + limits
5. Pagination product behavior
6. Keys, disabled options, cross-registry IDs
7. Durable Store if required

---

## How far RED can go

In-process: registry membership, dispatch, Store lifecycle, render validation, planner budgets.

Not without a double: live Discord HTTP (use a test double for FIRST).

Agent workflow: one checklist row per PR; cargo test must flip that row; re-run cockroach sweep so new stubs cannot hide.

## Cockroach sweep

\`\`\`bash
rg -n --type rust 'todo!\(|unimplemented!|not (yet )?(implemented|wired|supported)' src/ || true
rg -n --type rust 'executions\.insert|ExecutionRegistry' src/page/ src/app/
sed -n '1,80p' src/app/database.rs
\`\`\`

*Revised after full-tree investigation. No claim that GREEN is complete.*
