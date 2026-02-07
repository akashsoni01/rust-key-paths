use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, Data, DeriveInput, Fields, Type, parse_macro_input, spanned::Spanned};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WrapperKind {
    None,
    Option,
    Box,
    Rc,
    Arc,
    Vec,
    HashMap,
    BTreeMap,
    HashSet,
    BTreeSet,
    VecDeque,
    LinkedList,
    BinaryHeap,
    // Error handling containers
    Result,
    // Synchronization primitives (std::sync - requires explicit std::sync:: prefix)
    StdMutex,
    StdRwLock,
    // Synchronization primitives (parking_lot - default when no prefix)
    Mutex,
    RwLock,
    // Synchronization primitives (tokio::sync - requires tokio feature)
    TokioMutex,
    TokioRwLock,
    // Reference counting with weak references
    Weak,
    // String types (currently unused)
    // String,
    // OsString,
    // PathBuf,
    // Nested container support
    OptionBox,
    OptionRc,
    OptionArc,
    BoxOption,
    RcOption,
    ArcOption,
    VecOption,
    OptionVec,
    HashMapOption,
    OptionHashMap,
    // Arc with synchronization primitives (std::sync - requires explicit std::sync:: prefix)
    StdArcMutex,
    StdArcRwLock,
    // Arc with synchronization primitives (parking_lot - default when no prefix)
    ArcMutex,
    ArcRwLock,
    // Arc with synchronization primitives (tokio::sync - requires tokio feature)
    TokioArcMutex,
    TokioArcRwLock,
    // Tagged types
    Tagged,
}

