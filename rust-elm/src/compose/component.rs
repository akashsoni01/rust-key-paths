use crate::cmd::Cmd;

/// Scoped child slot — lifts child update into parent state/action space.
pub struct Slot<ParentState, ParentAction, ChildState, ChildAction> {
    pub get: fn(&ParentState) -> Option<&ChildState>,
    pub get_mut: fn(&mut ParentState) -> Option<&mut ChildState>,
    pub embed: fn(ChildAction) -> ParentAction,
    pub child_update: fn(&mut ChildState, ChildAction) -> Cmd<ChildAction>,
}

impl<PS, PA: Send + 'static, CS, CA: Send + 'static> Slot<PS, PA, CS, CA> {
    pub fn new(
        get: fn(&PS) -> Option<&CS>,
        get_mut: fn(&mut PS) -> Option<&mut CS>,
        embed: fn(CA) -> PA,
        child_update: fn(&mut CS, CA) -> Cmd<CA>,
    ) -> Self {
        Self {
            get,
            get_mut,
            embed,
            child_update,
        }
    }

    pub fn lift(&self, parent: &mut PS, action: PA, extract: fn(PA) -> Option<CA>) -> Cmd<PA> {
        let Some(child_action) = extract(action) else {
            return Cmd::none();
        };
        let Some(child) = (self.get_mut)(parent) else {
            return Cmd::none();
        };
        (self.child_update)(child, child_action).map(self.embed)
    }
}

/// Lift a child `Cmd` into the parent action space.
pub fn lift<PA: Send + 'static, CA: Send + 'static>(
    cmd: Cmd<CA>,
    embed: fn(CA) -> PA,
) -> Cmd<PA> {
    cmd.map(embed)
}

#[cfg(test)]
#[allow(dead_code, clippy::bool_assert_comparison)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Parent {
        child: Option<Child>,
    }

    #[derive(Default)]
    struct Child {
        n: i32,
    }

    #[derive(Clone, Copy)]
    enum PMsg {
        Child(CMsg),
    }

    #[derive(Clone, Copy)]
    enum CMsg {
        Inc,
    }

    fn child_update(c: &mut Child, msg: CMsg) -> Cmd<CMsg> {
        match msg {
            CMsg::Inc => c.n += 1,
        }
        Cmd::none()
    }

    #[test]
    fn slot_lift_updates_nested_state() {
        let slot = Slot::new(
            |p: &Parent| p.child.as_ref(),
            |p: &mut Parent| p.child.as_mut(),
            PMsg::Child,
            child_update,
        );
        let mut parent = Parent {
            child: Some(Child::default()),
        };
        let extract = |m: PMsg| match m {
            PMsg::Child(c) => Some(c),
        };
        let _ = slot.lift(&mut parent, PMsg::Child(CMsg::Inc), extract);
        assert_eq!(parent.child.unwrap().n, 1);
    }
}
