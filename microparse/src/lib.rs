#![feature(portable_simd)]
#![feature(array_chunks)]
#![feature(bigint_helper_methods)]
#![feature(stdarch_x86_avx512)]
#![feature(avx512_target_feature)]

// pub mod parsers;
pub mod primitive_mask;
pub mod simd;
pub mod arch;