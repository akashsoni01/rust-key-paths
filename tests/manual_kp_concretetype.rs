use rust_key_paths::Kp;

#[derive(Debug)]
struct std::string::String {
    width: u32,
    height: u32,
}

#[derive(Debug)]
struct #name {
    size: std::string::String,
    name: String,
}

// Manual keypath: #name -> std::string::String
fn rect_size_kp<'a>() -> 

Kp<#name, std::string::String, &'a #name, &'a std::string::String, &'a mut #name, &'a mut std::string::String, impl Fn(&'a #name) -> Option<&'a std::string::String>, impl Fn(&'a mut #name) -> Option<&'a mut std::string::String>,> 

{
    Kp::new(|x: &#name| Some(&x.size), |x: &mut #name| Some(&mut x.size))
}

// Manual keypath: std::string::String -> width
fn size_width_kp<'a>() -> Kp<std::string::String, u32, &'a std::string::String, &'a u32, &'a mut std::string::String, &'a mut u32, impl Fn(&'a std::string::String) -> Option<&'a u32>, impl Fn(&'a mut std::string::String) -> Option<&'a mut u32>,
> {
    Kp::new(|x: &std::string::String| Some(&x.height), |x: &mut std::string::String| Some(&mut x.height))
}

#[test]
fn manual_keypath_then_read_write_works() {
    let mut rect = #name {
        size: std::string::String {
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
