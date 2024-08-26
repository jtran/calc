use ast::{Expr, Factor};
use recursion::Evaluator;

mod ast;
mod recursion;

fn main() {
    let expr = ast::Expr::BinaryOp {
        op: ast::TermBinaryOp::Add,
        lhs: Box::new(Expr::Factor(Box::new(Factor::Literal(1.0)))),
        rhs: Box::new(Expr::Factor(Box::new(Factor::Literal(2.0)))),
    };

    let mut evaluator = Evaluator::default();
    let result = evaluator.eval(&expr);
    println!("{:?}", result);
}
