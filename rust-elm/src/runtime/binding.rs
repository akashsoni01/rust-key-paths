//! Lock-backed bindings with keypath projection (SwiftUI `Binding` dynamic-member style).
//!
//! Reads and writes go through the store's shared mutex — no full-state clone.
//! Project a field with [`StateBinding::project`] / [`ProjectedBinding::project`].
//!
//! When a keypath cannot focus (e.g. `None` in an `Option` field), [`ProjectedBinding::with`]
//! and [`ProjectedBinding::with_mut`] return [`None`] instead of panicking.

use std::marker::PhantomData;
use std::sync::Arc;

use key_paths_core::RefKpTrait;
use parking_lot::{Mutex, RwLock};

use crate::runtime::state_access::{StateRead, StateWrite};

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
    /// Borrow the focused value under the root lock, or [`None`] if the keypath misses.
    pub fn with<R>(&self, f: impl FnOnce(&Focus) -> R) -> Option<R> {
        let guard = self.state.lock();
        let focused = self.kp.focus(&*guard)?;
        Some(f(focused))
    }

    /// Mutably borrow the focused value under the root lock, or [`None`] if the keypath misses.
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut Focus) -> R) -> Option<R> {
        let mut guard = self.state.lock();
        let focused = self.kp.focus_mut(&mut *guard)?;
        Some(f(focused))
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
    /// Borrow through parent then child keypath; [`None`] if either segment misses.
    pub fn with<R>(&self, f: impl FnOnce(&Focus) -> R) -> Option<R> {
        let guard = self.state.lock();
        let mid = self.parent_kp.focus(&*guard)?;
        let focused = self.sub_kp.focus(mid)?;
        Some(f(focused))
    }

    /// Mutably borrow through parent then child keypath; [`None`] if either segment misses.
    pub fn with_mut<R>(&self, f: impl FnOnce(&mut Focus) -> R) -> Option<R> {
        let mut guard = self.state.lock();
        let mid = self.parent_kp.focus_mut(&mut *guard)?;
        let focused = self.sub_kp.focus_mut(mid)?;
        Some(f(focused))
    }
}

// ── RwLock-backed bindings (concurrent readers) ─────────────────────────────

/// Read-only binding over [`RwLock`] state — many concurrent readers.
#[derive(Clone)]
pub struct ReadStateBinding<S: 'static> {
    state: Arc<RwLock<S>>,
}

impl<S: 'static> ReadStateBinding<S> {
    pub(crate) fn new(state: Arc<RwLock<S>>) -> Self {
        Self { state }
    }

    pub fn with_read<R>(&self, f: impl FnOnce(&S) -> R) -> R {
        self.state.with_read(f)
    }

    pub fn project<V, SK>(&self, kp: SK) -> ReadProjectedBinding<S, V, SK>
    where
        SK: RefKpTrait<S, V>,
    {
        ReadProjectedBinding {
            state: Arc::clone(&self.state),
            kp,
            _marker: PhantomData,
        }
    }
}

/// Read/write binding over [`RwLock`] state — writes take exclusive lock.
#[derive(Clone)]
pub struct RwStateBinding<S: 'static> {
    state: Arc<RwLock<S>>,
}

impl<S: 'static> RwStateBinding<S> {
    pub(crate) fn new(state: Arc<RwLock<S>>) -> Self {
        Self { state }
    }

    pub fn with_read<R>(&self, f: impl FnOnce(&S) -> R) -> R {
        self.state.with_read(f)
    }

    pub fn with_write<R>(&self, f: impl FnOnce(&mut S) -> R) -> R {
        self.state.with_write(f)
    }

    pub fn read_store(&self) -> ReadStateBinding<S> {
        ReadStateBinding::new(Arc::clone(&self.state))
    }

    pub fn project<V, SK>(&self, kp: SK) -> RwProjectedBinding<S, V, SK>
    where
        SK: RefKpTrait<S, V>,
    {
        RwProjectedBinding {
            state: Arc::clone(&self.state),
            kp,
            _marker: PhantomData,
        }
    }
}

/// Read-only projected field from [`ReadStateBinding`].
pub struct ReadProjectedBinding<Root: 'static, Focus: 'static, SK> {
    state: Arc<RwLock<Root>>,
    kp: SK,
    _marker: PhantomData<Focus>,
}

impl<Root, Focus, SK> Clone for ReadProjectedBinding<Root, Focus, SK>
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

impl<Root, Focus, SK> ReadProjectedBinding<Root, Focus, SK>
where
    Root: 'static,
    Focus: 'static,
    SK: RefKpTrait<Root, Focus>,
{
    pub fn with_read<R>(&self, f: impl FnOnce(&Focus) -> R) -> Option<R> {
        let guard = self.state.read();
        let focused = self.kp.focus(&*guard)?;
        Some(f(focused))
    }
}

/// Projected field from [`RwStateBinding`].
pub struct RwProjectedBinding<Root: 'static, Focus: 'static, SK> {
    state: Arc<RwLock<Root>>,
    kp: SK,
    _marker: PhantomData<Focus>,
}

impl<Root, Focus, SK> Clone for RwProjectedBinding<Root, Focus, SK>
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

impl<Root, Focus, SK> RwProjectedBinding<Root, Focus, SK>
where
    Root: 'static,
    Focus: 'static,
    SK: RefKpTrait<Root, Focus>,
{
    pub fn with_read<R>(&self, f: impl FnOnce(&Focus) -> R) -> Option<R> {
        let guard = self.state.read();
        let focused = self.kp.focus(&*guard)?;
        Some(f(focused))
    }

    pub fn with_write<R>(&self, f: impl FnOnce(&mut Focus) -> R) -> Option<R> {
        let mut guard = self.state.write();
        let focused = self.kp.focus_mut(&mut *guard)?;
        Some(f(focused))
    }
}

// ── ArcSwap snapshot bindings (`arc-swap` feature) ──────────────────────────

/// Lock-free read binding over [`ArcSwap`](arc_swap::ArcSwap) state snapshots.
#[cfg(feature = "arc-swap")]
#[derive(Clone)]
pub struct SnapshotStateBinding<S: 'static> {
    state: Arc<arc_swap::ArcSwap<S>>,
}

#[cfg(feature = "arc-swap")]
impl<S: 'static> SnapshotStateBinding<S> {
    pub(crate) fn new(state: Arc<arc_swap::ArcSwap<S>>) -> Self {
        Self { state }
    }

    pub fn with_snapshot<R>(&self, f: impl FnOnce(&S) -> R) -> R {
        f(&*self.state.load_full())
    }

    pub fn load(&self) -> Arc<S>
    where
        S: Clone,
    {
        self.state.load_full()
    }

    pub fn project<V, SK>(&self, kp: SK) -> SnapshotProjectedBinding<S, V, SK>
    where
        SK: RefKpTrait<S, V>,
    {
        SnapshotProjectedBinding {
            state: Arc::clone(&self.state),
            kp,
            _marker: PhantomData,
        }
    }
}

/// Projected field from [`SnapshotStateBinding`].
#[cfg(feature = "arc-swap")]
pub struct SnapshotProjectedBinding<Root: 'static, Focus: 'static, SK> {
    state: Arc<arc_swap::ArcSwap<Root>>,
    kp: SK,
    _marker: PhantomData<Focus>,
}

#[cfg(feature = "arc-swap")]
impl<Root, Focus, SK> Clone for SnapshotProjectedBinding<Root, Focus, SK>
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

#[cfg(feature = "arc-swap")]
impl<Root, Focus, SK> SnapshotProjectedBinding<Root, Focus, SK>
where
    Root: 'static,
    Focus: 'static,
    SK: RefKpTrait<Root, Focus>,
{
    pub fn with_snapshot<R>(&self, f: impl FnOnce(&Focus) -> R) -> Option<R> {
        let snapshot = self.state.load_full();
        let focused = self.kp.focus(&*snapshot)?;
        Some(f(focused))
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
        assert_eq!(position.with(|p| *p), Some(1));

        position.with_mut(|p| *p = 2);
        assert_eq!(binding.with(|e| e.current_position), 2);

        let favorite = binding.project(favorite_kp());
        favorite.with_mut(|f| *f = true);
        assert!(binding.with(|e| e.is_favorite));
    }

    #[test]
    fn projected_binding_returns_none_when_keypath_misses() {
        #[derive(Kp, Clone, Hash, FieldDiff, PartialEq, Debug)]
        struct Root {
            child: Option<Episode>,
        }

        fn child_kp() -> KpPath<
            Root,
            Episode,
            &'static Root,
            &'static Episode,
            &'static mut Root,
            &'static mut Episode,
            for<'b> fn(&'b Root) -> Option<&'b Episode>,
            for<'b> fn(&'b mut Root) -> Option<&'b mut Episode>,
        > {
            fn get(r: &Root) -> Option<&Episode> {
                r.child.as_ref()
            }
            fn get_mut(r: &mut Root) -> Option<&mut Episode> {
                r.child.as_mut()
            }
            KpPath::new(get, get_mut)
        }

        let binding = StateBinding::new(Arc::new(Mutex::new(Root { child: None })));
        let child = binding.project(child_kp());
        assert!(child.with(|_| ()).is_none());
        assert!(child.with_mut(|_| ()).is_none());
    }
}
