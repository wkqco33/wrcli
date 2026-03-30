use crate::error::{Result, WrCliError};

/// A function that validates positional arguments.
pub type ArgValidator = Box<dyn Fn(&[String]) -> Result<()> + Send + Sync>;

/// Requires exactly zero positional arguments.
pub fn no_args() -> ArgValidator {
    Box::new(|args| {
        if args.is_empty() {
            Ok(())
        } else {
            Err(WrCliError::ArgValidationFailed(format!(
                "expected no arguments, got {}",
                args.len()
            )))
        }
    })
}

/// Accepts any number of positional arguments.
pub fn arbitrary_args() -> ArgValidator {
    Box::new(|_| Ok(()))
}

/// Requires at least `n` positional arguments.
pub fn minimum_n_args(n: usize) -> ArgValidator {
    Box::new(move |args| {
        if args.len() >= n {
            Ok(())
        } else {
            Err(WrCliError::ArgValidationFailed(format!(
                "requires at least {} argument(s), got {}",
                n,
                args.len()
            )))
        }
    })
}

/// Allows at most `n` positional arguments.
pub fn maximum_n_args(n: usize) -> ArgValidator {
    Box::new(move |args| {
        if args.len() <= n {
            Ok(())
        } else {
            Err(WrCliError::ArgValidationFailed(format!(
                "accepts at most {} argument(s), got {}",
                n,
                args.len()
            )))
        }
    })
}

/// Requires exactly `n` positional arguments.
pub fn exact_args(n: usize) -> ArgValidator {
    Box::new(move |args| {
        if args.len() == n {
            Ok(())
        } else {
            Err(WrCliError::ArgValidationFailed(format!(
                "requires exactly {} argument(s), got {}",
                n,
                args.len()
            )))
        }
    })
}

/// Requires between `min` and `max` positional arguments (inclusive).
pub fn range_args(min: usize, max: usize) -> ArgValidator {
    Box::new(move |args| {
        if args.len() >= min && args.len() <= max {
            Ok(())
        } else {
            Err(WrCliError::ArgValidationFailed(format!(
                "requires between {} and {} argument(s), got {}",
                min,
                max,
                args.len()
            )))
        }
    })
}

/// Requires that all positional arguments are in the provided allowlist.
pub fn valid_args(allowed: Vec<String>) -> ArgValidator {
    Box::new(move |args| {
        for arg in args {
            if !allowed.contains(arg) {
                return Err(WrCliError::ArgValidationFailed(format!(
                    "invalid argument '{}', allowed: {}",
                    arg,
                    allowed.join(", ")
                )));
            }
        }
        Ok(())
    })
}
