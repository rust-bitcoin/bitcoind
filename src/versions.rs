pub const HAS_FEATURE: bool = cfg!(any(
    feature = "23_0",
    feature = "22_0",
    feature = "0_21_1",
    feature = "0_21_0",
    feature = "0_20_1",
    feature = "0_20_0",
    feature = "0_19_1",
    feature = "0_19_0_1",
    feature = "0_18_1",
    feature = "0_18_0",
    feature = "0_17_1",
));

#[cfg(not(any(
    feature = "23_0",
    feature = "22_0",
    feature = "0_21_1",
    feature = "0_21_0",
    feature = "0_20_1",
    feature = "0_20_0",
    feature = "0_19_1",
    feature = "0_19_0_1",
    feature = "0_18_1",
    feature = "0_18_0",
    feature = "0_17_1",
)))]
pub const VERSION: &str = "N/A";

#[cfg(feature = "23_0")]
pub const VERSION: &str = "23.0";

#[cfg(feature = "22_0")]
pub const VERSION: &str = "22.0";

#[cfg(feature = "0_21_1")]
pub const VERSION: &str = "0.21.1";

#[cfg(feature = "0_21_0")]
pub const VERSION: &str = "0.21.0";

#[cfg(feature = "0_20_1")]
pub const VERSION: &str = "0.20.1";

#[cfg(feature = "0_20_0")]
pub const VERSION: &str = "0.20.0";

#[cfg(feature = "0_19_1")]
pub const VERSION: &str = "0.19.1";

#[cfg(feature = "0_19_0_1")]
pub const VERSION: &str = "0.19.0.1";

#[cfg(feature = "0_18_1")]
pub const VERSION: &str = "0.18.1";

#[cfg(feature = "0_18_0")]
pub const VERSION: &str = "0.18.0";

#[cfg(feature = "0_17_1")]
pub const VERSION: &str = "0.17.1";
