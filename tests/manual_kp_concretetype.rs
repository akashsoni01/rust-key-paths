use rust_key_paths::Kp;

#[derive(Debug)]
struct std::sync::Arc<std::sync::Mutex<#inner_ty>> {
    width: u32,
    height: u32,
}

#[derive(Debug)]
struct #name {
    size: std::sync::Arc<std::sync::Mutex<#inner_ty>>,
    name: String,
}

// Manual keypath: #name -> std::sync::Arc<std::sync::Mutex<#inner_ty>>
fn rect_size_kp<'a>() -> Kp<#name, std::sync::Arc<std::sync::Mutex<#inner_ty>>, &'a #name, &'a std::sync::Arc<std::sync::Mutex<#inner_ty>>, &'a mut #name, &'a mut std::sync::Arc<std::sync::Mutex<#inner_ty>>, impl Fn(&'a #name) -> Option<&'a std::sync::Arc<std::sync::Mutex<#inner_ty>>>, impl Fn(&'a mut #name) -> Option<&'a mut std::sync::Arc<std::sync::Mutex<#inner_ty>>>,> {
    Kp::new(|x: &#name| Some(&x.size), |x: &mut #name| Some(&mut x.size))
}

// Manual keypath: std::sync::Arc<std::sync::Mutex<#inner_ty>> -> width
fn size_width_kp<'a>() -> Kp<std::sync::Arc<std::sync::Mutex<#inner_ty>>, u32, &'a std::sync::Arc<std::sync::Mutex<#inner_ty>>, &'a u32, &'a mut std::sync::Arc<std::sync::Mutex<#inner_ty>>, &'a mut u32, impl Fn(&'a std::sync::Arc<std::sync::Mutex<#inner_ty>>) -> Option<&'a u32>, impl Fn(&'a mut std::sync::Arc<std::sync::Mutex<#inner_ty>>) -> Option<&'a mut u32>,
> {
    Kp::new(|x: &std::sync::Arc<std::sync::Mutex<#inner_ty>>| Some(&x.height), |x: &mut std::sync::Arc<std::sync::Mutex<#inner_ty>>| Some(&mut x.height))
}

#[test]
fn manual_keypath_then_read_write_works() {
    let mut rect = #name {
        size: std::sync::Arc<std::sync::Mutex<#inner_ty>> {
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
