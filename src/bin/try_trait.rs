#![feature(try_trait_v2)]

use std::{
    convert::Infallible,
    ops::{ControlFlow, FromResidual, Try},
};

struct Error;

struct Project;
impl Project {
    fn is_interesting(&self) -> bool {
        todo!()
    }

    fn bad_1(&self) -> Result<(), Error> {
        todo!()
    }

    fn is_bad_2(&self) -> bool {
        todo!()
    }
}

struct Plan;
impl Plan {
    fn as_project(&self) -> Option<&Project> {
        todo!()
    }
}

/// Result when applying an optimization rule.
enum OResult<T> {
    /// Successfully optimized the input.
    Ok(T),
    /// The current rule is not applicable to the input.
    /// The caller may try other rules.
    NotApplicable,
    /// There was an unrecoverable error while applying the rule.
    /// The caller should stop trying other rules and report the error.
    Err(Error),
}

/// `?` on an `Option<_>`.
impl<T> FromResidual<Option<Infallible>> for OResult<T> {
    fn from_residual(residual: Option<Infallible>) -> Self {
        match residual {
            Some(_) => unreachable!(),
            None => Self::NotApplicable,
        }
    }
}

/// `?` on a `Result<_, Error>`.
impl<T> FromResidual<Result<Infallible, Error>> for OResult<T> {
    fn from_residual(residual: Result<Infallible, Error>) -> Self {
        match residual {
            Ok(_) => unreachable!(),
            Err(e) => Self::Err(e),
        }
    }
}

impl<T> FromResidual<OResult<Infallible>> for OResult<T> {
    fn from_residual(residual: OResult<Infallible>) -> Self {
        match residual {
            OResult::Ok(_) => unreachable!(),
            OResult::NotApplicable => Self::NotApplicable,
            OResult::Err(e) => Self::Err(e),
        }
    }
}

impl<T> Try for OResult<T> {
    type Output = T;

    type Residual = OResult<Infallible>;

    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            OResult::Ok(v) => ControlFlow::Continue(v),
            OResult::NotApplicable => ControlFlow::Break(OResult::NotApplicable),
            OResult::Err(error) => ControlFlow::Break(OResult::Err(error)),
        }
    }
}

fn optimize(plan: Plan) -> OResult<Plan> {
    use OResult::*;

    // If the plan is not a project, return `NotApplicable`.
    // This can be done because we impl `FromResidual<Option<Infallible>>`.
    let project = plan.as_project()?;

    // We can also manually return `NotApplicable`.
    if !project.is_interesting() {
        return NotApplicable;
    }

    // If there's a non-recoverable error, return `Err`.
    // This can be done because we impl `FromResidual<Result<Infallible, Error>>`.
    project.bad_1()?;

    // We can also manually return `Err`.
    if project.is_bad_2() {
        return Err(Error);
    }

    /* do transformations */

    Ok(plan)
}

fn main() {}
