pub mod avx2;
pub mod sse4_2;
pub mod avx512_icelake;

cfg_if::cfg_if! {
    if #[cfg(all(target_feature="avx512f", target_feature="avx512dq", target_feature="avx512vl", target_feature="avx512bw", target_feature="avx512cd", target_feature="avx512vbmi", target_feature="avx512vbmi2", target_feature="avx2", target_feature="sse4.2", target_feature="pclmulqdq"))] {
        pub use avx512_icelake as native;
    } else if #[cfg(all(target_feature="avx2", target_feature="sse4.2", target_feature="pclmulqdq"))] {
        pub use native = avx2 as native;
    } else if #[cfg(all(target_feature="sse4.2", target_feature="pclmulqdq"))] {
        pub use sse4_2 as native;
    } else {
        pub use super::fallback as native;
    }
}
