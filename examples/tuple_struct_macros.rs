use rust_keypaths::{KeyPath, OptionalKeyPath, WritableKeyPath, WritableOptionalKeyPath};
use keypaths_proc::Keypaths;

#[derive(Debug, Keypaths)]
#[All]
struct Point(u32, Option<u32>, String);

fn main() {
    let mut p = Point(10, Some(20), "name".into());
    // Non-Option fields
    let x_r = Point::f0_r();
    let name_w = Point::f2_w();
    println!("x = {:?}", x_r.get(&p));
    let n = name_w.get_mut(&mut p);
    {
        n.push_str("_edited");
    }

    // Option field with failable
    let y_fr = Point::f1_fr();
    println!("y (fr) = {:?}", y_fr.get(&p));

    let y_fw = Point::f1_fw();
    if let Some(y) = y_fw.get_mut(&mut p)
    {
        *y += 1;
    }

    println!("updated p = {:?}", p);
}
