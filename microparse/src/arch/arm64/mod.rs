pub mod neon;

cfg_if::cfg_if! {
    if #[cfg(all(target_feature = "neon"))] {
        pub use neon as native;
    } else {
        pub use super::fallback as native;
    }
}