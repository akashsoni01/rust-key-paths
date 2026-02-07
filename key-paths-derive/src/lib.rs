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
    // Arc with synchronization primitives (default)
    StdArcMutex,
    StdArcRwLock,
    OptionStdArcMutex,
    OptionStdArcRwLock,
    // Synchronization primitives default
    StdMutex,
    StdRwLock,
    OptionStdMutex,
    OptionStdRwLock,
    // Synchronization primitives (parking_lot)
    Mutex,
    RwLock,
    OptionMutex,
    OptionRwLock,
    // Synchronization primitives (tokio::sync - requires tokio feature)
    TokioMutex,
    TokioRwLock,
    // parking_lot
    ArcMutex,
    ArcRwLock,
    OptionArcMutex,
    OptionArcRwLock,
    // Arc with synchronization primitives (tokio::sync - requires tokio feature)
    TokioArcMutex,
    TokioArcRwLock,
    OptionTokioArcMutex,
    OptionTokioArcRwLock,
    // Tagged types
    Tagged,
}

