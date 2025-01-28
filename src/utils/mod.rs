/// Heap collections which might be used for internal storage or I/O tasks.
#[cfg(feature = "alloc")]
pub mod heap;
/// Stack collections which might be used for internal storage or I/O tasks.
#[cfg(not(feature = "alloc"))]
pub mod stack;
/// All fixed capacity strings.
#[cfg(not(feature = "alloc"))]
pub mod strings;
