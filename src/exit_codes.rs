//! Exit codes
//! 
 
use std::convert::TryFrom;

/// Success exit codes
/// 
#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(i8)]
pub enum OkExitCode{
    NO_CHANGE = 0,
    SOME_COPIES = 1,
    EXTRA_FOUND = 2,
    SOME_COPIES_EXTRA_FOUND = 3,
    MISMATCHES = 4,
    SOME_COPIES_MISMATCHES = 5,
    MISMATCHES_EXTRA_FOUND = 6,
    SOME_COPIES_MISMATCHES_EXTRA_FOUND = 7,
}

/// Exit codes that include a failure.
/// 
#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(i8)]
pub enum ErrExitCode{
    FAIL = 8,
    SOME_COPIES_FAIL = 9,
    FAIL_EXTRA_FOUND = 10,
    SOME_COPIES_FAIL_EXTRA_FOUND = 11,
    FAIL_MISMATCHES = 12,
    SOME_COPIES_FAIL_MISMATCHES = 13,
    FAIL_MISMATCHES_EXTRA_FOUND = 14,
    SOME_COPIES_FAIL_MISMATCHES_EXTRA_FOUND = 15,
    NO_CHANGE_FATAL_ERROR = 16,
}

impl TryFrom<i8> for OkExitCode {
    type Error = Result<ErrExitCode, (&'static str, i8)>;

    fn try_from(n: i8) -> Result<Self, Self::Error> {
        if n < 8 {
            Ok(
                match n {
                    0 => OkExitCode::NO_CHANGE,
                    1 => OkExitCode::SOME_COPIES,
                    2 => OkExitCode::EXTRA_FOUND,
                    3 => OkExitCode::SOME_COPIES_EXTRA_FOUND,
                    4 => OkExitCode::MISMATCHES,
                    5 => OkExitCode::SOME_COPIES_MISMATCHES,
                    6 => OkExitCode::MISMATCHES_EXTRA_FOUND,
                    7 => OkExitCode::SOME_COPIES_MISMATCHES_EXTRA_FOUND,
                    _ => unreachable!(),
                }
            )
        } else {
            Err(
                match n {
                    8 => Ok(ErrExitCode::FAIL),
                    9 => Ok(ErrExitCode::SOME_COPIES_FAIL),
                    10 => Ok(ErrExitCode::FAIL_EXTRA_FOUND),
                    11 => Ok(ErrExitCode::SOME_COPIES_FAIL_EXTRA_FOUND),
                    12 => Ok(ErrExitCode::FAIL_MISMATCHES),
                    13 => Ok(ErrExitCode::SOME_COPIES_FAIL_MISMATCHES),
                    14 => Ok(ErrExitCode::FAIL_MISMATCHES_EXTRA_FOUND),
                    15 => Ok(ErrExitCode::SOME_COPIES_FAIL_MISMATCHES_EXTRA_FOUND),
                    16 => Ok(ErrExitCode::NO_CHANGE_FATAL_ERROR),
                    c => Err(("Invalid exit code", c)),
                }
            )
        }
    }
}