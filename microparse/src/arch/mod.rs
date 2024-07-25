pub mod fallback;
pub(crate) mod define_simd;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")] {
        pub mod arm64;
        pub use arm64::native as native;
    } else if #[cfg(target_arch = "x86_64")] {
        pub mod x86_64;
        pub use x86_64::native as native;
    } else {
        pub use fallback as native;
    }
}

pub trait Implementation {
    
}

pub trait SimdImplementation {

}
