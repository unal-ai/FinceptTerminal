# Agent Guidelines for FinceptTerminal

FinceptTerminal Web: A visually stunning way to track markets on the *Big Screen*. Pro-level data, browser-based ease, and a layout built for high-stakes demos. 

This is a mod project based on FinceptTerminal. You MUST write what-why-how as code comments.

## ⚠️ CORE PRINCIPLE

**NEVER submit changes without verifying they work.**

Before completing any task or merge:
1. Run `cargo check` (or appropriate build command) to verify compilation
2. If tests exist, run them
3. If you don't know how to verify, **ASK** before submitting

If you cannot verify (e.g., missing dependencies, environment issues), explicitly state this limitation to the user rather than submitting untested changes.

---

## Critical Rules for Code Modifications

### 1. Merge Conflict Resolution

When resolving merge conflicts:

1. **NEVER take one side and blindly add from the other** - Always analyze what changes each side makes and understand the full context:
   - Check function signatures that may have changed
   - Check imports that may be required by one branch but not the other
   - Check Cargo.toml for feature flags that may have been added

2. **Verify consistency after resolution** - After resolving conflicts, ensure:
   - All function calls match the function's actual signature
   - All imports are present for symbols used in the code
   - Any new feature dependencies are added to Cargo.toml

3. **Test compilation before committing** - Always run `cargo check` (or equivalent build command) before committing merge conflict resolutions:
   ```bash
   cd fincept-terminal-desktop/src-tauri
   cargo check
   ```

### 2. Function Signature Changes

When adding code that calls functions:

1. **Always verify the function signature first** using `view_file` or `view_code_item`
2. **If a function needs state/context**, ensure the caller provides it
3. **Update imports** when adding new type references

### 3. Rust-Specific Rules

1. **Feature Flags** - When using optional crate features (e.g., `axum::extract::ws`), verify the feature is enabled in `Cargo.toml`:
   ```toml
   axum = { version = "0.7", features = ["ws"] }
   ```

2. **State Management** - When dispatching with state:
   - Ensure the dispatcher function accepts `Arc<ServerState>`
   - Import required types: `use std::sync::Arc;`
   - Import state types from the types module

### 4. Pre-Commit Checklist

Before committing any changes, verify:

- [ ] `cargo check` passes (no compilation errors)
- [ ] All function signatures match their call sites
- [ ] All required imports are present
- [ ] Feature flags are enabled in Cargo.toml for optional features
- [ ] (For Web) fincept-terminal-desktop/run_web.sh works? Load webpage and see its console log.

### 5. Merge Workflow

The correct workflow for merging branches:

```bash
# 1. Fetch latest
git fetch --all

# 2. Checkout the target branch
git checkout main && git pull

# 3. Merge (or rebase)
git merge feature-branch

# 4. If conflicts: resolve carefully, checking both sides fully

# 5. ALWAYS verify before committing
cargo check

# 6. Only then commit and push
git add .
git commit -m "Merge description"
git push
```

## Common Mistakes to Avoid

1. **Partial conflict resolution** - Taking code from one branch without its required supporting changes (imports, signatures, dependencies)

2. **Assuming function signatures haven't changed** - Always verify what a function expects

3. **Missing feature flags** - Using crate features without enabling them

4. **Not testing compilation** - Always `cargo check` before committing Rust changes
