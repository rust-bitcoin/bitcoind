pub const HAS_FEATURE: bool = cfg!(any(
    feature = "0.21.1",
    feature = "0.21.0",
    feature = "0.20.1",
    feature = "0.20.0",
    feature = "0.19.1",
    feature = "0.19.0.1",
    feature = "0.18.1",
    feature = "0.18.0",
    feature = "0.17.1",
));

#[cfg(not(any(
    feature = "0.21.1",
    feature = "0.21.0",
    feature = "0.20.1",
    feature = "0.20.0",
    feature = "0.19.1",
    feature = "0.19.0.1",
    feature = "0.18.1",
    feature = "0.18.0",
    feature = "0.17.1",
)))]
pub const VERSION: &str = "N/A";

#[cfg(feature = "0.21.1")]
pub const VERSION: &str = "0.21.1";

#[cfg(feature = "0.21.0")]
pub const VERSION: &str = "0.21.0";

#[cfg(feature = "0.20.1")]
pub const VERSION: &str = "0.20.1";

#[cfg(feature = "0.20.0")]
pub const VERSION: &str = "0.20.0";

#[cfg(feature = "0.19.1")]
pub const VERSION: &str = "0.19.1";

#[cfg(feature = "0.19.0.1")]
pub const VERSION: &str = "0.19.0.1";

#[cfg(feature = "0.18.1")]
pub const VERSION: &str = "0.18.1";

#[cfg(feature = "0.18.0")]
pub const VERSION: &str = "0.18.0";

#[cfg(feature = "0.17.1")]
pub const VERSION: &str = "0.17.1";
