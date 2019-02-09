mod literals {
    use crate::*;

    #[test]
    fn comma_sequence() {
        expect("1,2").to_yield(tuple!(1, 2))
    }
    #[test]
    fn comma_sequence_three() {
        expect("1,2,3").to_yield(tuple!(1, 2, 3))
    }
    #[test]
    fn comma_left_space() {
        expect("1 ,2").to_yield(tuple!(1, 2))
    }
    #[test]
    fn comma_right_space() {
        expect("1, 2").to_yield(tuple!(1, 2))
    }
    #[test]
    fn comma_both_space() {
        expect("1 , 2").to_yield(tuple!(1, 2))
    }

    #[test]
    fn comma_sequence_bare_parentheses() {
        expect("(1,2)").to_yield(tuple!(1, 2));
    }

    #[test]
    fn only_comma() {
        expect(",").to_error(MissingOperand, 0)
    }
    #[test]
    fn right_comma() {
        expect("1,").to_yield(tuple!(1))
    }
    #[test]
    fn right_comma_nested() {
        expect("(1),").to_yield(tuple!(1))
    }
    #[test]
    fn right_comma_inside_parentheses() {
        expect("(1,)").to_yield(tuple!(1))
    }
    #[test]
    fn right_comma_inside_and_outside_parentheses() {
        expect("(1,),").to_yield(tuple!([1]))
    }
    #[test]
    fn right_comma_nested_multiple() {
        expect("(1,2),").to_yield(tuple!([1, 2]))
    }
    #[test]
    fn left_comma() {
        expect(",1").to_error(MissingOperand, 0)
    }
    #[test]
    fn both_comma() {
        expect(",1,").to_error(MissingOperand, 0)
    }

    #[test]
    fn left_double_comma() {
        expect(",,1").to_error(MissingOperand, 0)
    }
    #[test]
    fn right_double_comma() {
        expect("1,,").to_error(MissingOperand, 1)
    }
    #[test]
    fn both_double_comma() {
        expect(",,1,,").to_error(MissingOperand, 0)
    }
    #[test]
    fn between_double_comma() {
        expect("1,,2").to_error(MissingOperand, 1)
    }

    #[test]
    fn paren_comma_all_over() {
        expect(",(,(,),),").to_error(MissingOperand, 0)
    }

    #[test]
    fn nested_comma() {
        expect("1,(2,3)").to_yield(tuple!(1, [2, 3]));
    }

    #[test]
    fn nested_comma_first() {
        expect("(1,2),3").to_yield(tuple!([1, 2], 3));
    }

    #[test]
    fn nested_comma_single() {
        expect("(1,2),").to_yield(tuple!([1, 2]));
    }

    #[test]
    #[should_panic]
    fn comma_trailing_comma_is_not_same_as_nested_tuple() {
        expect("(1,2),").to_yield(tuple!(1, 2));
    }

    #[test]
    fn comma_trailing_comma_is_same_as_single_value() {
        expect("1,").to_yield(1);
    }

    #[test]
    #[should_panic]
    fn comma_multiple_values_is_not_same_as_nested_tuple() {
        expect("1,2").to_yield(tuple!([1, 2]));
    }

    #[test]
    #[should_panic]
    fn comma_bare_parentheses_is_not_same_as_nested_tuple() {
        expect("(1,2)").to_yield(tuple!([1, 2]));
    }
}

mod operators_comparison {
    use crate::*;

    #[test]
    fn equal_empty_0() {
        expect("()==0").to_yield(false)
    }

    #[test]
    fn equal_empty_1() {
        expect("()==1").to_yield(false)
    }

    #[test]
    fn equal_empty_false() {
        expect("()==false").to_yield(false)
    }

    #[test]
    fn equal_empty_empty() {
        expect("()==()").to_yield(true)
    }

    #[test]
    fn equal_empty_true() {
        expect("()==true").to_yield(false)
    }

    #[test]
    fn not_equal_empty_0() {
        expect("()!=0").to_yield(true)
    }

    #[test]
    fn not_equal_empty_1() {
        expect("()!=1").to_yield(true)
    }

    #[test]
    fn not_equal_empty_false() {
        expect("()!=false").to_yield(true)
    }

    #[test]
    fn not_equal_empty_empty() {
        expect("()!=()").to_yield(false)
    }

    #[test]
    fn not_equal_empty_true() {
        expect("()!=true").to_yield(true)
    }

    #[test]
    fn greater_or_equal_empty_0() {
        expect("()>=0").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn greater_or_equal_empty_error() {
        expect("()>=1/0").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn greater_or_equal_empty_false() {
        expect("()>=false").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn greater_or_equal_empty_empty() {
        expect("()>=()").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn greater_or_equal_empty_true() {
        expect("()>=true").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn greater_than_empty_0() {
        expect("()>0").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn greater_than_empty_error() {
        expect("()>1/0").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn greater_than_empty_false() {
        expect("()>false").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn greater_than_empty_empty() {
        expect("()>()").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn greater_than_empty_true() {
        expect("()>true").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn less_or_equal_empty_0() {
        expect("()<=0").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn less_or_equal_empty_error() {
        expect("1/0<=()").to_error(DivideByZero, 2)
    }

    #[test]
    fn less_or_equal_empty_false() {
        expect("()<=false").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn less_or_equal_empty_empty() {
        expect("()<=()").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn less_or_equal_empty_true() {
        expect("()<=true").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn less_than_empty_0() {
        expect("()<0").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn less_than_empty_error() {
        expect("()<1/0").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn less_than_empty_false() {
        expect("()<false").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn less_than_empty_empty() {
        expect("()<()").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn less_than_empty_true() {
        expect("()<true").to_error(UnsupportedOperator, 2)
    }
}

mod operators_logical {
    use crate::*;

    #[test]
    fn and_empty_1() {
        expect("()&&1").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn and_empty_error() {
        expect("()&&(1/0)").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn and_empty_false() {
        expect("()&&false").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn and_empty_empty() {
        expect("()&&()").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn and_empty_true() {
        expect("()&&true").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn or_empty_1() {
        expect("()||1").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn or_empty_error() {
        expect("()||(1/0)").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn or_empty_false() {
        expect("()||false").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn or_empty_empty() {
        expect("()||()").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn or_empty_true() {
        expect("()||true").to_error(UnsupportedOperator, 2..=3)
    }

    #[test]
    fn not_empty() {
        expect("!()").to_error(UnsupportedOperator, 0)
    }
}

mod operators_math {
    use crate::*;

    #[test]
    fn add_empty_1() {
        expect("()+1").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn add_empty_error() {
        expect("()+1/0").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn add_empty_false() {
        expect("()+false").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn add_empty_empty() {
        expect("()+()").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn add_empty_true() {
        expect("()+true").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn sub_empty_1() {
        expect("()-1").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn sub_empty_error() {
        expect("()-1/0").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn sub_empty_false() {
        expect("()-false").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn sub_empty_empty() {
        expect("()-()").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn sub_empty_true() {
        expect("()-true").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn mul_empty_1() {
        expect("()*1").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn mul_empty_error() {
        expect("()*(1/0)").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn mul_empty_false() {
        expect("()*false").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn mul_empty_empty() {
        expect("()*()").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn mul_empty_true() {
        expect("()*true").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn div_empty_1() {
        expect("()/1").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn div_empty_error() {
        expect("()/(1/0)").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn div_empty_false() {
        expect("()/false").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn div_empty_empty() {
        expect("()/()").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn div_empty_true() {
        expect("()/true").to_error(UnsupportedOperator, 2)
    }

    #[test]
    fn neg_empty() {
        expect("-()").to_error(UnsupportedOperator, 0)
    }

    #[test]
    fn pos_empty() {
        expect("+()").to_error(UnsupportedOperator, 0)
    }
}
