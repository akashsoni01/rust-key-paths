# rust-elm Roadmap

Evolution toward [TCA](https://github.com/pointfreeco/swift-composable-architecture) parity.

## Done (v0.1.0 foundation)

- **New crate:** `rust-elm/` workspace member
- Core engine: `Cmd`, `Effect`, `Sub`, `Program`, `Runtime`, `Bus`, `Environment`
- Composition stub: `Slot`, `lift`
- Testing: `TestRuntime`, `ReplayHarness`, `assert_effect!`
- Macros: `total_update!`, `arbitrary_msg!`
- Key paths (phase 2): `optics` module + `keypath` prelude, `wrap_action`/`extract`, smoke tests
- Reducers (phase 3): `Reducer` trait, `Reduce`, `CombineReducers`, `reducers!`, `ReducerProgram`, `Runtime::from_reducer_program`
- Identified collections (phase 5): `Identifiable`, `IdentifiedVec` with serde round-trip
- Scope combinators (phase 4): `ScopeReducer`, `IfLetReducer`, `IfCaseLetReducer`, `ForEachReducer`, `OptionalReducer`, `lift_cmd` / `lift_cmd_with_id`
- Effect cancel (partial 6.1): `Effect::cancel(id)` interpreted in `Runtime`

## Next

- Phase 6–11: effects parity (debounce/throttle/run), dependencies, store, test store, shared state, polish

See [`todo.md`](../todo.md) for the full checklist.
