use std::ops::Add;

use itertools::Either;

#[derive(Clone)]
struct Scalar;

struct Array(Vec<Scalar>);

impl Array {
    fn cardinality(&self) -> usize {
        self.0.len()
    }
}

struct Chunk(Vec<Array>);

impl Chunk {
    fn cardinality(&self) -> usize {
        self.0[0].cardinality()
    }
}

enum Value {
    Scalar(Scalar),
    Array(Array),
}

impl Value {
    fn into_iter(self, cardinality: usize) -> impl Iterator<Item = Scalar> {
        match self {
            Value::Scalar(s) => Either::Left(std::iter::repeat(s).take(cardinality)),
            Value::Array(a) => {
                assert_eq!(a.cardinality(), cardinality);
                Either::Right(a.0.into_iter())
            }
        }
    }
}

// Implementors must implement either `eval` or `eval_new` to avoid recursion.
trait Expr {
    fn eval(&self, input: &Chunk) -> Array {
        match self.eval_new(input) {
            Value::Scalar(s) => Array(vec![s; input.cardinality()]),
            Value::Array(a) => a,
        }
    }

    fn eval_new(&self, input: &Chunk) -> Value {
        Value::Array(self.eval(input))
    }
}

type BoxedExpr = Box<dyn Expr>;

struct BinaryExpr {
    lhs: BoxedExpr,
    rhs: BoxedExpr,
    func: Box<dyn Fn(Scalar, Scalar) -> Scalar>,
}

impl Expr for BinaryExpr {
    // Old implementation to be replaced.
    fn eval(&self, input: &Chunk) -> Array {
        let lhs = self.lhs.eval(input);
        let rhs = self.rhs.eval(input);
        Array(
            lhs.0
                .into_iter()
                .zip(rhs.0)
                .map(|(a, b)| (self.func)(a, b))
                .collect(),
        )
    }

    // New implementation.
    fn eval_new(&self, input: &Chunk) -> Value {
        let lhs = self.lhs.eval_new(input);
        let rhs = self.rhs.eval_new(input);
        let cardinality = input.cardinality();

        match (lhs, rhs) {
            (Value::Scalar(l), Value::Scalar(r)) => Value::Scalar((self.func)(l, r)),
            (lhs, rhs) => Value::Array(Array(
                lhs.into_iter(cardinality)
                    .zip(rhs.into_iter(cardinality))
                    .map(|(a, b)| (self.func)(a, b))
                    .collect(),
            )),
        }
    }
}

fn main() {}
