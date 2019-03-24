use crate::*;

#[test]
fn try_catch_error() {
    expect("
        :try_progress = 0
        :catch_ran = false
        :try_result = (
            try {
                try_progress = 1
                1/0
                try_progress = 2
                10
            } catch {
                catch_ran = true
                20
            }
        )
        try_result,try_progress,catch_ran
    ").to_yield(tuple!(20,1,true));
}

#[test]
fn try_catch_ok() {
    expect("
        :try_progress = 0
        :catch_ran = false
        :try_result = (
            try {
                try_progress = 1
                10
            } catch {
                catch_ran = true
                20
            }
        )
        try_result,try_progress,catch_ran
    ").to_yield(tuple!(10,1,false));
}

#[test]
fn try_catch_finally_ok() {
    expect("
        :try_progress = 0
        :catch_ran = false
        :finally_ran = false
        :try_result = (
            try {
                try_progress = 1
                10
            } catch {
                catch_ran = true
                20
            } finally {
                finally_ran = true
                30
            }
        )
        try_result,try_progress,catch_ran,finally_ran
    ").to_yield(tuple!(10,1,false,true));
}

#[test]
fn try_catch_finally_error() {
    expect("
        :try_progress = 0
        :catch_ran = false
        :finally_ran = false
        :try_result = (
            try {
                try_progress = 1
                1/0
                try_progress = 2
                10
            } catch {
                catch_ran = true
                20
            } finally {
                finally_ran = true
                30
            }
        )
        try_result,try_progress,catch_ran,finally_ran
    ").to_yield(tuple!(20,1,true,true));
}

#[test]
fn dangling_try() {
    expect("try").to_error(TryWithoutBlock, "try")
}

#[test]
fn dangling_catch() {
    expect("catch").to_error(CatchWithoutResult, "catch")
}

#[test]
fn dangling_finally() {
    expect("finally").to_error(FinallyWithoutResult, "finally")
}
#[test]
fn dangling_try_try() {
    expect("try try").to_error(TryWithoutBlock, "try".after(" "))
}
#[test]
fn dangling_try_1() {
    expect("try 1").to_error(TryBlockMustBeBlock, "1")
}
#[test]
fn dangling_try_catch() {
    expect("try catch").to_error(CatchWithoutResult, "catch")
}
#[test]
fn dangling_try_finally() {
    expect("try finally").to_error(FinallyWithoutResult, "finally")
}
#[test]
fn throw() {
    expect("throw 1").to_error(1, "throw 1")
}
#[test]
fn dangling_throw() {
    expect("throw").to_error(ThrowWithoutException, "throw")
}
