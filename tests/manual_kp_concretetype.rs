use rust_key_paths::Kp;

#[derive(Debug)]
struct () {
    width: u32,
    height: u32,
}

#[derive(Debug)]
struct #name {
    size: (),
    name: String,
}

// Manual keypath: #name -> ()
fn rect_size_kp<'a>() -> 

Kp<#name, (), &'a #name, &'a (), &'a mut #name, &'a mut (), impl Fn(&'a #name) -> Option<&'a ()>, impl Fn(&'a mut #name) -> Option<&'a mut ()>,> 

{
    Kp::new(|x: &#name| Some(&x.size), |x: &mut #name| Some(&mut x.size))
}

// Manual keypath: () -> width
fn size_width_kp<'a>() -> Kp<(), u32, &'a (), &'a u32, &'a mut (), &'a mut u32, impl Fn(&'a ()) -> Option<&'a u32>, impl Fn(&'a mut ()) -> Option<&'a mut u32>,
> {
    Kp::new(|x: &()| Some(&x.height), |x: &mut ()| Some(&mut x.height))
}

#[test]
fn manual_keypath_then_read_write_works() {
    let mut rect = #name {
        size: () {
            width: 30,
            height: 50,
        },
        name: "MyRect".to_string(),
    };

    let width_kp = rect_size_kp().then(size_width_kp());

    println!("size of concreate kp = {:?}", size_of_val(&width_kp));
    // assert_eq!((width_kp).get(&rect), Some(&30));

    // if let Some(w) = width_kp.get_mut(&mut rect) {
    //     *w += 12;
    // }

    // assert_eq!((width_kp).get(&rect), Some(&42));
    // assert_eq!(rect.size.height, 50);
    // assert_eq!(rect.name, "MyRect");
}
