# rust-elm Roadmap


## Done (v0.1.0 foundation)

- **New crate:** `rust-elm/` workspace member
- Core engine: `Cmd`, `Effect`, `Sub`, `Program`, `Runtime`, `Bus`, `Environment`
- Composition stub: `Slot`, `lift`
- Testing: `TestRuntime`, `ReplayHarness`, `assert_effect!`
- Macros: `total_update!`, `arbitrary_msg!`
- Key paths (phase 2): `optics` module + `keypath` prelude, `wrap_action`/`extract`, smoke tests
- Reducers (phase 3): `Reducer` trait, `Reduce`, `CombineReducers`, `reducers!`, `ReducerProgram`, `Runtime::from_reducer_program`
- Identified collections (phase 5): [`rust_identified_vec`](../rust_identified_vec) — `Identifiable`, `IdentifiedVec` with serde round-trip
- Scope combinators (phase 4): `ScopeReducer`, `IfLetReducer`, `IfCaseLetReducer`, `ForEachReducer`, `OptionalReducer`, `lift_cmd` / `lift_cmd_with_id`
- Effect cancel (partial 6.1): `Effect::cancel(id)` interpreted in `Runtime`
- Effects parity (phase 6): `debounce`, `throttle`, `from_run`/`RunSender`, `cancellable_with`, `result_task`, `task_try`
- Dependencies (phase 7): `DependencyValues`, `DependencyKey`, `Environment::live`/`test`, built-in Clock/Uuid/Now/Rng, `Effect::provide_dependency`
- Store (phase 8): `Store`, `StoreTask`, `ScopedStore`, `StateSubscriber`, `Runtime::store()`, ping-based state subscription, in-flight work tracking for `send().finish()`
- Safe reducer (phase 8b): `safe_reducer` module — `safe_reduce_update`, `safe_reduce_rollback`, `SafeReduceError`; `CatchReducer`, `RollbackCatchReducer`
- Test store (phase 9): `ExhaustiveTestStore`, `send_with`, `receive`/`receive_timeout`, `finish`, `with_exhaustivity(false)`
- Shared state (phase 10): `Shared<T>`, `Storage`/`InMemoryStorage`/`FileStorage`, `ReplayHarness::snapshot`/`restore`, `StateSnapshot`

## Next

- Phase 11: polish

See [`todo.md`](../todo.md) for the full checklist.
