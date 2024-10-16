#[cfg(feature = "28_0")]
pub const VERSION: &str = "28.0";

#[cfg(feature = "26_0")]
pub const VERSION: &str = "26.0";

#[cfg(all(feature = "25_1", not(feature = "26_0")))]
pub const VERSION: &str = "25.1";

#[cfg(all(feature = "25_0", not(feature = "25_1")))]
pub const VERSION: &str = "25.0";

#[cfg(all(feature = "24_0_1", not(feature = "25_0")))]
pub const VERSION: &str = "24.0.1";

#[cfg(all(feature = "23_1", not(feature = "24_0_1")))]
pub const VERSION: &str = "23.1";

#[cfg(all(feature = "22_1", not(feature = "23_1")))]
pub const VERSION: &str = "22.1";

#[cfg(all(feature = "0_21_2", not(feature = "22_1")))]
pub const VERSION: &str = "0.21.2";

#[cfg(all(feature = "0_20_2", not(feature = "0_21_2")))]
pub const VERSION: &str = "0.20.2";

#[cfg(all(feature = "0_19_1", not(feature = "0_20_2")))]
pub const VERSION: &str = "0.19.1";

#[cfg(all(feature = "0_18_1", not(feature = "0_19_1")))]
pub const VERSION: &str = "0.18.1";

#[cfg(all(feature = "0_17_1", not(feature = "0_18_1")))]
pub const VERSION: &str = "0.17.1";
