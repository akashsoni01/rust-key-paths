use std::rc::Rc;

// #[derive(Clone)]
pub enum KeyPaths<Root, Value> {
    Readable(Rc<dyn for<'a> Fn(&'a Root) -> &'a Value>),
    Prism {
        extract: Rc<dyn for<'a> Fn(&'a Root) -> Option<&'a Value>>,
        embed: Rc<dyn Fn(Value) -> Root>,
    },
    FailableReadable(Rc<dyn for<'a> Fn(&'a Root) -> Option<&'a Value>>),

    Writable(Rc<dyn for<'a> Fn(&'a mut Root) -> &'a mut Value>),
    FailableWritable(Rc<dyn for<'a> Fn(&'a mut Root) -> Option<&'a mut Value>>),
    PrismMut {
        extract: Rc<dyn for<'a> Fn(&'a Root) -> Option<&'a Value>>,
        extract_mut: Rc<dyn for<'a> Fn(&'a mut Root) -> Option<&'a mut Value>>,
        embed: Rc<dyn Fn(Value) -> Root>,
    },
}

impl<Root, Value> KeyPaths<Root, Value> {
    pub fn readable(get: impl for<'a> Fn(&'a Root) -> &'a Value + 'static) -> Self {
        Self::Readable(Rc::new(get))
    }

    pub fn writable(get_mut: impl for<'a> Fn(&'a mut Root) -> &'a mut Value + 'static) -> Self {
        Self::Writable(Rc::new(get_mut))
    }

    pub fn failable_readable(
        get: impl for<'a> Fn(&'a Root) -> Option<&'a Value> + 'static,
    ) -> Self {
        Self::FailableReadable(Rc::new(get))
    }

    pub fn failable_writable(
        get_mut: impl for<'a> Fn(&'a mut Root) -> Option<&'a mut Value> + 'static,
    ) -> Self {
        Self::FailableWritable(Rc::new(get_mut))
    }

    pub fn prism(
        embed: impl Fn(Value) -> Root + 'static,
        extract: impl for<'a> Fn(&'a Root) -> Option<&'a Value> + 'static,
    ) -> Self {
        Self::Prism {
            extract: Rc::new(extract),
            embed: Rc::new(embed),
        }
    }

    pub fn prism_mut(
        embed: impl Fn(Value) -> Root + 'static,
        extract: impl for<'a> Fn(&'a Root) -> Option<&'a Value> + 'static,
        extract_mut: impl for<'a> Fn(&'a mut Root) -> Option<&'a mut Value> + 'static,
    ) -> Self {
        Self::PrismMut {
            extract: Rc::new(extract),
            embed: Rc::new(embed),
            extract_mut: Rc::new(extract_mut),
        }
    }
}

impl<Root, Value> KeyPaths<Root, Value> {
    /// Get an immutable reference if possible
    pub fn get<'a>(&'a self, root: &'a Root) -> Option<&'a Value> {
        match self {
            KeyPaths::Readable(f) => Some(f(root)),
            KeyPaths::Writable(_) => None, // Writable requires mut
            KeyPaths::FailableReadable(f) => f(root),
            KeyPaths::FailableWritable(_) => None, // needs mut
            KeyPaths::Prism { extract, .. } => extract(root),
            KeyPaths::PrismMut { extract, .. } => extract(root),
        }
    }

    /// Get a mutable reference if possible
    pub fn get_mut<'a>(&'a self, root: &'a mut Root) -> Option<&'a mut Value> {
        match self {
            KeyPaths::Readable(_) => None, // immutable only
            KeyPaths::Writable(f) => Some(f(root)),
            KeyPaths::FailableReadable(_) => None, // immutable only
            KeyPaths::FailableWritable(f) => f(root),
            KeyPaths::PrismMut { extract_mut, .. } => extract_mut(root),
            _ => None,
        }
    }

    pub fn embed(&self, value: Value) -> Option<Root>
    where
        Value: Clone,
    {
        match self {
            KeyPaths::Prism { embed, .. } => Some(embed(value)),
            _ => None,
        }
    }

    pub fn embed_mut(&self, value: Value) -> Option<Root>
    where
        Value: Clone,
    {
        match self {
            KeyPaths::PrismMut { embed, .. } => Some(embed(value)),
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
    pub fn into_iter<T>(self, root: Root) -> Option<<Value as IntoIterator>::IntoIter>
    where
        Value: IntoIterator<Item = T> + Clone,
    {
        match self {
            KeyPaths::Readable(f) => Some(f(&root).clone().into_iter()), // requires Clone
            KeyPaths::Writable(_) => None,
            KeyPaths::FailableReadable(f) => f(&root).map(|v| v.clone().into_iter()),
            KeyPaths::FailableWritable(_) => None,
            KeyPaths::Prism { extract, .. } => extract(&root).map(|v| v.clone().into_iter()),
            KeyPaths::PrismMut { extract, .. } => extract(&root).map(|v| v.clone().into_iter()),
        }
    }
}

impl<Root, Mid> KeyPaths<Root, Mid>
where
    Root: 'static,
    Mid: 'static,
{
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

            (Prism { extract, .. }, Readable(f2)) => {
                FailableReadable(Rc::new(move |r| extract(r).map(|m| f2(m))))
            }

            (Prism { extract, .. }, FailableReadable(f2)) => {
                FailableReadable(Rc::new(move |r| extract(r).and_then(|m| f2(m))))
            }

            (PrismMut { extract, .. }, Readable(f2)) => {
                FailableReadable(Rc::new(move |r| extract(r).map(|m| f2(m))))
            }

            (PrismMut { extract, .. }, FailableReadable(f2)) => {
                FailableReadable(Rc::new(move |r| extract(r).and_then(|m| f2(m))))
            }

            (PrismMut { extract_mut, .. }, Writable(f2)) => {
                FailableWritable(Rc::new(move |r| extract_mut(r).map(|m| f2(m))))
            }

            (PrismMut { extract_mut, .. }, FailableWritable(f2)) => {
                FailableWritable(Rc::new(move |r| extract_mut(r).and_then(|m| f2(m))))
            }

            (
                Prism {
                    extract: ex1,
                    embed: em1,
                },
                Prism {
                    extract: ex2,
                    embed: em2,
                },
            ) => Prism {
                extract: Rc::new(move |r| ex1(r).and_then(|m| ex2(m))),
                embed: Rc::new(move |v| em1(em2(v))),
            },

            (
                PrismMut {
                    extract: ex1,
                    extract_mut,
                    embed: em1,
                },
                Prism {
                    extract: ex2,
                    embed: em2,
                },
            ) => Prism {
                extract: Rc::new(move |r| ex1(r).and_then(|m| ex2(m))),
                embed: Rc::new(move |v| em1(em2(v))),
            },

            (
                PrismMut {
                    extract: ex1,
                    extract_mut: exm1,
                    embed: em1,
                },
                PrismMut {
                    extract: ex2,
                    extract_mut: exm2,
                    embed: em2,
                },
            ) => PrismMut {
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
        Prism { .. } => "Prism",
        PrismMut { .. } => "PrismMut",
    }
}
