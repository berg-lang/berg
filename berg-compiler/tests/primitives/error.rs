mod operators_comparison {
    use crate::*;

    #[test]
    fn equal_error_0() {
        expect("1/0==0").to_error(DivideByZero, 2)
    }

    #[test]
    fn equal_error_1() {
        expect("1/0==1").to_error(DivideByZero, 2)
    }

    #[test]
    fn equal_error_error() {
        expect("1/0==1/0").to_error(DivideByZero, 2)
    }

    #[test]
    fn equal_error_false() {
        expect("1/0==false").to_error(DivideByZero, 2)
    }

    #[test]
    fn equal_error_true() {
        expect("1/0==true").to_error(DivideByZero, 2)
    }

    #[test]
    fn not_equal_error_0() {
        expect("1/0!=0").to_error(DivideByZero, 2)
    }

    #[test]
    fn not_equal_error_1() {
        expect("1/0!=1").to_error(DivideByZero, 2)
    }

    #[test]
    fn not_equal_error_error() {
        expect("1/0!=1/0").to_error(DivideByZero, 2)
    }

    #[test]
    fn not_equal_error_false() {
        expect("1/0!=false").to_error(DivideByZero, 2)
    }

    #[test]
    fn not_equal_error_true() {
        expect("1/0!=true").to_error(DivideByZero, 2)
    }

    #[test]
    fn greater_or_equal_error_0() {
        expect("1/0>=0").to_error(DivideByZero, 2)
    }

    #[test]
    fn greater_or_equal_error_error() {
        expect("1/0>=1/0").to_error(DivideByZero, 2)
    }

    #[test]
    fn greater_or_equal_error_false() {
        expect("1/0>=false").to_error(DivideByZero, 2)
    }

    #[test]
    fn greater_or_equal_error_empty() {
        expect("1/0>=()").to_error(DivideByZero, 2)
    }

    #[test]
    fn greater_or_equal_error_true() {
        expect("1/0>=true").to_error(DivideByZero, 2)
    }

    #[test]
    fn greater_than_error_0() {
        expect("1/0>0").to_error(DivideByZero, 2)
    }

    #[test]
    fn greater_than_error_error() {
        expect("1/0>1/0").to_error(DivideByZero, 2)
    }

    #[test]
    fn greater_than_error_false() {
        expect("1/0>false").to_error(DivideByZero, 2)
    }

    #[test]
    fn greater_than_error_empty() {
        expect("1/0>()").to_error(DivideByZero, 2)
    }

    #[test]
    fn greater_than_error_true() {
        expect("1/0>true").to_error(DivideByZero, 2)
    }

    #[test]
    fn less_or_equal_error_0() {
        expect("1/0<=0").to_error(DivideByZero, 2)
    }

    #[test]
    fn less_or_equal_error_error() {
        expect("1/0<=1/0").to_error(DivideByZero, 2)
    }

    #[test]
    fn less_or_equal_error_false() {
        expect("1/0<=false").to_error(DivideByZero, 2)
    }

    #[test]
    fn less_or_equal_error_empty() {
        expect("()<=1/0").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn less_or_equal_error_true() {
        expect("1/0<=true").to_error(DivideByZero, 2)
    }

    #[test]
    fn less_than_error_0() {
        expect("1/0<0").to_error(DivideByZero, 2)
    }

    #[test]
    fn less_than_error_error() {
        expect("1/0<1/0").to_error(DivideByZero, 2)
    }

    #[test]
    fn less_than_error_false() {
        expect("1/0<false").to_error(DivideByZero, 2)
    }

    #[test]
    fn less_than_error_empty() {
        expect("1/0<()").to_error(DivideByZero, 2)
    }

    #[test]
    fn less_than_error_true() {
        expect("1/0<true").to_error(DivideByZero, 2)
    }
}

mod operators_logical {
    use crate::*;

    #[test]
    fn and_error_1() {
        expect("(1/0)&&1").to_error(DivideByZero, 3)
    }

    #[test]
    fn and_error_error() {
        expect("(1/0)&&(1/0)").to_error(DivideByZero, 3)
    }

    #[test]
    fn and_error_false() {
        expect("(1/0)&&false").to_error(DivideByZero, 3)
    }

    #[test]
    fn and_error_empty() {
        expect("(1/0)&&()").to_error(DivideByZero, 3)
    }

    #[test]
    fn and_error_true() {
        expect("(1/0)&&true").to_error(DivideByZero, 3)
    }

    #[test]
    fn or_error_1() {
        expect("(1/0)||1").to_error(DivideByZero, 3)
    }

    #[test]
    fn or_error_error() {
        expect("(1/0)||(1/0)").to_error(DivideByZero, 3)
    }

    #[test]
    fn or_error_false() {
        expect("(1/0)||false").to_error(DivideByZero, 3)
    }

    #[test]
    fn or_error_empty() {
        expect("(1/0)||()").to_error(DivideByZero, 3)
    }

    #[test]
    fn or_error_true() {
        expect("(1/0)||true").to_error(DivideByZero, 3)
    }

    #[test]
    fn not_error() {
        expect("!(1/0)").to_error(DivideByZero, 4)
    }
}

mod operators_math {
    use crate::*;

    #[test]
    fn add_error_1() {
        expect("1/0+1").to_error(DivideByZero, 2)
    }

    #[test]
    fn add_error_error() {
        expect("1/0+1/0").to_error(DivideByZero, 2)
    }

    #[test]
    fn add_error_false() {
        expect("1/0+false").to_error(DivideByZero, 2)
    }

    #[test]
    fn add_error_empty() {
        expect("1/0+()").to_error(DivideByZero, 2)
    }

    #[test]
    fn add_error_true() {
        expect("1/0+true").to_error(DivideByZero, 2)
    }

    #[test]
    fn sub_error_1() {
        expect("1/0-1").to_error(DivideByZero, 2)
    }

    #[test]
    fn sub_error_error() {
        expect("1/0-1/0").to_error(DivideByZero, 2)
    }

    #[test]
    fn sub_error_false() {
        expect("1/0-false").to_error(DivideByZero, 2)
    }

    #[test]
    fn sub_error_empty() {
        expect("1/0-()").to_error(DivideByZero, 2)
    }

    #[test]
    fn sub_error_true() {
        expect("1/0-true").to_error(DivideByZero, 2)
    }

    #[test]
    fn mul_error_1() {
        expect("(1/0)*1").to_error(DivideByZero, 3)
    }

    #[test]
    fn mul_error_error() {
        expect("(1/0)*(1/0)").to_error(DivideByZero, 3)
    }

    #[test]
    fn mul_error_false() {
        expect("(1/0)*false").to_error(DivideByZero, 3)
    }

    #[test]
    fn mul_error_empty() {
        expect("(1/0)*()").to_error(DivideByZero, 3)
    }

    #[test]
    fn mul_error_true() {
        expect("(1/0)*true").to_error(DivideByZero, 3)
    }

    #[test]
    fn div_error_1() {
        expect("(1/0)/1").to_error(DivideByZero, 3)
    }

    #[test]
    fn div_error_error() {
        expect("(1/0)/(1/0)").to_error(DivideByZero, 3)
    }

    #[test]
    fn div_error_false() {
        expect("(1/0)/false").to_error(DivideByZero, 3)
    }

    #[test]
    fn div_error_empty() {
        expect("(1/0)/()").to_error(DivideByZero, 3)
    }

    #[test]
    fn div_error_true() {
        expect("(1/0)/true").to_error(DivideByZero, 3)
    }

    #[test]
    fn neg_error() {
        expect("-(1/0)").to_error(DivideByZero, 4)
    }

    #[test]
    fn pos_error() {
        expect("+(1/0)").to_error(DivideByZero, 4)
    }
}