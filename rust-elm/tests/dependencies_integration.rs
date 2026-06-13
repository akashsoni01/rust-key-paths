use std::sync::Arc;

use rust_elm::{
    dependencies::{RngDep, UuidDep, UuidGen},
    DependencyKey, Environment, RngKey, SeededRng, UuidKey,
};

#[test]
fn live_env_has_controllable_builtins() {
    let live = Environment::live();
    assert!(live.require::<UuidDep>().is_ok());
    assert!(live.require::<RngDep>().is_ok());
    let _ = UuidKey::live();
    let _ = RngKey::live();
}

#[test]
fn test_env_uuid_and_rng_are_seeded() {
    let env = Environment::test();
    let u1 = env.require::<UuidDep>().unwrap().0.next();
    let u2 = env.require::<UuidDep>().unwrap().0.next();
    let env2 = Environment::test();
    assert_eq!(u1, env2.require::<UuidDep>().unwrap().0.next());
    assert_ne!(u1, u2);

    let r1 = env.require::<RngDep>().unwrap().0.next_u64();
    let env3 = Environment::test();
    assert_eq!(r1, env3.require::<RngDep>().unwrap().0.next_u64());
}

#[test]
fn provide_dependency_scopes_override() {
    let base = Environment::test();
    let base_uuid = base.require::<UuidDep>().unwrap().0.next();

    let overlay = Environment::new().with(UuidDep(Arc::new(rust_elm::SeededUuidGen::new(777))));
    let scoped = base.scoped_with(overlay);
    let scoped_uuid = scoped.require::<UuidDep>().unwrap().0.next();

    assert_ne!(base_uuid, scoped_uuid);
    assert_eq!(
        scoped_uuid,
        rust_elm::SeededUuidGen::new(777).next()
    );
}

#[test]
fn rng_override_is_repeatable() {
    let env = Environment::test().with(RngDep(Arc::new(SeededRng::new(123))));
    let a = env.require::<RngDep>().unwrap().0.next_u64();
    let env2 = Environment::test().with(RngDep(Arc::new(SeededRng::new(123))));
    assert_eq!(a, env2.require::<RngDep>().unwrap().0.next_u64());
}
