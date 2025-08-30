// #[derive(Clone)]
pub enum KeyPathKind<Root, Value> {
    Readable(Box<dyn for<'a> Fn(&'a Root) -> &'a Value>),
    Writable(Box<dyn for<'a> Fn(&'a mut Root) -> &'a mut Value>),
    FailableReadable(Box<dyn for<'a> Fn(&'a Root) -> Option<&'a Value>>),
    FailableWritable(Box<dyn for<'a> Fn(&'a mut Root) -> Option<&'a mut Value>>),
}

impl<Root, Value> KeyPathKind<Root, Value> {
    pub fn readable(get: impl for<'a> Fn(&'a Root) -> &'a Value + 'static) -> Self {
        Self::Readable(Box::new(get))
    }

    pub fn writable(get_mut: impl for<'a> Fn(&'a mut Root) -> &'a mut Value + 'static) -> Self {
        Self::Writable(Box::new(get_mut))
    }

    pub fn failable_readable(
        get: impl for<'a> Fn(&'a Root) -> Option<&'a Value> + 'static,
    ) -> Self {
        Self::FailableReadable(Box::new(get))
    }

    pub fn failable_writable(
        get_mut: impl for<'a> Fn(&'a mut Root) -> Option<&'a mut Value> + 'static,
    ) -> Self {
        Self::FailableWritable(Box::new(get_mut))
    }
}

impl<Root, Mid> KeyPathKind<Root, Mid> 
where 
    Root: 'static,
    Mid: 'static,
{
    pub fn compose<Value>(
        self,
        mid: KeyPathKind<Mid, Value>,
    ) -> KeyPathKind<Root, Value> 
    where 
        Value: 'static,
    {
        use KeyPathKind::*;

        match (self, mid) {
            (Readable(f1), Readable(f2)) => {
                Readable(Box::new(move |r| f2(f1(r))))
            }

            (Writable(f1), Writable(f2)) => {
                Writable(Box::new(move |r| f2(f1(r))))
            }

            (FailableReadable(f1), Readable(f2)) => {
                FailableReadable(Box::new(move |r| f1(r).map(|m| f2(m))))
            }

            (Readable(f1), FailableReadable(f2)) => {
                FailableReadable(Box::new(move |r| f2(f1(r))))
            }

            (FailableReadable(f1), FailableReadable(f2)) => {
                FailableReadable(Box::new(move |r| f1(r).and_then(|m| f2(m))))
            }

            (FailableWritable(f1), Writable(f2)) => {
                FailableWritable(Box::new(move |r| f1(r).map(|m| f2(m))))
            }

            (Writable(f1), FailableWritable(f2)) => {
                FailableWritable(Box::new(move |r| f2(f1(r))))
            }

            (FailableWritable(f1), FailableWritable(f2)) => {
                FailableWritable(Box::new(move |r| f1(r).and_then(|m| f2(m))))
            }

            (a, b) => panic!(
                "Unsupported composition: {:?} then {:?}",
                kind_name(&a),
                kind_name(&b)
            ),
        }
    }
}

fn kind_name<Root, Value>(k: &KeyPathKind<Root, Value>) -> &'static str {
    use KeyPathKind::*;
    match k {
        Readable(_) => "Readable",
        Writable(_) => "Writable",
        FailableReadable(_) => "FailableReadable",
        FailableWritable(_) => "FailableWritable",
    }
}
