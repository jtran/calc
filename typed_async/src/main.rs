use ast::{Expr, Factor};
use evaluator::Evaluator;

mod ast;
mod evaluator;
mod log;
mod runtime;
mod unparser;

#[tokio::main]
async fn main() {
    let stmts = [
        ast::Stmt::Let(
            "x".to_owned(),
            Box::new(ast::Expr::Factor(Box::new(ast::Factor::Literal(1.0)))),
        ),
        ast::Stmt::Let(
            "y".to_owned(),
            Box::new(ast::Expr::BinaryOp {
                op: ast::TermBinaryOp::Add,
                lhs: Box::new(Expr::Factor(Box::new(Factor::Variable("x".to_owned())))),
                rhs: Box::new(Expr::Factor(Box::new(Factor::Literal(2.0)))),
            }),
        ),
        ast::Stmt::Fun(
            "add".to_owned(),
            Box::new(ast::Function {
                params: vec!["a".to_string(), "b".to_string()],
                body: Expr::BinaryOp {
                    op: ast::TermBinaryOp::Add,
                    lhs: Box::new(Expr::Factor(Box::new(Factor::Variable("a".to_owned())))),
                    rhs: Box::new(Expr::Factor(Box::new(Factor::Variable("b".to_owned())))),
                },
            }),
        ),
        ast::Stmt::Let(
            "answer".to_owned(),
            Box::new(Expr::Factor(Box::new(Factor::Call(
                Box::new(Factor::Variable("add".to_owned())),
                vec![
                    Expr::Factor(Box::new(Factor::Variable("x".to_owned()))),
                    Expr::Factor(Box::new(Factor::Variable("y".to_owned()))),
                ],
            )))),
        ),
    ];

    let mut evaluator = Evaluator::default();
    evaluator.visitors.push(Box::new(log::Print::default()));
    let result = evaluator.eval_stmts(&stmts).await;
    println!("{:?}", result);
    println!("{:#?}", evaluator.bindings);

    let mut unp = unparser::Unparser::default();
    unp.unparse_stmts(&stmts).unwrap();
    println!();
    println!("{}", unp.output());
}
