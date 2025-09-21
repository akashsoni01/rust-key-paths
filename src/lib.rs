// use std::rc::Rc;

// // Core traits
// pub trait ReadKeyPath<Root, Value> {
//     fn get<'a>(&self, root: &'a Root) -> Option<&'a Value>;
// }

// pub trait WriteKeyPath<Root, Value> {
//     fn get_mut<'a>(&self, root: &'a mut Root) -> Option<&'a mut Value>;
//     fn set(&self, root: &mut Root, value: Value);
// }

// pub trait EmbedKeyPath<Root, Value> {
//     fn embed(&self, value: Value) -> Root;
// }

// // Concrete implementations
// pub struct FailableWritable<Root, Value> {
//     func: Rc<dyn for<'a> Fn(&'a mut Root) -> Option<&'a mut Value>>,
// }

// pub struct Writable<Root, Value> {
//     get: Rc<dyn for<'a> Fn(&'a Root) -> Option<&'a Value>>,
//     get_mut: Rc<dyn for<'a> Fn(&'a mut Root) -> Option<&'a mut Value>>,
//     set_func: Rc<dyn Fn(&mut Root, Value)>,
// }

// pub struct WritableEmbed<Root, Value> {
//     get: Rc<dyn for<'a> Fn(&'a Root) -> Option<&'a Value>>,
//     get_mut: Rc<dyn for<'a> Fn(&'a mut Root) -> Option<&'a mut Value>>,
//     set_func: Rc<dyn Fn(&mut Root, Value)>,
//     embed: Rc<dyn Fn(Value) -> Root>,
// }

// // Implement ReadKeyPath for FailableWritable
// impl<Root, Value> ReadKeyPath<Root, Value> for FailableWritable<Root, Value> {
//     fn get<'a>(&self, _root: &'a Root) -> Option<&'a Value> {
//         None // FailableWritable can't provide read access
//     }
// }

// // Implement WriteKeyPath for FailableWritable
// impl<Root, Value> WriteKeyPath<Root, Value> for FailableWritable<Root, Value> {
//     fn get_mut<'a>(&self, root: &'a mut Root) -> Option<&'a mut Value> {
//         (self.func)(root)
//     }

//     fn set(&self, root: &mut Root, value: Value) {
//         if let Some(target) = self.get_mut(root) {
//             *target = value;
//         }
//     }
// }

// // Implement ReadKeyPath for Writable
// impl<Root, Value> ReadKeyPath<Root, Value> for Writable<Root, Value> {
//     fn get<'a>(&self, root: &'a Root) -> Option<&'a Value> {
//         (self.get)(root)
//     }
// }

// // Implement WriteKeyPath for Writable
// impl<Root, Value> WriteKeyPath<Root, Value> for Writable<Root, Value> {
//     fn get_mut<'a>(&self, root: &'a mut Root) -> Option<&'a mut Value> {
//         (self.get_mut)(root)
//     }

//     fn set(&self, root: &mut Root, value: Value) {
//         (self.set_func)(root, value)
//     }
// }

// // Implement ReadKeyPath for WritableEmbed
// impl<Root, Value> ReadKeyPath<Root, Value> for WritableEmbed<Root, Value> {
//     fn get<'a>(&self, root: &'a Root) -> Option<&'a Value> {
//         (self.get)(root)
//     }
// }

// // Implement WriteKeyPath for WritableEmbed
// impl<Root, Value> WriteKeyPath<Root, Value> for WritableEmbed<Root, Value> {
//     fn get_mut<'a>(&self, root: &'a mut Root) -> Option<&'a mut Value> {
//         (self.get_mut)(root)
//     }

//     fn set(&self, root: &mut Root, value: Value) {
//         (self.set_func)(root, value)
//     }
// }

// // Implement EmbedKeyPath for WritableEmbed
// impl<Root, Value> EmbedKeyPath<Root, Value> for WritableEmbed<Root, Value> {
//     fn embed(&self, value: Value) -> Root {
//         (self.embed)(value)
//     }
// }

// // Composition wrapper
// pub struct ComposedKeyPath<First, Second, Mid> {
//     first: First,
//     second: Second,
//     _phantom: std::marker::PhantomData<Mid>,
// }

// // Implement ReadKeyPath for ComposedKeyPath
// impl<Root, Mid, Value, First, Second> ReadKeyPath<Root, Value>
// for ComposedKeyPath<First, Second, Mid>
// where
//     First: ReadKeyPath<Root, Mid>,
//     Second: ReadKeyPath<Mid, Value>,
//     Mid: 'static,
// {
//     fn get<'a>(&self, root: &'a Root) -> Option<&'a Value> {
//         let mid = self.first.get(root)?;
//         self.second.get(mid)
//     }
// }

// // Implement WriteKeyPath for ComposedKeyPath
// impl<Root, Mid, Value, First, Second> WriteKeyPath<Root, Value>
// for ComposedKeyPath<First, Second, Mid>
// where
//     First: WriteKeyPath<Root, Mid>,
//     Second: WriteKeyPath<Mid, Value>,
//     Mid: 'static,
// {
//     fn get_mut<'a>(&self, root: &'a mut Root) -> Option<&'a mut Value> {
//         let mid = self.first.get_mut(root)?;
//         self.second.get_mut(mid)
//     }

//     fn set(&self, root: &mut Root, value: Value) {
//         if let Some(mid) = self.first.get_mut(root) {
//             self.second.set(mid, value);
//         }
//     }
// }

// // Composition trait
// pub trait Compose<Root, Mid, Value>: Sized + ReadKeyPath<Root, Mid> {
//     fn then<Second>(self, second: Second) -> ComposedKeyPath<Self, Second, Mid>
//     where
//         Second: ReadKeyPath<Mid, Value>;
// }

// // Blanket implementation
// impl<Root, Mid, Value, First> Compose<Root, Mid, Value> for First
// where
//     First: Sized + ReadKeyPath<Root, Mid>,
// {
//     fn then<Second>(self, second: Second) -> ComposedKeyPath<Self, Second, Mid>
//     where
//         Second: ReadKeyPath<Mid, Value>,
//     {
//         ComposedKeyPath {
//             first: self,
//             second,
//             _phantom: std::marker::PhantomData,
//         }
//     }
// }

// // Builder functions
// impl<Root, Value> FailableWritable<Root, Value> {
//     pub fn new(func: impl for<'a> Fn(&'a mut Root) -> Option<&'a mut Value> + 'static) -> Self {
//         Self { func: Rc::new(func) }
//     }
// }

// impl<Root, Value> Writable<Root, Value> {
//     pub fn new(
//         get: impl for<'a> Fn(&'a Root) -> Option<&'a Value> + 'static,
//         get_mut: impl for<'a> Fn(&'a mut Root) -> Option<&'a mut Value> + 'static,
//         set: impl Fn(&mut Root, Value) + 'static,
//     ) -> Self {
//         Self {
//             get: Rc::new(get),
//             get_mut: Rc::new(get_mut),
//             set_func: Rc::new(set),
//         }
//     }
// }

