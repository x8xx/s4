#[cfg(feature="logger-std")]
pub mod std;
#[cfg(feature="logger-std")]
pub use self::std::*;
