#[cfg(feature = "24_0_1")]
pub const VERSION: &str = "24.0.1";

#[cfg(all(feature = "23_0", not(feature = "24_0_1")))]
pub const VERSION: &str = "23.0";

#[cfg(all(feature = "22_0", not(feature = "23_0")))]
pub const VERSION: &str = "22.0";

#[cfg(all(feature = "0_21_1", not(feature = "22_0")))]
pub const VERSION: &str = "0.21.1";

#[cfg(all(feature = "0_21_0", not(feature = "0_21_1")))]
pub const VERSION: &str = "0.21.0";

#[cfg(all(feature = "0_20_1", not(feature = "0_21_0")))]
pub const VERSION: &str = "0.20.1";

#[cfg(all(feature = "0_20_0", not(feature = "0_20_1")))]
pub const VERSION: &str = "0.20.0";

#[cfg(all(feature = "0_19_1", not(feature = "0_20_0")))]
pub const VERSION: &str = "0.19.1";

#[cfg(all(feature = "0_19_0_1", not(feature = "0_19_1")))]
pub const VERSION: &str = "0.19.0.1";

#[cfg(all(feature = "0_18_1", not(feature = "0_19_0_1")))]
pub const VERSION: &str = "0.18.1";

#[cfg(all(feature = "0_18_0", not(feature = "0_18_1")))]
pub const VERSION: &str = "0.18.0";

#[cfg(all(feature = "0_17_1", not(feature = "0_18_0")))]
pub const VERSION: &str = "0.17.1";

#[cfg(not(feature = "0_17_1"))]
pub const VERSION: &str = "N/A";
