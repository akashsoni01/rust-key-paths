// #[derive(Clone)]
pub enum KeyPathKind<Root, Value> {
    Readable(Box<dyn for<'a> Fn(&'a Root) -> &'a Value>),
    Writable(Box<dyn for<'a> Fn(&'a mut Root) -> &'a mut Value>),
    FailableReadable(Box<dyn for<'a> Fn(&'a Root) -> Option<&'a Value>>),
    FailableWritable(Box<dyn for<'a> Fn(&'a mut Root) -> Option<&'a mut Value>>),
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
            // --- Readable + Readable = Readable
            (Readable(f1), Readable(f2)) => {
                Readable(Box::new(move |r| f2(f1(r))))
            }

            // --- Writable + Writable = Writable
            (Writable(f1), Writable(f2)) => {
                Writable(Box::new(move |r| f2(f1(r))))
            }

            // --- FailableReadable + Readable = FailableReadable
            (FailableReadable(f1), Readable(f2)) => {
                FailableReadable(Box::new(move |r| f1(r).map(|m| f2(m))))
            }

            // --- Readable + FailableReadable = FailableReadable
            (Readable(f1), FailableReadable(f2)) => {
                FailableReadable(Box::new(move |r| f2(f1(r))))
            }

            // --- FailableReadable + FailableReadable = FailableReadable
            (FailableReadable(f1), FailableReadable(f2)) => {
                FailableReadable(Box::new(move |r| f1(r).and_then(|m| f2(m))))
            }

            // --- FailableWritable + Writable = FailableWritable
            (FailableWritable(f1), Writable(f2)) => {
                FailableWritable(Box::new(move |r| f1(r).map(|m| f2(m))))
            }

            // --- Writable + FailableWritable = FailableWritable
            (Writable(f1), FailableWritable(f2)) => {
                FailableWritable(Box::new(move |r| f2(f1(r))))
            }

            // --- FailableWritable + FailableWritable = FailableWritable
            (FailableWritable(f1), FailableWritable(f2)) => {
                FailableWritable(Box::new(move |r| f1(r).and_then(|m| f2(m))))
            }

            // --- Readable + Writable (nonsense: mismatch) 
            // --- or anything else: panic for now
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
