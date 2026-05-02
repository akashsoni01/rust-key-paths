use rust_key_paths::{Kp, KpType};
use std::mem::size_of_val;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug)]
struct Level2 {
    value: u32,
}

#[derive(Debug)]
struct Level1 {
    l2: Level2,
}

#[derive(Debug)]
struct Root {
    plain: Level1,
    boxed: Box<Level1>,
    rc: Rc<Level1>,
    arc: Arc<Level1>,
}

fn l2_value_kp() -> KpType<'static, Level2, u32> {
    Kp::new(
        |l2: &Level2| Some(&l2.value),
        |l2: &mut Level2| Some(&mut l2.value),
    )
}

fn l1_l2_kp() -> KpType<'static, Level1, Level2> {
    Kp::new(
        |l1: &Level1| Some(&l1.l2),
        |l1: &mut Level1| Some(&mut l1.l2),
    )
}

fn root_plain_kp() -> KpType<'static, Root, Level1> {
    Kp::new(
        |root: &Root| Some(&root.plain),
        |root: &mut Root| Some(&mut root.plain),
    )
}

fn root_boxed_kp() -> KpType<'static, Root, Box<Level1>> {
    Kp::new(
        |root: &Root| Some(&root.boxed),
        |root: &mut Root| Some(&mut root.boxed),
    )
}

fn box_inner_kp() -> KpType<'static, Box<Level1>, Level1> {
    Kp::new(
        |boxed: &Box<Level1>| Some(boxed.as_ref()),
        |boxed: &mut Box<Level1>| Some(boxed.as_mut()),
    )
}

fn root_rc_kp() -> KpType<'static, Root, Rc<Level1>> {
    Kp::new(
        |root: &Root| Some(&root.rc),
        |root: &mut Root| Some(&mut root.rc),
    )
}

fn rc_inner_kp() -> KpType<'static, Rc<Level1>, Level1> {
    Kp::new(
        |rc: &Rc<Level1>| Some(rc.as_ref()),
        |rc: &mut Rc<Level1>| Rc::get_mut(rc),
    )
}

fn root_arc_kp() -> KpType<'static, Root, Arc<Level1>> {
    Kp::new(
        |root: &Root| Some(&root.arc),
        |root: &mut Root| Some(&mut root.arc),
    )
}

fn arc_inner_kp() -> KpType<'static, Arc<Level1>, Level1> {
    Kp::new(
        |arc: &Arc<Level1>| Some(arc.as_ref()),
        |arc: &mut Arc<Level1>| Arc::get_mut(arc),
    )
}

fn make_root() -> Root {
    let level1 = Level1 {
        l2: Level2 { value: 99 },
    };
    Root {
        plain: Level1 {
            l2: Level2 { value: 10 },
        },
        boxed: Box::new(Level1 {
            l2: Level2 { value: 20 },
        }),
        rc: Rc::new(Level1 {
            l2: Level2 { value: 30 },
        }),
        arc: Arc::new(level1),
    }
}

#[test]
fn three_level_keypath_size_with_basic_containers_is_not_zero() {
    let root = make_root();

    let plain_three_level = root_plain_kp().then(l1_l2_kp()).then(l2_value_kp());
    let boxed_three_level = root_boxed_kp()
        .then(box_inner_kp())
        .then(l1_l2_kp())
        .then(l2_value_kp());
    let rc_three_level = root_rc_kp()
        .then(rc_inner_kp())
        .then(l1_l2_kp())
        .then(l2_value_kp());
    let arc_three_level = root_arc_kp()
        .then(arc_inner_kp())
        .then(l1_l2_kp())
        .then(l2_value_kp());

    assert_eq!(plain_three_level.get(&root), Some(&10));
    assert_eq!(boxed_three_level.get(&root), Some(&20));
    assert_eq!(rc_three_level.get(&root), Some(&30));
    assert_eq!(arc_three_level.get(&root), Some(&99));

    assert_ne!(size_of_val(&plain_three_level), 0);
    assert_ne!(size_of_val(&boxed_three_level), 0);
    assert_ne!(size_of_val(&rc_three_level), 0);
    assert_ne!(size_of_val(&arc_three_level), 0);
}
