# Compile fixes applied on rewrite/rust-red-tests

## Already on branch

`src/app/router.rs` uses `custom_id.to_string()` for Serenity `FixedString`.

## If contract_audit.rs still fails to compile after pull

The interaction fixtures must use normal JSON quotes inside `r#"..."#`, not `\"`.

Also avoid `{result:?}` on `RenderedMessage` (no `Debug`) and avoid `.clone()` on it (no `Clone` on the outer message in some builds — use `into_parts()` once).

T-02 tests should expect `Err` from Ctx methods, not `should_panic` on `todo!()`.

Pull latest and if `contract_audit.rs` still has `\"type\"` in fixtures, replace with `"type"` (run):

```bash
sed -i 's/\\"/"/g' src/tests/contract_audit.rs
```

Then `cargo test` should **compile**. Many contract tests will still **fail** (RED) until GREEN finishes.
