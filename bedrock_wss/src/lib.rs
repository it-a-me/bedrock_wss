#![warn(
    clippy::complexity,
    clippy::correctness,
    clippy::nursery,
    clippy::perf,
    clippy::pedantic,
    clippy::style,
    clippy::suspicious
)]
#![allow(clippy::missing_errors_doc)]

pub const DEFAULT_WSS_PORT: &str = "127.0.0.1:9988";
pub const DEFAULT_CLIENT_PORT: &str = "127.0.0.1:9989";
pub mod request;

pub mod re_exports {
    pub use strum::{AsRefStr, EnumString, IntoEnumIterator};
    pub use uuid::Uuid;
}
