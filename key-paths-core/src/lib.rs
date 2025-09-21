use std::rc::Rc;

#[derive(Clone)]
/// Go to examples section to see the implementations
///
pub enum KeyPaths<Root, Value> {
    Readable(Rc<dyn for<'a> Fn(&'a Root) -> &'a Value>),
    ReadableEnum {
        extract: Rc<dyn for<'a> Fn(&'a Root) -> Option<&'a Value>>,
        embed: Rc<dyn Fn(Value) -> Root>,
    },
    FailableReadable(Rc<dyn for<'a> Fn(&'a Root) -> Option<&'a Value>>),

    Writable(Rc<dyn for<'a> Fn(&'a mut Root) -> &'a mut Value>),
    FailableWritable(Rc<dyn for<'a> Fn(&'a mut Root) -> Option<&'a mut Value>>),
    WritableEnum {
        extract: Rc<dyn for<'a> Fn(&'a Root) -> Option<&'a Value>>,
        extract_mut: Rc<dyn for<'a> Fn(&'a mut Root) -> Option<&'a mut Value>>,
        embed: Rc<dyn Fn(Value) -> Root>,
    },
}

impl<Root, Value> KeyPaths<Root, Value> {
    #[inline]
    pub fn readable(get: impl for<'a> Fn(&'a Root) -> &'a Value + 'static) -> Self {
        Self::Readable(Rc::new(get))
    }

    #[inline]
    pub fn writable(get_mut: impl for<'a> Fn(&'a mut Root) -> &'a mut Value + 'static) -> Self {
        Self::Writable(Rc::new(get_mut))
    }

    #[inline]
    pub fn failable_readable(
        get: impl for<'a> Fn(&'a Root) -> Option<&'a Value> + 'static,
    ) -> Self {
        Self::FailableReadable(Rc::new(get))
    }

    #[inline]
    pub fn failable_writable(
        get_mut: impl for<'a> Fn(&'a mut Root) -> Option<&'a mut Value> + 'static,
    ) -> Self {
        Self::FailableWritable(Rc::new(get_mut))
    }

    #[inline]
    pub fn readable_enum(
        embed: impl Fn(Value) -> Root + 'static,
        extract: impl for<'a> Fn(&'a Root) -> Option<&'a Value> + 'static,
    ) -> Self {
        Self::ReadableEnum {
            extract: Rc::new(extract),
            embed: Rc::new(embed),
        }
    }

    #[inline]
    pub fn writable_enum(
        embed: impl Fn(Value) -> Root + 'static,
        extract: impl for<'a> Fn(&'a Root) -> Option<&'a Value> + 'static,
        extract_mut: impl for<'a> Fn(&'a mut Root) -> Option<&'a mut Value> + 'static,
    ) -> Self {
        Self::WritableEnum {
            extract: Rc::new(extract),
            embed: Rc::new(embed),
            extract_mut: Rc::new(extract_mut),
        }
    }
}

impl<Root, Value> KeyPaths<Root, Value> {
    /// Get an immutable reference if possible
    #[inline]
    pub fn get<'a>(&'a self, root: &'a Root) -> Option<&'a Value> {
        match self {
            KeyPaths::Readable(f) => Some(f(root)),
            KeyPaths::Writable(_) => None, // Writable requires mut
            KeyPaths::FailableReadable(f) => f(root),
            KeyPaths::FailableWritable(_) => None, // needs mut
            KeyPaths::ReadableEnum { extract, .. } => extract(root),
            KeyPaths::WritableEnum { extract, .. } => extract(root),
        }
    }

    /// Get a mutable reference if possible
    #[inline]
    pub fn get_mut<'a>(&'a self, root: &'a mut Root) -> Option<&'a mut Value> {
        match self {
            KeyPaths::Readable(_) => None, // immutable only
            KeyPaths::Writable(f) => Some(f(root)),
            KeyPaths::FailableReadable(_) => None, // immutable only
            KeyPaths::FailableWritable(f) => f(root),
            KeyPaths::WritableEnum { extract_mut, .. } => extract_mut(root),
            _ => None,
        }
    }

    pub fn embed(&self, value: Value) -> Option<Root>
    where
        Value: Clone,
    {
        match self {
            KeyPaths::ReadableEnum { embed, .. } => Some(embed(value)),
            _ => None,
        }
    }

    pub fn embed_mut(&self, value: Value) -> Option<Root>
    where
        Value: Clone,
    {
        match self {
            KeyPaths::WritableEnum { embed, .. } => Some(embed(value)),
            _ => None,
        }
    }

    /// Iter over immutable references if `Value: IntoIterator`
    pub fn iter<'a, T>(&'a self, root: &'a Root) -> Option<<&'a Value as IntoIterator>::IntoIter>
    where
        &'a Value: IntoIterator<Item = &'a T>,
        T: 'a,
    {
        self.get(root).map(|v| v.into_iter())
    }

    /// Iter over mutable references if `&mut Value: IntoIterator`
    pub fn iter_mut<'a, T>(
        &'a self,
        root: &'a mut Root,
    ) -> Option<<&'a mut Value as IntoIterator>::IntoIter>
    where
        &'a mut Value: IntoIterator<Item = &'a mut T>,
        T: 'a,
    {
        self.get_mut(root).map(|v| v.into_iter())
    }

    /// Consume root and iterate if `Value: IntoIterator`
    #[inline]
    pub fn into_iter<T>(self, root: Root) -> Option<<Value as IntoIterator>::IntoIter>
    where
        Value: IntoIterator<Item = T> + Clone,
    {
        match self {
            KeyPaths::Readable(f) => Some(f(&root).clone().into_iter()), // requires Clone
            KeyPaths::Writable(_) => None,
            KeyPaths::FailableReadable(f) => f(&root).map(|v| v.clone().into_iter()),
            KeyPaths::FailableWritable(_) => None,
            KeyPaths::ReadableEnum { extract, .. } => extract(&root).map(|v| v.clone().into_iter()),
            KeyPaths::WritableEnum { extract, .. } => extract(&root).map(|v| v.clone().into_iter()),
        }
    }
}

impl<Root, Mid> KeyPaths<Root, Mid>
where
    Root: 'static,
    Mid: 'static,
{
    /// Alias for `compose` for ergonomic chaining.
    #[inline]
    pub fn then<Value>(self, mid: KeyPaths<Mid, Value>) -> KeyPaths<Root, Value>
    where
        Value: 'static,
    {
        self.compose(mid)
    }

    pub fn compose<Value>(self, mid: KeyPaths<Mid, Value>) -> KeyPaths<Root, Value>
    where
        Value: 'static,
    {
        use KeyPaths::*;

        match (self, mid) {
            (Readable(f1), Readable(f2)) => Readable(Rc::new(move |r| f2(f1(r)))),

            (Writable(f1), Writable(f2)) => Writable(Rc::new(move |r| f2(f1(r)))),

            (FailableReadable(f1), Readable(f2)) => {
                FailableReadable(Rc::new(move |r| f1(r).map(|m| f2(m))))
            }

            (Readable(f1), FailableReadable(f2)) => FailableReadable(Rc::new(move |r| f2(f1(r)))),

            (FailableReadable(f1), FailableReadable(f2)) => {
                FailableReadable(Rc::new(move |r| f1(r).and_then(|m| f2(m))))
            }

            (FailableWritable(f1), Writable(f2)) => {
                FailableWritable(Rc::new(move |r| f1(r).map(|m| f2(m))))
            }

            (Writable(f1), FailableWritable(f2)) => FailableWritable(Rc::new(move |r| f2(f1(r)))),

            (FailableWritable(f1), FailableWritable(f2)) => {
                FailableWritable(Rc::new(move |r| f1(r).and_then(|m| f2(m))))
            }
            (FailableReadable(f1), ReadableEnum { extract, .. }) => {
                FailableReadable(Rc::new(move |r| f1(r).and_then(|m| extract(m))))
            }
            // (ReadableEnum { extract, .. }, FailableReadable(f2)) => {
            //     FailableReadable(Rc::new(move |r| extract(r).map(|m| f2(m).unwrap())))
            // }
            (ReadableEnum { extract, .. }, Readable(f2)) => {
                FailableReadable(Rc::new(move |r| extract(r).map(|m| f2(m))))
            }

            (ReadableEnum { extract, .. }, FailableReadable(f2)) => {
                FailableReadable(Rc::new(move |r| extract(r).and_then(|m| f2(m))))
            }

            (WritableEnum { extract, .. }, Readable(f2)) => {
                FailableReadable(Rc::new(move |r| extract(r).map(|m| f2(m))))
            }

            (WritableEnum { extract, .. }, FailableReadable(f2)) => {
                FailableReadable(Rc::new(move |r| extract(r).and_then(|m| f2(m))))
            }

            (WritableEnum { extract_mut, .. }, Writable(f2)) => {
                FailableWritable(Rc::new(move |r| extract_mut(r).map(|m| f2(m))))
            }

            (
                FailableWritable(f_root_mid),
                WritableEnum {
                    extract_mut: exm_mid_val,
                    ..
                },
            ) => {
                FailableWritable(Rc::new(move |r: &mut Root| {
                    // First, apply the function that operates on Root.
                    // This will give us `Option<&mut Mid>`.
                    let intermediate_mid_ref = f_root_mid(r);

                    // Then, apply the function that operates on Mid.
                    // This will give us `Option<&mut Value>`.
                    intermediate_mid_ref.and_then(|intermediate_mid| exm_mid_val(intermediate_mid))
                }))
            }

            (WritableEnum { extract_mut, .. }, FailableWritable(f2)) => {
                FailableWritable(Rc::new(move |r| extract_mut(r).and_then(|m| f2(m))))
            }

            // New: Writable then WritableEnum => FailableWritable
            (Writable(f1), WritableEnum { extract_mut, .. }) => {
                FailableWritable(Rc::new(move |r: &mut Root| {
                    let mid: &mut Mid = f1(r);
                    extract_mut(mid)
                }))
            }

            (
                ReadableEnum {
                    extract: ex1,
                    embed: em1,
                },
                ReadableEnum {
                    extract: ex2,
                    embed: em2,
                },
            ) => ReadableEnum {
                extract: Rc::new(move |r| ex1(r).and_then(|m| ex2(m))),
                embed: Rc::new(move |v| em1(em2(v))),
            },

            (
                WritableEnum {
                    extract: ex1,
                    extract_mut,
                    embed: em1,
                },
                ReadableEnum {
                    extract: ex2,
                    embed: em2,
                },
            ) => ReadableEnum {
                extract: Rc::new(move |r| ex1(r).and_then(|m| ex2(m))),
                embed: Rc::new(move |v| em1(em2(v))),
            },

            (
                WritableEnum {
                    extract: ex1,
                    extract_mut: exm1,
                    embed: em1,
                },
                WritableEnum {
                    extract: ex2,
                    extract_mut: exm2,
                    embed: em2,
                },
            ) => WritableEnum {
                extract: Rc::new(move |r| ex1(r).and_then(|m| ex2(m))),
                extract_mut: Rc::new(move |r| exm1(r).and_then(|m| exm2(m))),
                embed: Rc::new(move |v| em1(em2(v))),
            },

            (a, b) => panic!(
                "Unsupported composition: {:?} then {:?}",
                kind_name(&a),
                kind_name(&b)
            ),
        }
    }
}

fn kind_name<Root, Value>(k: &KeyPaths<Root, Value>) -> &'static str {
    use KeyPaths::*;
    match k {
        Readable(_) => "Readable",
        Writable(_) => "Writable",
        FailableReadable(_) => "FailableReadable",
        FailableWritable(_) => "FailableWritable",
        ReadableEnum { .. } => "ReadableEnum",
        WritableEnum { .. } => "WritableEnum",
    }
}

// ===== Helper macros for enum case keypaths =====

#[macro_export]
macro_rules! readable_enum_macro {
    // Unit variant: Enum::Variant
    ($enum:path, $variant:ident) => {{
        $crate::KeyPaths::readable_enum(
            |_| <$enum>::$variant,
            |e: &$enum| match e {
                <$enum>::$variant => Some(&()),
                _ => None,
            },
        )
    }};
    // Single-field tuple variant: Enum::Variant(Inner)
    ($enum:path, $variant:ident($inner:ty)) => {{
        $crate::KeyPaths::readable_enum(
            |v: $inner| <$enum>::$variant(v),
            |e: &$enum| match e {
                <$enum>::$variant(v) => Some(v),
                _ => None,
            },
        )
    }};
}

#[macro_export]
macro_rules! writable_enum_macro {
    // Unit variant: Enum::Variant (creates prism to and from ())
    ($enum:path, $variant:ident) => {{
        $crate::KeyPaths::writable_enum(
            |_| <$enum>::$variant,
            |e: &$enum| match e {
                <$enum>::$variant => Some(&()),
                _ => None,
            },
            |e: &mut $enum| match e {
                <$enum>::$variant => Some(&mut ()),
                _ => None,
            },
        )
    }};
    // Single-field tuple variant: Enum::Variant(Inner)
    ($enum:path, $variant:ident($inner:ty)) => {{
        $crate::KeyPaths::writable_enum(
            |v: $inner| <$enum>::$variant(v),
            |e: &$enum| match e {
                <$enum>::$variant(v) => Some(v),
                _ => None,
            },
            |e: &mut $enum| match e {
                <$enum>::$variant(v) => Some(v),
                _ => None,
            },
        )
    }};
}
