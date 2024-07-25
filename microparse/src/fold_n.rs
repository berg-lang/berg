#[inline(always)]
pub fn fold_n<const N: usize, T>(init: T, f: impl FnMut(T, usize) -> T) -> T where FoldN<N>: Supported {
    FoldN::<N>::fold_n(init, f)
}

pub struct FoldN<const N: usize>;
pub trait Supported {
    fn fold_n<T>(input: T, f: impl FnMut(T, usize) -> T) -> T;
}

impl Supported for FoldN<1> {
    #[inline(always)]
    fn fold_n<T>(mut val: T, mut f: impl FnMut(T, usize) -> T) -> T {
        val = f(val, 0);
        val
    }
}

impl Supported for FoldN<2> {
    #[inline(always)]
    fn fold_n<T>(mut val: T, mut f: impl FnMut(T, usize) -> T) -> T {
        val = f(val, 0);
        val = f(val, 1);
        val
    }
}

impl Supported for FoldN<4> {
    #[inline(always)]
    fn fold_n<T>(mut val: T, mut f: impl FnMut(T, usize) -> T) -> T {
        val = f(val, 0);
        val = f(val, 1);
        val = f(val, 2);
        val = f(val, 3);
        val
    }
}

impl Supported for FoldN<8> {
    #[inline(always)]
    fn fold_n<T>(mut val: T, mut f: impl FnMut(T, usize) -> T) -> T {
        val = f(val, 0);
        val = f(val, 1);
        val = f(val, 2);
        val = f(val, 3);
        val = f(val, 4);
        val = f(val, 5);
        val = f(val, 6);
        val = f(val, 7);
        val
    }
}

impl Supported for FoldN<16> {
    #[inline(always)]
    fn fold_n<T>(mut val: T, mut f: impl FnMut(T, usize) -> T) -> T {
        val = fold_n::<8, T>(val, &mut f);
        val = fold_n::<8, T>(val, #[inline(always)] |v, i| f(v, i + 8));
        val
    }
}
impl Supported for FoldN<32> {
    #[inline(always)]
    fn fold_n<T>(mut val: T, mut f: impl FnMut(T, usize) -> T) -> T {
        val = fold_n::<16, T>(val, &mut f);
        val = fold_n::<16, T>(val, #[inline(always)] |v, i| f(v, i + 16));
        val
    }
}
impl Supported for FoldN<64> {
    #[inline(always)]
    fn fold_n<T>(mut val: T, mut f: impl FnMut(T, usize) -> T) -> T {
        val = fold_n::<32, T>(val, &mut f);
        val = fold_n::<32, T>(val, #[inline(always)] |v, i| f(v, i + 32));
        val
    }
}
