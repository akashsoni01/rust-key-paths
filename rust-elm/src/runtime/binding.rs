//! Lock-backed bindings with keypath projection (SwiftUI `Binding` dynamic-member style).
//!
//! Reads and writes go through the store's shared mutex — no full-state clone.
//! Project a field with [`StateBinding::project`] / [`ProjectedBinding::project`].

use std::marker::PhantomData;
use std::sync::Arc;

use key_paths_core::RefKpTrait;
use parking_lot::Mutex;

/// Read/write access to store state without cloning `S`.
#[derive(Clone)]
pub struct StateBinding<S: 'static> {
    state: Arc<Mutex<S>>,
}

impl<S: 'static> StateBinding<S> {
    pub(crate) fn new(state: Arc<Mutex<S>>) -> Self {
        Self { state }
    }

    /// Borrow the root state under the lock.
    pub fn with<R>(&self, f: impl FnOnce(&S) -> R) -> R {
        f(&*self.state.lock())
    }

    /// Mutably borrow the root state under the lock.
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut S) -> R) -> R {
        f(&mut *self.state.lock())
    }

    /// Project a property via keypath — reads/writes go to the original binding's state.
    pub fn project<V, SK>(&self, kp: SK) -> ProjectedBinding<S, V, SK>
    where
        SK: RefKpTrait<S, V>,
    {
        ProjectedBinding {
            state: Arc::clone(&self.state),
            kp,
            _marker: PhantomData,
        }
    }
}

/// A field projected from a [`StateBinding`] through a keypath.
pub struct ProjectedBinding<Root: 'static, Focus: 'static, SK> {
    state: Arc<Mutex<Root>>,
    kp: SK,
    _marker: PhantomData<Focus>,
}

impl<Root, Focus, SK> Clone for ProjectedBinding<Root, Focus, SK>
where
    Root: 'static,
    Focus: 'static,
    SK: RefKpTrait<Root, Focus> + Clone,
{
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            kp: self.kp.clone(),
            _marker: PhantomData,
        }
    }
}

impl<Root, Focus, SK> ProjectedBinding<Root, Focus, SK>
where
    Root: 'static,
    Focus: 'static,
    SK: RefKpTrait<Root, Focus>,
{
    /// Borrow the focused value under the root lock.
    pub fn with<R>(&self, f: impl FnOnce(&Focus) -> R) -> R {
        let guard = self.state.lock();
        let focused = self
            .kp
            .focus(&*guard)
            .expect("keypath projection failed: field not present");
        f(focused)
    }

    /// Mutably borrow the focused value under the root lock.
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut Focus) -> R) -> R {
        let mut guard = self.state.lock();
        let focused = self
            .kp
            .focus_mut(&mut *guard)
            .expect("keypath projection failed: field not present");
        f(focused)
    }

    /// Chain another keypath segment (consumes `self`; parent keypath must be [`Clone`]).
    pub fn project<V, SubSK>(self, sub_kp: SubSK) -> ComposedBinding<Root, Focus, V, SK, SubSK>
    where
        SubSK: RefKpTrait<Focus, V>,
        SK: Clone,
    {
        ComposedBinding {
            state: self.state,
            parent_kp: self.kp,
            sub_kp,
            _marker: PhantomData,
        }
    }
}

/// Two-segment keypath projection from root state.
pub struct ComposedBinding<Root: 'static, Mid: 'static, Focus: 'static, ParentKP, SubKP> {
    state: Arc<Mutex<Root>>,
    parent_kp: ParentKP,
    sub_kp: SubKP,
    _marker: PhantomData<(Mid, Focus)>,
}

impl<Root, Mid, Focus, ParentKP, SubKP> ComposedBinding<Root, Mid, Focus, ParentKP, SubKP>
where
    Root: 'static,
    Mid: 'static,
    Focus: 'static,
    ParentKP: RefKpTrait<Root, Mid>,
    SubKP: RefKpTrait<Mid, Focus>,
{
    pub fn with<R>(&self, f: impl FnOnce(&Focus) -> R) -> R {
        let guard = self.state.lock();
        let mid = self
            .parent_kp
            .focus(&*guard)
            .expect("keypath projection failed: parent field not present");
        let focused = self
            .sub_kp
            .focus(mid)
            .expect("keypath projection failed: child field not present");
        f(focused)
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut Focus) -> R) -> R {
        let mut guard = self.state.lock();
        let mid = self
            .parent_kp
            .focus_mut(&mut *guard)
            .expect("keypath projection failed: parent field not present");
        let focused = self
            .sub_kp
            .focus_mut(mid)
            .expect("keypath projection failed: child field not present");
        f(focused)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use key_paths_derive::{FieldDiff, Kp};
    use rust_key_paths::Kp as KpPath;

    #[derive(Debug, Kp, Clone, Hash, FieldDiff, PartialEq)]
    struct Episode {
        current_position: i32,
        is_favorite: bool,
    }

    fn position_kp() -> KpPath<
        Episode,
        i32,
        &'static Episode,
        &'static i32,
        &'static mut Episode,
        &'static mut i32,
        for<'b> fn(&'b Episode) -> Option<&'b i32>,
        for<'b> fn(&'b mut Episode) -> Option<&'b mut i32>,
    > {
        fn get(e: &Episode) -> Option<&i32> {
            Some(&e.current_position)
        }
        fn get_mut(e: &mut Episode) -> Option<&mut i32> {
            Some(&mut e.current_position)
        }
        KpPath::new(get, get_mut)
    }

    fn favorite_kp() -> KpPath<
        Episode,
        bool,
        &'static Episode,
        &'static bool,
        &'static mut Episode,
        &'static mut bool,
        for<'b> fn(&'b Episode) -> Option<&'b bool>,
        for<'b> fn(&'b mut Episode) -> Option<&'b mut bool>,
    > {
        fn get(e: &Episode) -> Option<&bool> {
            Some(&e.is_favorite)
        }
        fn get_mut(e: &mut Episode) -> Option<&mut bool> {
            Some(&mut e.is_favorite)
        }
        KpPath::new(get, get_mut)
    }

    #[test]
    fn binding_projects_fields_without_cloning_root() {
        let state = Arc::new(Mutex::new(Episode {
            current_position: 1,
            is_favorite: false,
        }));
        let binding = StateBinding::new(state);

        let position = binding.project(position_kp());
        assert_eq!(position.with(|p| *p), 1);

        position.with_mut(|p| *p = 2);
        assert_eq!(binding.with(|e| e.current_position), 2);

        let favorite = binding.project(favorite_kp());
        favorite.with_mut(|f| *f = true);
        assert!(binding.with(|e| e.is_favorite));
    }
}
