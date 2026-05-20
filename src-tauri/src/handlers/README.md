# Adding a New Format Handler

Each format handler lives in its own sub-module under `src-tauri/src/handlers/`.

## Steps

1. Create `handlers/<name>/mod.rs` with a struct that implements `FormatHandler` (from `handlers/mod.rs`).
2. Implement all trait methods. At minimum `detect()` must return a meaningful confidence score.
3. In `lib.rs → run()`, after constructing the `FormatHandlerRegistry`, call:
   ```rust
   registry.register(Arc::new(YourHandler::new()));
   ```
   The registry stores `Arc<dyn FormatHandler>` so you can share state inside your struct.

## Contract

- `id()` must be globally unique (e.g. `"unity"`, `"unreal"`, `"pe"`).
- `detect()` must **never panic** and should return quickly (no heavy I/O).
- `open()` may be slow; emit progress events via `ctx.app.emit(...)`.
- `tree()` / `preview()` / `export()` may spawn sidecars via `ctx.sidecar`.
- Return `AppError::NotImplemented` for unimplemented optional operations.

## Existing Handlers

| Handler | Phase | Status |
|---------|-------|--------|
| unity   | 05    | pending |
