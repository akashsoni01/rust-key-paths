use crate::cmd::Cmd;
use crate::effect::{run_registered_env_task, run_registered_task, Effect, EffectId};
use crate::optics::{Casepath, CasePath, StateLens};
use rust_identified_vec::{Identifiable, IdentifiedVec};
use crate::reducer::Reducer;
use std::marker::PhantomData;

/// Focuses nested state/action and runs a child reducer (TCA `Scope`).
#[derive(Debug)]
pub struct ScopeReducer<R, PS: 'static, PA: 'static, CS: 'static, CA: 'static, AK: 'static, SK>
where
    AK: Copy,
    SK: StateLens<PS, CS>,
{
    pub state_kp: SK,
    pub action_kp: AK,
    pub cancel_id: EffectId,
    pub child: R,
    _marker: PhantomData<(PA, CA, PS, CS)>,
}

impl<R, PS: 'static, PA: 'static, CS: 'static, CA: 'static, AK: 'static, SK>
    ScopeReducer<R, PS, PA, CS, CA, AK, SK>
where
    AK: Copy,
    SK: StateLens<PS, CS>,
{
    pub fn new(state_kp: SK, action_kp: AK, cancel_id: EffectId, child: R) -> Self {
        Self {
            state_kp,
            action_kp,
            cancel_id,
            child,
            _marker: PhantomData,
        }
    }
}

impl<R, PS: 'static, PA: 'static, CS: 'static, CA: 'static, AK: 'static, SK> Reducer
    for ScopeReducer<R, PS, PA, CS, CA, AK, SK>
where
    R: Reducer<State = CS, Action = CA>,
    PA: Send + 'static,
    CA: Copy + Send + 'static,
    AK: Casepath<PA, CA> + Copy + Send + Sync + 'static,
    SK: StateLens<PS, CS>,
{
    type State = PS;
    type Action = PA;

    fn reduce(&self, state: &mut PS, action: PA) -> Cmd<PA> {
        let Some(child_action) = self.action_kp.extract(&action) else {
            return Cmd::none();
        };
        let Some(child) = self.state_kp.focus_mut(state) else {
            return Cmd::none();
        };
        let cmd = self.child.reduce(child, child_action);
        lift_cmd(cmd, self.action_kp, self.cancel_id)
    }
}

/// `ifLet` — run child reducer when optional state is `Some`; cancel on dismiss.
#[derive(Debug)]
pub struct IfLetReducer<R, PS: 'static, PA: 'static, CS: 'static, CA: 'static, AK: 'static, SK>
where
    AK: Copy,
    SK: StateLens<PS, CS>,
{
    pub scope: ScopeReducer<R, PS, PA, CS, CA, AK, SK>,
    pub dismiss: fn(PA) -> bool,
    pub clear: fn(&mut PS),
    _marker: PhantomData<(PA, CA, PS, CS)>,
}

impl<R, PS: 'static, PA: 'static, CS: 'static, CA: 'static, AK: 'static, SK>
    IfLetReducer<R, PS, PA, CS, CA, AK, SK>
where
    AK: Copy,
    SK: StateLens<PS, CS>,
{
    pub fn new(
        state_kp: SK,
        action_kp: AK,
        dismiss: fn(PA) -> bool,
        clear: fn(&mut PS),
        cancel_id: EffectId,
        child: R,
    ) -> Self {
        Self {
            scope: ScopeReducer::new(state_kp, action_kp, cancel_id, child),
            dismiss,
            clear,
            _marker: PhantomData,
        }
    }
}

impl<R, PS: 'static, PA: 'static, CS: 'static, CA: 'static, AK: 'static, SK> Reducer
    for IfLetReducer<R, PS, PA, CS, CA, AK, SK>
where
    R: Reducer<State = CS, Action = CA>,
    PA: Copy + Send + 'static,
    CA: Copy + Send + 'static,
    AK: Casepath<PA, CA> + Copy + Send + Sync + 'static,
    SK: StateLens<PS, CS>,
{
    type State = PS;
    type Action = PA;

    fn reduce(&self, state: &mut PS, action: PA) -> Cmd<PA> {
        if (self.dismiss)(action) {
            let had_child = self.scope.state_kp.focus_mut(state).is_some();
            (self.clear)(state);
            if had_child {
                return Cmd::single(Effect::cancel(self.scope.cancel_id));
            }
            return Cmd::none();
        }
        self.scope.reduce(state, action)
    }
}

/// `ifCaseLet` — `ifLet` for enum-variant child state (same mechanics, enum-focused accessors).
pub type IfCaseLetReducer<R, PS, PA, CS, CA, SK> =
    IfLetReducer<R, PS, PA, CS, CA, CasePath<'static, PA, CA>, SK>;

/// `optional` — scope that no-ops when child state is absent (no dismiss/cancel).
pub type OptionalReducer<R, PS, PA, CS, CA, SK> =
    ScopeReducer<R, PS, PA, CS, CA, CasePath<'static, PA, CA>, SK>;

/// Run a child reducer for each element in an [`IdentifiedVec`].
#[derive(Clone, Debug)]
pub struct ForEachReducer<R, PS, PA, CS, CA, Id> {
    pub get_vec: fn(&mut PS) -> &mut IdentifiedVec<Id, CS>,
    pub embed: fn(Id, CA) -> PA,
    pub extract: fn(PA) -> Option<(Id, CA)>,
    pub extract_remove: fn(PA) -> Option<Id>,
    pub cancel_id: fn(Id) -> EffectId,
    pub child: R,
}

impl<R, PS, PA, CS, CA, Id> ForEachReducer<R, PS, PA, CS, CA, Id> {
    pub fn new(
        get_vec: fn(&mut PS) -> &mut IdentifiedVec<Id, CS>,
        embed: fn(Id, CA) -> PA,
        extract: fn(PA) -> Option<(Id, CA)>,
        extract_remove: fn(PA) -> Option<Id>,
        cancel_id: fn(Id) -> EffectId,
        child: R,
    ) -> Self {
        Self {
            get_vec,
            embed,
            extract,
            extract_remove,
            cancel_id,
            child,
        }
    }
}

impl<R, PS, PA, CS, CA, Id> Reducer for ForEachReducer<R, PS, PA, CS, CA, Id>
where
    R: Reducer<State = CS, Action = CA>,
    CS: Identifiable<Id = Id>,
    Id: Copy + Eq + std::hash::Hash + Send + Sync + 'static,
    PA: Copy + Send + 'static,
    CA: Send + 'static,
{
    type State = PS;
    type Action = PA;

    fn reduce(&self, state: &mut PS, action: PA) -> Cmd<PA> {
        if let Some(id) = (self.extract_remove)(action) {
            let vec = (self.get_vec)(state);
            if vec.remove(id).is_some() {
                return Cmd::single(Effect::cancel((self.cancel_id)(id)));
            }
            return Cmd::none();
        }

        let Some((id, child_action)) = (self.extract)(action) else {
            return Cmd::none();
        };
        let cancel_id = (self.cancel_id)(id);
        let Some(child) = (self.get_vec)(state).get_mut(id) else {
            return Cmd::none();
        };
        let cmd = self.child.reduce(child, child_action);
        lift_cmd_with_id(cmd, self.embed, id, cancel_id)
    }
}

/// Lift child command into parent action space with a per-id embed fn.
pub fn lift_cmd_with_id<PA, CA, Id>(
    cmd: Cmd<CA>,
    embed: fn(Id, CA) -> PA,
    id: Id,
    cancel_id: EffectId,
) -> Cmd<PA>
where
    CA: Send + 'static,
    PA: Send + 'static,
    Id: Copy + Send + Sync + 'static,
{
    match cmd {
        Cmd::None => Cmd::None,
        Cmd::Single(effect) => Cmd::Single(tag_cancel_id(
            map_effect_with_id(effect, embed, id),
            cancel_id,
        )),
        Cmd::Batch(cmds) => Cmd::Batch(
            cmds.into_iter()
                .map(|c| lift_cmd_with_id(c, embed, id, cancel_id))
                .collect(),
        ),
    }
}

fn map_effect_with_id<PA, CA, Id>(
    effect: Effect<CA>,
    embed: fn(Id, CA) -> PA,
    id: Id,
) -> Effect<PA>
where
    CA: Send + 'static,
    PA: Send + 'static,
    Id: Copy + Send + Sync + 'static,
{
    match effect {
        Effect::None => Effect::None,
        Effect::Task { run, .. } => Effect::from_fn(move || {
            let fut = run();
            Box::pin(async move { fut.await.map(|a| embed(id, a)) })
        }),
        Effect::RegisteredTask { id: task_id } => Effect::from_fn(move || {
            let fut = run_registered_task::<CA>(task_id);
            Box::pin(async move { fut.await.map(|a| embed(id, a)) })
        }),
        Effect::EnvTask { run, .. } => Effect::from_env_fn(move |env| {
            let fut = run(env);
            Box::pin(async move { fut.await.map(|a| embed(id, a)) })
        }),
        Effect::RegisteredEnvTask { id: task_id } => Effect::from_env_fn(move |env| {
            let fut = run_registered_env_task::<CA>(&env, task_id);
            Box::pin(async move { fut.await.map(|a| embed(id, a)) })
        }),
        Effect::Batch(items) => Effect::Batch(
            items
                .into_iter()
                .map(|e| map_effect_with_id(e, embed, id))
                .collect(),
        ),
        Effect::Cancellable {
            id: cid,
            cancel_in_flight,
            inner,
        } => Effect::Cancellable {
            id: cid,
            cancel_in_flight,
            inner: Box::new(map_effect_with_id(*inner, embed, id)),
        },
        Effect::Debounce {
            id: did,
            duration,
            inner,
        } => Effect::Debounce {
            id: did,
            duration,
            inner: Box::new(map_effect_with_id(*inner, embed, id)),
        },
        Effect::Throttle {
            id: tid,
            duration,
            latest,
            inner,
        } => Effect::Throttle {
            id: tid,
            duration,
            latest,
            inner: Box::new(map_effect_with_id(*inner, embed, id)),
        },
        Effect::RegisteredRun { id: run_id } => Effect::RegisteredRun { id: run_id },
        Effect::Provide { env, inner } => Effect::Provide {
            env,
            inner: Box::new(map_effect_with_id(*inner, embed, id)),
        },
        Effect::Retry { attempts, inner } => Effect::Retry {
            attempts,
            inner: Box::new(map_effect_with_id(*inner, embed, id)),
        },
        Effect::Timeout { duration, inner } => Effect::Timeout {
            duration,
            inner: Box::new(map_effect_with_id(*inner, embed, id)),
        },
        Effect::Sequence(items) => Effect::Sequence(
            items
                .into_iter()
                .map(|e| map_effect_with_id(e, embed, id))
                .collect(),
        ),
        Effect::Race(items) => Effect::Race(
            items
                .into_iter()
                .map(|e| map_effect_with_id(e, embed, id))
                .collect(),
        ),
        Effect::Catch { inner, recover: _ } => map_effect_with_id(*inner, embed, id),
        Effect::Cancel { id: cancel_id } => Effect::Cancel { id: cancel_id },
    }
}

/// Lift child command into parent action space and tag effects with a cancel id.
pub fn lift_cmd<PA, CA, CP>(cmd: Cmd<CA>, casepath: CP, cancel_id: EffectId) -> Cmd<PA>
where
    CA: Send + 'static,
    PA: Send + 'static,
    CP: Casepath<PA, CA> + Copy + Send + Sync + 'static,
{
    match cmd {
        Cmd::None => Cmd::None,
        Cmd::Single(effect) => Cmd::Single(tag_cancel_id(
            map_effect_with_casepath(effect, casepath),
            cancel_id,
        )),
        Cmd::Batch(cmds) => Cmd::Batch(
            cmds.into_iter()
                .map(|c| lift_cmd(c, casepath, cancel_id))
                .collect(),
        ),
    }
}

fn map_effect_with_casepath<PA, CA, CP>(effect: Effect<CA>, casepath: CP) -> Effect<PA>
where
    CA: Send + 'static,
    PA: Send + 'static,
    CP: Casepath<PA, CA> + Copy + Send + Sync + 'static,
{
    match effect {
        Effect::None => Effect::None,
        Effect::Task { run, .. } => Effect::from_fn(move || {
            let fut = run();
            Box::pin(async move { fut.await.map(|a| casepath.wrap(a)) })
        }),
        Effect::RegisteredTask { id: task_id } => Effect::from_fn(move || {
            let fut = run_registered_task::<CA>(task_id);
            Box::pin(async move { fut.await.map(|a| casepath.wrap(a)) })
        }),
        Effect::EnvTask { run, .. } => Effect::from_env_fn(move |env| {
            let fut = run(env);
            Box::pin(async move { fut.await.map(|a| casepath.wrap(a)) })
        }),
        Effect::RegisteredEnvTask { id: task_id } => Effect::from_env_fn(move |env| {
            let fut = run_registered_env_task::<CA>(&env, task_id);
            Box::pin(async move { fut.await.map(|a| casepath.wrap(a)) })
        }),
        Effect::Batch(items) => Effect::Batch(
            items
                .into_iter()
                .map(|e| map_effect_with_casepath(e, casepath))
                .collect(),
        ),
        Effect::Cancellable {
            id: cid,
            cancel_in_flight,
            inner,
        } => Effect::Cancellable {
            id: cid,
            cancel_in_flight,
            inner: Box::new(map_effect_with_casepath(*inner, casepath)),
        },
        Effect::Debounce {
            id: did,
            duration,
            inner,
        } => Effect::Debounce {
            id: did,
            duration,
            inner: Box::new(map_effect_with_casepath(*inner, casepath)),
        },
        Effect::Throttle {
            id: tid,
            duration,
            latest,
            inner,
        } => Effect::Throttle {
            id: tid,
            duration,
            latest,
            inner: Box::new(map_effect_with_casepath(*inner, casepath)),
        },
        Effect::RegisteredRun { id: run_id } => Effect::RegisteredRun { id: run_id },
        Effect::Provide { env, inner } => Effect::Provide {
            env,
            inner: Box::new(map_effect_with_casepath(*inner, casepath)),
        },
        Effect::Retry { attempts, inner } => Effect::Retry {
            attempts,
            inner: Box::new(map_effect_with_casepath(*inner, casepath)),
        },
        Effect::Timeout { duration, inner } => Effect::Timeout {
            duration,
            inner: Box::new(map_effect_with_casepath(*inner, casepath)),
        },
        Effect::Sequence(items) => Effect::Sequence(
            items
                .into_iter()
                .map(|e| map_effect_with_casepath(e, casepath))
                .collect(),
        ),
        Effect::Race(items) => Effect::Race(
            items
                .into_iter()
                .map(|e| map_effect_with_casepath(e, casepath))
                .collect(),
        ),
        Effect::Catch { inner, recover: _ } => map_effect_with_casepath(*inner, casepath),
        Effect::Cancel { id: cancel_id } => Effect::Cancel { id: cancel_id },
    }
}

fn tag_cancel_id<M>(effect: Effect<M>, cancel_id: EffectId) -> Effect<M> {
    match effect {
        Effect::None => Effect::None,
        Effect::Task { run, .. } => Effect::Task { id: cancel_id, run },
        Effect::RegisteredTask { .. } | Effect::EnvTask { .. } | Effect::RegisteredEnvTask { .. }
        | Effect::RegisteredRun { .. } => Effect::Cancellable {
            id: cancel_id,
            cancel_in_flight: true,
            inner: Box::new(tag_cancel_id(effect, cancel_id)),
        },
        Effect::Batch(items) => {
            Effect::Batch(items.into_iter().map(|e| tag_cancel_id(e, cancel_id)).collect())
        }
        Effect::Cancellable {
            id,
            cancel_in_flight,
            inner,
        } => Effect::Cancellable {
            id,
            cancel_in_flight,
            inner: Box::new(tag_cancel_id(*inner, cancel_id)),
        },
        Effect::Debounce {
            id,
            duration,
            inner,
        } => Effect::Debounce {
            id,
            duration,
            inner: Box::new(tag_cancel_id(*inner, cancel_id)),
        },
        Effect::Throttle {
            id,
            duration,
            latest,
            inner,
        } => Effect::Throttle {
            id,
            duration,
            latest,
            inner: Box::new(tag_cancel_id(*inner, cancel_id)),
        },
        Effect::Provide { env, inner } => Effect::Provide {
            env,
            inner: Box::new(tag_cancel_id(*inner, cancel_id)),
        },
        Effect::Retry { attempts, inner } => Effect::Retry {
            attempts,
            inner: Box::new(tag_cancel_id(*inner, cancel_id)),
        },
        Effect::Timeout { duration, inner } => Effect::Timeout {
            duration,
            inner: Box::new(tag_cancel_id(*inner, cancel_id)),
        },
        Effect::Sequence(items) => Effect::Sequence(
            items
                .into_iter()
                .map(|e| tag_cancel_id(e, cancel_id))
                .collect(),
        ),
        Effect::Race(items) => {
            Effect::Race(items.into_iter().map(|e| tag_cancel_id(e, cancel_id)).collect())
        }
        Effect::Catch { inner, recover } => Effect::Catch {
            inner: Box::new(tag_cancel_id(*inner, cancel_id)),
            recover,
        },
        Effect::Cancel { .. } => effect,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::Effect;
    use crate::optics::CasePath;
    use key_paths_derive::{Cp, Kp};
    use rust_identified_vec::IdentifiedVec;
    use crate::reducer::Reduce;

    #[derive(Default, Debug, PartialEq, Eq, Kp, Cp)]
    struct Child {
        n: i32,
    }

    #[derive(Default, Debug, PartialEq, Eq, Kp)]
    struct Parent {
        child: Option<Child>,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Kp, Cp)]
    enum ParentAction {
        Child(ChildAction),
        Dismiss,
        Other,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Kp)]
    enum ChildAction {
        Inc,
    }

    fn child_reducer(s: &mut Child, a: ChildAction) -> Cmd<ChildAction> {
        match a {
            ChildAction::Inc => s.n += 1,
        }
        Cmd::none()
    }

    // fn child_kp() -> rust_key_paths::KpType<'static, Parent, Child> {
    //     fn get(p: &Parent) -> Option<&Child> {
    //         p.child.as_ref()
    //     }
    //     fn get_mut(p: &mut Parent) -> Option<&mut Child> {
    //         p.child.as_mut()
    //     }
    //     rust_key_paths::Kp::new(get, get_mut)
    // }

    fn child_action_kp() -> CasePath<'static, ParentAction, ChildAction> {
        ParentAction::child_cp()
    }

    fn dismiss(a: ParentAction) -> bool {
        matches!(a, ParentAction::Dismiss)
    }

    fn clear(p: &mut Parent) {
        p.child = None;
    }

    #[test]
    fn scope_accepts_derived_state_kp() {
        let n_kp = Child::n_cp();
        assert_eq!(n_kp.get_ref(&Child { n: 42 }), Some(&42));

        let scope = ScopeReducer::new(
            Parent::child(),
            ParentAction::child_cp(),
            1,
            Reduce::new(child_reducer),
        );
        let mut parent = Parent {
            child: Some(Child { n: 0 }),
        };
        scope.reduce(&mut parent, ParentAction::Child(ChildAction::Inc));
        assert_eq!(parent.child.as_ref().unwrap().n, 1);
    }

    #[test]
    fn scope_isolates_child_state() {
        let scope = ScopeReducer::new(
           Parent::child(),
            ParentAction::child_cp(),
            1,
            Reduce::new(child_reducer),
        );
        let mut parent = Parent {
            child: Some(Child { n: 0 }),
        };
        scope.reduce(&mut parent, ParentAction::Child(ChildAction::Inc));
        assert_eq!(parent.child.as_ref().unwrap().n, 1);
        scope.reduce(&mut parent, ParentAction::Other);
        assert_eq!(parent.child.as_ref().unwrap().n, 1);
    }

    #[test]
    fn if_let_dismiss_cancels_and_clears() {
        let if_let = IfLetReducer::new(
            Parent::child(),
            ParentAction::child_cp(),
            dismiss,
            clear,
            42,
            Reduce::new(child_reducer),
        );
        let mut parent = Parent {
            child: Some(Child { n: 0 }),
        };
        let cmd = if_let.reduce(&mut parent, ParentAction::Dismiss);
        assert!(parent.child.is_none());
        assert_eq!(cmd.effect_count(), 1);
        assert!(matches!(
            cmd.into_effects().first(),
            Some(Effect::Cancel { id: 42 })
        ));
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Row {
        id: u64,
        n: i32,
    }

    impl Identifiable for Row {
        type Id = u64;
        fn id(&self) -> u64 {
            self.id
        }
    }

    #[derive(Default, Debug, PartialEq, Eq)]
    struct ListParent {
        rows: IdentifiedVec<u64, Row>,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ListAction {
        Row(u64, RowAction),
        Remove(u64),
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum RowAction {
        Bump,
    }

    fn row_reducer(r: &mut Row, a: RowAction) -> Cmd<RowAction> {
        if matches!(a, RowAction::Bump) {
            r.n += 1;
        }
        Cmd::none()
    }

    fn get_vec(p: &mut ListParent) -> &mut IdentifiedVec<u64, Row> {
        &mut p.rows
    }

    fn embed_row(id: u64, a: RowAction) -> ListAction {
        ListAction::Row(id, a)
    }

    fn extract_row(a: ListAction) -> Option<(u64, RowAction)> {
        match a {
            ListAction::Row(id, action) => Some((id, action)),
            _ => None,
        }
    }

    fn extract_remove(a: ListAction) -> Option<u64> {
        match a {
            ListAction::Remove(id) => Some(id),
            _ => None,
        }
    }

    fn cancel_for_id(id: u64) -> EffectId {
        1000 + id
    }

    #[test]
    fn for_each_scopes_by_id() {
        let for_each = ForEachReducer::new(
            get_vec,
            embed_row,
            extract_row,
            extract_remove,
            cancel_for_id,
            Reduce::new(row_reducer),
        );
        let mut parent = ListParent::default();
        parent.rows.insert(Row { id: 1, n: 0 });
        parent.rows.insert(Row { id: 2, n: 0 });
        for_each.reduce(&mut parent, ListAction::Row(1, RowAction::Bump));
        assert_eq!(parent.rows.get(1).unwrap().n, 1);
        assert_eq!(parent.rows.get(2).unwrap().n, 0);
    }

    #[test]
    fn for_each_remove_emits_cancel() {
        let for_each = ForEachReducer::new(
            get_vec,
            embed_row,
            extract_row,
            extract_remove,
            cancel_for_id,
            Reduce::new(row_reducer),
        );
        let mut parent = ListParent::default();
        parent.rows.insert(Row { id: 1, n: 0 });
        let cmd = for_each.reduce(&mut parent, ListAction::Remove(1));
        assert!(parent.rows.get(1).is_none());
        assert!(matches!(
            cmd.into_effects().first(),
            Some(Effect::Cancel { id: 1001 })
        ));
    }
}
