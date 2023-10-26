use visitor1::Visitor;
use visitor2::Accept;

struct InputRef;

enum Expression {
    InputRef(InputRef),
    FunctionCall(FunctionCall),
}

struct FunctionCallType;

struct FunctionCall {
    inputs: Vec<Expression>,
    ty: FunctionCallType,
}

mod visitor1 {
    use super::*;

    pub(super) trait Visitor {
        fn visit_expression(&mut self, expression: &Expression) {
            match expression {
                Expression::InputRef(input_ref) => self.visit_input_ref(input_ref),
                Expression::FunctionCall(function_call) => self.visit_function_call(function_call),
            }
        }

        fn visit_function_call(&mut self, function_call: &FunctionCall) {
            for input in &function_call.inputs {
                self.visit_expression(input);
            }
        }

        fn visit_input_ref(&mut self, _input_ref: &InputRef) {}
    }
}

mod visitor2 {
    use super::*;

    pub(super) trait Visit {
        fn visit_expression(&mut self, _expression: &Expression) {}

        fn visit_function_call(&mut self, _function_call: &FunctionCall) {}

        fn visit_input_ref(&mut self, _input_ref: &InputRef) {}
    }

    pub(super) trait Accept {
        fn accept(&self, visitor: &mut dyn Visit);
    }

    impl Accept for Expression {
        fn accept(&self, visitor: &mut dyn Visit) {
            match self {
                Expression::InputRef(input_ref) => input_ref.accept(visitor),
                Expression::FunctionCall(function_call) => function_call.accept(visitor),
            }

            visitor.visit_expression(self);
        }
    }

    impl Accept for FunctionCall {
        fn accept(&self, visitor: &mut dyn Visit) {
            for input in &self.inputs {
                input.accept(visitor);
            }

            visitor.visit_function_call(self);
        }
    }

    impl Accept for InputRef {
        fn accept(&self, visitor: &mut dyn Visit) {
            visitor.visit_input_ref(self);
        }
    }
}

#[derive(Default)]
struct CountFunctionCall {
    count: usize,
}

impl visitor1::Visitor for CountFunctionCall {
    fn visit_function_call(&mut self, _function_call: &FunctionCall) {
        self.count += 1;

        // Repeat the default implementation.
        for input in &_function_call.inputs {
            self.visit_expression(input);
        }
    }
}

impl visitor2::Visit for CountFunctionCall {
    fn visit_function_call(&mut self, _function_call: &FunctionCall) {
        self.count += 1;
    }
}

fn visit1(expr: &Expression) -> usize {
    let mut visitor = CountFunctionCall::default();
    visitor.visit_expression(expr);
    visitor.count
}

fn visit2(expr: &Expression) -> usize {
    let mut visitor = CountFunctionCall::default();
    expr.accept(&mut visitor);
    visitor.count
}

fn visit2_customize_accept(expr: &Expression) -> usize {
    use visitor2::Visit;

    trait MyAccept {
        fn my_accept(&self, visitor: &mut dyn Visit);
    }

    impl MyAccept for Expression {
        fn my_accept(&self, visitor: &mut dyn Visit) {
            match self {
                Expression::InputRef(input_ref) => input_ref.my_accept(visitor),
                Expression::FunctionCall(function_call) => function_call.my_accept(visitor),
            }

            visitor.visit_expression(self);
        }
    }

    impl MyAccept for FunctionCall {
        fn my_accept(&self, visitor: &mut dyn Visit) {
            for input in &self.inputs {
                input.my_accept(visitor);
            }

            visitor.visit_function_call(self);
        }
    }

    impl MyAccept for InputRef {
        fn my_accept(&self, visitor: &mut dyn Visit) {
            visitor.visit_input_ref(self);
        }
    }

    let mut visitor = CountFunctionCall::default();
    expr.my_accept(&mut visitor);
    visitor.count
}

fn main() {
    let expression = Expression::FunctionCall(FunctionCall {
        inputs: vec![Expression::FunctionCall(FunctionCall {
            inputs: vec![Expression::InputRef(InputRef)],
            ty: FunctionCallType,
        })],
        ty: FunctionCallType,
    });

    assert_eq!(visit1(&expression), 2);
    assert_eq!(visit2(&expression), 2);
    assert_eq!(visit2_customize_accept(&expression), 2);
}
