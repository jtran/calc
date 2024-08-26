use ast::{Expr, Factor};
use evaluator::Evaluator;

mod ast;
mod evaluator;
mod recursion;

fn main() {
    let stmts = [
        ast::Stmt::Let(
            "x".to_string(),
            Box::new(ast::Expr::Factor(Box::new(ast::Factor::Literal(1.0)))),
        ),
        ast::Stmt::Let(
            "y".to_string(),
            Box::new(ast::Expr::BinaryOp {
                op: ast::TermBinaryOp::Add,
                lhs: Box::new(Expr::Factor(Box::new(Factor::Variable("x".to_owned())))),
                rhs: Box::new(Expr::Factor(Box::new(Factor::Literal(2.0)))),
            }),
        ),
    ];

    let mut evaluator = Evaluator::default();
    let result = evaluator.eval_stmts(&stmts);
    println!("{:?}", result);
    println!("{:#?}", evaluator.bindings);
}
