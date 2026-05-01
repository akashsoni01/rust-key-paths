use criterion::{black_box, criterion_group, criterion_main, Criterion};
use key_paths_derive::Kp;

#[derive(Debug, Kp, Default, Clone)]
struct SomeComplexStruct {
    scsf: Box<SomeOtherStruct>,
}

#[derive(Debug, Kp, Default, Clone)]
struct SomeOtherStruct {
    sosf: OneMoreStruct,
}

#[derive(Debug, Kp, Clone)]
enum SomeEnum {
    A(String),
    B(DarkStruct),
}

impl Default for SomeEnum {
    fn default() -> Self {
        SomeEnum::A(String::new())
    }
}

#[derive(Debug, Kp, Default, Clone)]
struct OneMoreStruct {
    omsf: String,
    omse: SomeEnum,
}

#[derive(Debug, Kp, Default, Clone)]
struct DarkStruct {
    dsf: String,
}

fn init_via_keypaths() -> SomeComplexStruct {
    let mut root = SomeComplexStruct::default();
    SomeComplexStruct::scsf()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omsf())
        .get_mut(&mut root)
        .map(|s| *s = "omsf_value".to_string());
    SomeComplexStruct::scsf()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omse())
        .get_mut(&mut root)
        .map(|e| *e = SomeEnum::B(DarkStruct::default()));
    SomeComplexStruct::scsf()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omse())
        .then(SomeEnum::b())
        .then(DarkStruct::dsf())
        .get_mut(&mut root)
        .map(|s| *s = "dark_value".to_string());
    root
}

fn read_keypath(instance: &SomeComplexStruct) -> Option<&String> {
    SomeComplexStruct::scsf()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omse())
        .then(SomeEnum::b())
        .then(DarkStruct::dsf())
        .get(instance)
}

fn read_unwrap(instance: &SomeComplexStruct) -> &String {
    match &instance.scsf.sosf.omse {
        SomeEnum::B(ds) => &ds.dsf,
        SomeEnum::A(_) => unreachable!("fixture always sets B variant"),
    }
}

fn read_as_ref_map(instance: &SomeComplexStruct) -> Option<&String> {
    Some(&instance.scsf)
        .map(|s| &s.sosf)
        .map(|o| &o.omse)
        .and_then(|e| match e {
            SomeEnum::B(ds) => Some(&ds.dsf),
            SomeEnum::A(_) => None,
        })
}

fn read_question_operator(instance: &SomeComplexStruct) -> Option<&String> {
    fn inner(root: &SomeComplexStruct) -> Option<&String> {
        let scsf = Some(&root.scsf)?;
        let sosf = Some(&scsf.sosf)?;
        match &sosf.omse {
            SomeEnum::B(ds) => Some(&ds.dsf),
            SomeEnum::A(_) => None,
        }
    }
    inner(instance)
}

fn write_keypath(instance: &mut SomeComplexStruct) -> bool {
    SomeComplexStruct::scsf()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omse())
        .then(SomeEnum::b())
        .then(DarkStruct::dsf())
        .get_mut(instance)
        .map(|val| {
            *val = "changed".to_string();
            val.is_empty()
        })
        .unwrap_or(false)
}

fn write_unwrap(instance: &mut SomeComplexStruct) -> bool {
    let dsf = match &mut instance.scsf.sosf.omse {
        SomeEnum::B(ds) => &mut ds.dsf,
        SomeEnum::A(_) => unreachable!("fixture always sets B variant"),
    };
    *dsf = "changed".to_string();
    dsf.is_empty()
}

fn write_as_ref_map(instance: &mut SomeComplexStruct) -> bool {
    Some(&mut instance.scsf)
        .map(|s| &mut s.sosf)
        .map(|o| &mut o.omse)
        .and_then(|e| match e {
            SomeEnum::B(ds) => Some(&mut ds.dsf),
            SomeEnum::A(_) => None,
        })
        .map(|v| {
            *v = "changed".to_string();
            v.is_empty()
        })
        .unwrap_or(false)
}

fn write_question_operator(instance: &mut SomeComplexStruct) -> bool {
    fn inner(root: &mut SomeComplexStruct) -> Option<bool> {
        let scsf = Some(&mut root.scsf)?;
        let sosf = Some(&mut scsf.sosf)?;
        let value = match &mut sosf.omse {
            SomeEnum::B(ds) => Some(&mut ds.dsf),
            SomeEnum::A(_) => None,
        }?;
        *value = "changed".to_string();
        Some(value.is_empty())
    }
    inner(instance).unwrap_or(false)
}

#[cfg(feature = "arc_swap_1_9_1")]
mod arc_swap_keypath {
    use std::sync::Arc;

    use arc_swap::ArcSwap;
    use criterion::{black_box, Criterion};
    use key_paths_derive::Kp;
    use rust_key_paths::ChainExt;

    #[derive(Debug, Kp, Default)]
    struct SomeComplexStruct {
        scsf: Box<SomeOtherStruct>,
    }

    #[derive(Debug, Kp)]
    struct SomeOtherStruct {
        sosf: arc_swap::ArcSwap<OneMoreStruct>,
    }

    impl Default for SomeOtherStruct {
        fn default() -> Self {
            Self {
                sosf: ArcSwap::from_pointee(OneMoreStruct::default()),
            }
        }
    }

    impl Clone for SomeOtherStruct {
        fn clone(&self) -> Self {
            Self {
                sosf: ArcSwap::new(self.sosf.load_full()),
            }
        }
    }

    #[derive(Debug, Kp, Clone)]
    enum SomeEnum {
        A(String),
        B(DarkStruct),
    }

    impl Default for SomeEnum {
        fn default() -> Self {
            SomeEnum::A(String::new())
        }
    }

    #[derive(Debug, Kp, Default, Clone)]
    struct OneMoreStruct {
        omsf: String,
        omse: SomeEnum,
    }

    #[derive(Debug, Kp, Default, Clone)]
    struct DarkStruct {
        dsf: String,
    }

    fn init_via_keypaths() -> SomeComplexStruct {
        let mut root = SomeComplexStruct::default();
        SomeComplexStruct::scsf()
            .then_lock::<_, OneMoreStruct, _, _, _, _, _, _, _, _, _, _, _, _>(
                SomeOtherStruct::sosf(),
            )
            .then(OneMoreStruct::omsf())
            .get_mut(&mut root)
            .map(|s| *s = "omsf_value".to_string());
        SomeComplexStruct::scsf()
            .then_lock::<_, OneMoreStruct, _, _, _, _, _, _, _, _, _, _, _, _>(
                SomeOtherStruct::sosf(),
            )
            .then(OneMoreStruct::omse())
            .get_mut(&mut root)
            .map(|e| *e = SomeEnum::B(DarkStruct::default()));
        SomeComplexStruct::scsf()
            .then_lock::<_, OneMoreStruct, _, _, _, _, _, _, _, _, _, _, _, _>(
                SomeOtherStruct::sosf(),
            )
            .then(OneMoreStruct::omse())
            .then(SomeEnum::b())
            .then(DarkStruct::dsf())
            .get_mut(&mut root)
            .map(|s| *s = "dark_value".to_string());
        root
    }

    fn read_keypath(instance: &SomeComplexStruct) -> Option<&String> {
        SomeComplexStruct::scsf()
            .then_lock::<_, OneMoreStruct, _, _, _, _, _, _, _, _, _, _, _, _>(
                SomeOtherStruct::sosf(),
            )
            .then(OneMoreStruct::omse())
            .then(SomeEnum::b())
            .then(DarkStruct::dsf())
            .get(instance)
    }

    /// Materializes the leaf string (snapshot from [`ArcSwap`] does not outlive this call).
    fn read_unwrap(instance: &SomeComplexStruct) -> String {
        let snap = instance.scsf.sosf.load_full();
        match &snap.omse {
            SomeEnum::B(ds) => ds.dsf.clone(),
            SomeEnum::A(_) => unreachable!("fixture always sets B variant"),
        }
    }

    fn read_as_ref_map(instance: &SomeComplexStruct) -> Option<String> {
        let snap = instance.scsf.sosf.load_full();
        match &snap.omse {
            SomeEnum::B(ds) => Some(ds.dsf.clone()),
            SomeEnum::A(_) => None,
        }
    }

    fn read_question_operator(instance: &SomeComplexStruct) -> Option<String> {
        let snap = instance.scsf.sosf.load_full();
        match &snap.omse {
            SomeEnum::B(ds) => Some(ds.dsf.clone()),
            SomeEnum::A(_) => None,
        }
    }

    fn write_keypath(instance: &mut SomeComplexStruct) -> bool {
        SomeComplexStruct::scsf()
            .then_lock::<_, OneMoreStruct, _, _, _, _, _, _, _, _, _, _, _, _>(
                SomeOtherStruct::sosf(),
            )
            .then(OneMoreStruct::omse())
            .then(SomeEnum::b())
            .then(DarkStruct::dsf())
            .get_mut(instance)
            .map(|val| {
                *val = "changed".to_string();
                val.is_empty()
            })
            .unwrap_or(false)
    }

    fn write_unwrap(instance: &mut SomeComplexStruct) -> bool {
        let mut arc = instance.scsf.sosf.load_full();
        let inner = Arc::make_mut(&mut arc);
        let dsf = match &mut inner.omse {
            SomeEnum::B(ds) => &mut ds.dsf,
            SomeEnum::A(_) => unreachable!("fixture always sets B variant"),
        };
        *dsf = "changed".to_string();
        let empty = dsf.is_empty();
        instance.scsf.sosf.store(arc);
        empty
    }

    fn write_as_ref_map(instance: &mut SomeComplexStruct) -> bool {
        let mut arc = instance.scsf.sosf.load_full();
        let inner = Arc::make_mut(&mut arc);
        let r = match &mut inner.omse {
            SomeEnum::B(ds) => Some(&mut ds.dsf),
            SomeEnum::A(_) => None,
        };
        let empty = r
            .map(|v| {
                *v = "changed".to_string();
                v.is_empty()
            })
            .unwrap_or(false);
        instance.scsf.sosf.store(arc);
        empty
    }

    fn write_question_operator(instance: &mut SomeComplexStruct) -> bool {
        let mut arc = instance.scsf.sosf.load_full();
        let inner = Arc::make_mut(&mut arc);
        let empty = match &mut inner.omse {
            SomeEnum::B(ds) => {
                ds.dsf = "changed".to_string();
                ds.dsf.is_empty()
            }
            SomeEnum::A(_) => false,
        };
        instance.scsf.sosf.store(arc);
        empty
    }

    pub fn bench_arc_swap_keypath(c: &mut Criterion) {
        let mut read_group = c.benchmark_group("arc_swap_keypath_read");
        let instance = init_via_keypaths();
        let kp = SomeComplexStruct::scsf()
            .then_lock::<_, OneMoreStruct, _, _, _, _, _, _, _, _, _, _, _, _>(
                SomeOtherStruct::sosf(),
            )
            .then(OneMoreStruct::omse())
            .then(SomeEnum::b())
            .then(DarkStruct::dsf());

        read_group.bench_function("keypath", |b| {
            b.iter(|| black_box(read_keypath(black_box(&instance))))
        });
        read_group.bench_function("unwrap", |b| {
            b.iter(|| black_box(read_unwrap(black_box(&instance))))
        });
        read_group.bench_function("as_ref_map", |b| {
            b.iter(|| black_box(read_as_ref_map(black_box(&instance))))
        });
        read_group.bench_function("question_operator", |b| {
            b.iter(|| black_box(read_question_operator(black_box(&instance))))
        });
        read_group.bench_function("kp_size_bytes", |b| {
            b.iter(|| black_box(std::mem::size_of_val(black_box(&kp))))
        });
        read_group.finish();

        let mut write_group = c.benchmark_group("arc_swap_keypath_write");
        write_group.bench_function("keypath", |b| {
            b.iter(|| {
                let mut inst = init_via_keypaths();
                black_box(write_keypath(black_box(&mut inst)))
            })
        });
        write_group.bench_function("unwrap", |b| {
            b.iter(|| {
                let mut inst = init_via_keypaths();
                black_box(write_unwrap(black_box(&mut inst)))
            })
        });
        write_group.bench_function("as_ref_map", |b| {
            b.iter(|| {
                let mut inst = init_via_keypaths();
                black_box(write_as_ref_map(black_box(&mut inst)))
            })
        });
        write_group.bench_function("question_operator", |b| {
            b.iter(|| {
                let mut inst = init_via_keypaths();
                black_box(write_question_operator(black_box(&mut inst)))
            })
        });
        write_group.finish();
    }
}

fn bench_box_keypath(c: &mut Criterion) {
    let mut read_group = c.benchmark_group("box_keypath_read");
    let instance = init_via_keypaths();
    let kp = SomeComplexStruct::scsf()
        .then(SomeOtherStruct::sosf())
        .then(OneMoreStruct::omse())
        .then(SomeEnum::b())
        .then(DarkStruct::dsf());

    read_group.bench_function("keypath", |b| b.iter(|| black_box(read_keypath(black_box(&instance)))));
    read_group.bench_function("unwrap", |b| b.iter(|| black_box(read_unwrap(black_box(&instance)))));
    read_group.bench_function("as_ref_map", |b| {
        b.iter(|| black_box(read_as_ref_map(black_box(&instance))))
    });
    read_group.bench_function("question_operator", |b| {
        b.iter(|| black_box(read_question_operator(black_box(&instance))))
    });
    read_group.bench_function("kp_size_bytes", |b| {
        b.iter(|| black_box(std::mem::size_of_val(black_box(&kp))))
    });
    read_group.finish();

    let mut write_group = c.benchmark_group("box_keypath_write");
    write_group.bench_function("keypath", |b| {
        b.iter(|| {
            let mut inst = init_via_keypaths();
            black_box(write_keypath(black_box(&mut inst)))
        })
    });
    write_group.bench_function("unwrap", |b| {
        b.iter(|| {
            let mut inst = init_via_keypaths();
            black_box(write_unwrap(black_box(&mut inst)))
        })
    });
    write_group.bench_function("as_ref_map", |b| {
        b.iter(|| {
            let mut inst = init_via_keypaths();
            black_box(write_as_ref_map(black_box(&mut inst)))
        })
    });
    write_group.bench_function("question_operator", |b| {
        b.iter(|| {
            let mut inst = init_via_keypaths();
            black_box(write_question_operator(black_box(&mut inst)))
        })
    });
    write_group.finish();
}

criterion_group!(benches, bench_box_keypath);

#[cfg(feature = "arc_swap_1_9_1")]
criterion_group!(arc_swap_benches, arc_swap_keypath::bench_arc_swap_keypath);

#[cfg(not(feature = "arc_swap_1_9_1"))]
criterion_main!(benches);

#[cfg(feature = "arc_swap_1_9_1")]
criterion_main!(benches, arc_swap_benches);
