mod literals {
    use crate::*;
    #[test]
    fn true_literal() {
        expect("true").to_yield(true)
    }
    #[test]
    fn false_literal() {
        expect("false").to_yield(false)
    }
    #[test]
    fn uppercase_true() {
        expect("TRUE").to_error(NoSuchField, 0..=3)
    }
    #[test]
    fn uppercase_false() {
        expect("FALSE").to_error(NoSuchField, 0..=4)
    }
}

mod operators_comparison {
    use crate::*;
    #[test]
    fn equal_false_0() {
        expect("false==0").to_yield(false)
    }

    #[test]
    fn equal_false_1() {
        expect("false==1").to_yield(false)
    }

    #[test]
    fn equal_false_error() {
        expect("false==1/0").to_error(DivideByZero, 9)
    }

    #[test]
    fn equal_false_false() {
        expect("false==false").to_yield(true)
    }

    #[test]
    fn equal_false_empty() {
        expect("false==()").to_yield(false)
    }

    #[test]
    fn equal_false_true() {
        expect("false==true").to_yield(false)
    }

    #[test]
    fn equal_true_0() {
        expect("true==0").to_yield(false)
    }

    #[test]
    fn equal_true_1() {
        expect("true==1").to_yield(false)
    }

    #[test]
    fn equal_true_error() {
        expect("true==1/0").to_error(DivideByZero, 8)
    }

    #[test]
    fn equal_true_false() {
        expect("true==false").to_yield(false)
    }

    #[test]
    fn equal_true_empty() {
        expect("true==()").to_yield(false)
    }

    #[test]
    fn equal_true_true() {
        expect("true==true").to_yield(true)
    }

    #[test]
    fn not_equal_false_0() {
        expect("false!=0").to_yield(true)
    }

    #[test]
    fn not_equal_false_1() {
        expect("false!=1").to_yield(true)
    }

    #[test]
    fn not_equal_false_error() {
        expect("false!=1/0").to_error(DivideByZero, 9)
    }

    #[test]
    fn not_equal_false_false() {
        expect("false!=false").to_yield(false)
    }

    #[test]
    fn not_equal_false_empty() {
        expect("false!=()").to_yield(true)
    }

    #[test]
    fn not_equal_false_true() {
        expect("false!=true").to_yield(true)
    }

    #[test]
    fn not_equal_true_0() {
        expect("true!=0").to_yield(true)
    }

    #[test]
    fn not_equal_true_1() {
        expect("true!=1").to_yield(true)
    }

    #[test]
    fn not_equal_true_error() {
        expect("true!=1/0").to_error(DivideByZero, 8)
    }

    #[test]
    fn not_equal_true_false() {
        expect("true!=false").to_yield(true)
    }

    #[test]
    fn not_equal_true_empty() {
        expect("true!=()").to_yield(true)
    }

    #[test]
    fn not_equal_true_true() {
        expect("true!=true").to_yield(false)
    }

    #[test]
    fn greater_or_equal_false_0() {
        expect("false>=0").to_error(UnsupportedOperator, 5..=6)
    }

    #[test]
    fn greater_or_equal_false_1() {
        expect("false>=1").to_error(UnsupportedOperator, 5..=6)
    }

    #[test]
    fn greater_or_equal_false_error() {
        expect("false>=1/0").to_error(UnsupportedOperator, 5..=6)
    }

    #[test]
    fn greater_or_equal_false_false() {
        expect("false>=false").to_error(UnsupportedOperator, 5..=6)
    }

    #[test]
    fn greater_or_equal_false_empty() {
        expect("false>=()").to_error(UnsupportedOperator, 5..=6)
    }

    #[test]
    fn greater_or_equal_false_true() {
        expect("false>=true").to_error(UnsupportedOperator, 5..=6)
    }

    #[test]
    fn greater_or_equal_true_0() {
        expect("true>=0").to_error(UnsupportedOperator, 4..=5)
    }

    #[test]
    fn greater_or_equal_true_1() {
        expect("true>=1").to_error(UnsupportedOperator, 4..=5)
    }

    #[test]
    fn greater_or_equal_true_error() {
        expect("true>=1/0").to_error(UnsupportedOperator, 4..=5)
    }

    #[test]
    fn greater_or_equal_true_false() {
        expect("true>=false").to_error(UnsupportedOperator, 4..=5)
    }

    #[test]
    fn greater_or_equal_true_empty() {
        expect("true>=()").to_error(UnsupportedOperator, 4..=5)
    }

    #[test]
    fn greater_or_equal_true_true() {
        expect("true>=true").to_error(UnsupportedOperator, 4..=5)
    }

    #[test]
    fn greater_than_false_0() {
        expect("false>0").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn greater_than_false_1() {
        expect("false>1").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn greater_than_false_error() {
        expect("false>1/0").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn greater_than_false_false() {
        expect("false>false").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn greater_than_false_empty() {
        expect("false>()").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn greater_than_false_true() {
        expect("false>true").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn greater_than_true_0() {
        expect("true>0").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn greater_than_true_1() {
        expect("true>1").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn greater_than_true_error() {
        expect("true>1/0").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn greater_than_true_false() {
        expect("true>false").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn greater_than_true_empty() {
        expect("true>()").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn greater_than_true_true() {
        expect("true>true").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn less_or_equal_false_0() {
        expect("false<=0").to_error(UnsupportedOperator, 5..=6)
    }

    #[test]
    fn less_or_equal_false_1() {
        expect("false<=1").to_error(UnsupportedOperator, 5..=6)
    }

    #[test]
    fn less_or_equal_false_error() {
        expect("false<=1/0").to_error(UnsupportedOperator, 5..=6)
    }

    #[test]
    fn less_or_equal_false_false() {
        expect("false<=false").to_error(UnsupportedOperator, 5..=6)
    }

    #[test]
    fn less_or_equal_false_empty() {
        expect("false<=()").to_error(UnsupportedOperator, 5..=6)
    }

    #[test]
    fn less_or_equal_false_true() {
        expect("false<=true").to_error(UnsupportedOperator, 5..=6)
    }

    #[test]
    fn less_or_equal_true_0() {
        expect("true<=0").to_error(UnsupportedOperator, 4..=5)
    }

    #[test]
    fn less_or_equal_true_1() {
        expect("true<=1").to_error(UnsupportedOperator, 4..=5)
    }

    #[test]
    fn less_or_equal_true_error() {
        expect("true<=1/0").to_error(UnsupportedOperator, 4..=5)
    }

    #[test]
    fn less_or_equal_true_false() {
        expect("true<=false").to_error(UnsupportedOperator, 4..=5)
    }

    #[test]
    fn less_or_equal_true_empty() {
        expect("true<=()").to_error(UnsupportedOperator, 4..=5)
    }

    #[test]
    fn less_or_equal_true_true() {
        expect("true<=true").to_error(UnsupportedOperator, 4..=5)
    }

    #[test]
    fn less_than_false_0() {
        expect("false<0").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn less_than_false_1() {
        expect("false<1").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn less_than_false_error() {
        expect("false<1/0").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn less_than_false_false() {
        expect("false<false").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn less_than_false_empty() {
        expect("false<()").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn less_than_false_true() {
        expect("false<true").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn less_than_true_0() {
        expect("true<0").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn less_than_true_1() {
        expect("true<1").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn less_than_true_error() {
        expect("true<1/0").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn less_than_true_false() {
        expect("true<false").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn less_than_true_empty() {
        expect("true<()").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn less_than_true_true() {
        expect("true<true").to_error(UnsupportedOperator, 4)
    }
}

mod operators_logical {
    use crate::*;

    #[test]
    fn and_false_1() {
        expect("false&&1").to_yield(false)
    }

    #[test]
    fn and_false_error() {
        expect("false&&(1/0)").to_yield(false)
    }

    #[test]
    fn and_false_false() {
        expect("false&&false").to_yield(false)
    }

    #[test]
    fn and_false_multiple() {
        expect("false&&(1,2)").to_yield(false)
    }

    #[test]
    fn and_false_empty() {
        expect("false&&()").to_yield(false)
    }

    #[test]
    fn and_false_true() {
        expect("false&&true").to_yield(false)
    }

    #[test]
    fn and_true_1() {
        expect("true&&1").to_error(BadType, 6)
    }

    #[test]
    fn and_true_error() {
        expect("true&&(1/0)").to_error(DivideByZero, 9)
    }

    #[test]
    fn and_true_false() {
        expect("true&&false").to_yield(false)
    }

    #[test]
    fn and_true_multiple() {
        expect("true&&(1,2)").to_error(BadType, 6..=10)
    }

    #[test]
    fn and_true_empty() {
        expect("true&&()").to_error(BadType, 6..=7)
    }

    #[test]
    fn and_true_true() {
        expect("true&&true").to_yield(true)
    }

    #[test]
    fn or_false_1() {
        expect("false||1").to_error(BadType, 7)
    }

    #[test]
    fn or_false_error() {
        expect("false||(1/0)").to_error(DivideByZero, 10)
    }

    #[test]
    fn or_false_false() {
        expect("false||false").to_yield(false)
    }

    #[test]
    fn or_false_multiple() {
        expect("false||(1,2)").to_error(BadType, 7..=11)
    }

    #[test]
    fn or_false_empty() {
        expect("false||()").to_error(BadType, 7..=8)
    }

    #[test]
    fn or_false_true() {
        expect("false||true").to_yield(true)
    }

    #[test]
    fn or_true_1() {
        expect("true||1").to_yield(true)
    }

    #[test]
    fn or_true_error() {
        expect("true||(1/0)").to_yield(true)
    }

    #[test]
    fn or_true_false() {
        expect("true||false").to_yield(true)
    }

    #[test]
    fn or_true_multiple() {
        expect("true||(1,2)").to_yield(true)
    }

    #[test]
    fn or_true_empty() {
        expect("true||()").to_yield(true)
    }

    #[test]
    fn or_true_true() {
        expect("true||true").to_yield(true)
    }

    #[test]
    fn not_false() {
        expect("!false").to_yield(true)
    }

    #[test]
    fn not_true() {
        expect("!true").to_yield(false)
    }
}

mod operators_math {
    use crate::*;

    #[test]
    fn add_false_1() {
        expect("false+1").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn add_false_error() {
        expect("false+1/0").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn add_false_false() {
        expect("false+false").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn add_false_empty() {
        expect("false+()").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn add_false_true() {
        expect("false+true").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn add_true_1() {
        expect("true+1").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn add_true_error() {
        expect("true+1/0").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn add_true_false() {
        expect("true+false").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn add_true_empty() {
        expect("true+()").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn add_true_true() {
        expect("true+true").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn sub_false_1() {
        expect("false-1").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn sub_false_error() {
        expect("false-1/0").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn sub_false_false() {
        expect("false-false").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn sub_false_empty() {
        expect("false-()").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn sub_false_true() {
        expect("false-true").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn sub_true_1() {
        expect("true-1").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn sub_true_error() {
        expect("true-1/0").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn sub_true_false() {
        expect("true-false").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn sub_true_empty() {
        expect("true-()").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn sub_true_true() {
        expect("true-true").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn mul_false_1() {
        expect("false*1").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn mul_false_error() {
        expect("false*(1/0)").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn mul_false_false() {
        expect("false*false").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn mul_false_empty() {
        expect("false*()").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn mul_false_true() {
        expect("false*true").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn mul_true_1() {
        expect("true*1").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn mul_true_error() {
        expect("true*(1/0)").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn mul_true_false() {
        expect("true*false").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn mul_true_empty() {
        expect("true*()").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn mul_true_true() {
        expect("true*true").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn div_false_1() {
        expect("false/1").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn div_false_error() {
        expect("false/(1/0)").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn div_false_false() {
        expect("false/false").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn div_false_empty() {
        expect("false/()").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn div_false_true() {
        expect("false/true").to_error(UnsupportedOperator, 5)
    }

    #[test]
    fn div_true_1() {
        expect("true/1").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn div_true_error() {
        expect("true/(1/0)").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn div_true_false() {
        expect("true/false").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn div_true_empty() {
        expect("true/()").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn div_true_true() {
        expect("true/true").to_error(UnsupportedOperator, 4)
    }

    #[test]
    fn neg_false() {
        expect("-false").to_error(UnsupportedOperator, 0)
    }

    #[test]
    fn neg_true() {
        expect("-true").to_error(UnsupportedOperator, 0)
    }

    #[test]
    fn pos_false() {
        expect("+false").to_error(UnsupportedOperator, 0)
    }

    #[test]
    fn pos_true() {
        expect("+true").to_error(UnsupportedOperator, 0)
    }
}
