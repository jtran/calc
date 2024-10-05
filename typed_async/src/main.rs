use ast::{Expr, Factor, Param, Type};
use evaluator::Evaluator;
use tc::TypeChecker;

mod ast;
mod evaluator;
mod log;
mod runtime;
mod tc;
mod unparser;

#[tokio::main]
async fn main() {
    let stmts = [
        ast::Stmt::Let(
            "x".to_owned(),
            Type::Number,
            Box::new(ast::Expr::Factor(Box::new(ast::Factor::Literal(1.0)))),
        ),
        ast::Stmt::Let(
            "y".to_owned(),
            Type::Number,
            Box::new(ast::Expr::BinaryOp {
                op: ast::TermBinaryOp::Add,
                lhs: Box::new(Expr::Factor(Box::new(Factor::Variable("x".to_owned())))),
                rhs: Box::new(Expr::Factor(Box::new(Factor::Literal(2.0)))),
            }),
        ),
        ast::Stmt::Fun(
            "add".to_owned(),
            Box::new(ast::Function {
                params: vec![
                    Param {
                        name: "a".to_owned(),
                        ty: Type::Number,
                    },
                    Param {
                        name: "b".to_owned(),
                        ty: Type::Number,
                    },
                ],
                return_ty: Type::Number,
                // return_ty: Type::Arrow(vec![Type::Number], Box::new(Type::Number)),
                body: Expr::BinaryOp {
                    op: ast::TermBinaryOp::Add,
                    lhs: Box::new(Expr::Factor(Box::new(Factor::Variable("a".to_owned())))),
                    rhs: Box::new(Expr::Factor(Box::new(Factor::Variable("b".to_owned())))),
                },
            }),
        ),
        ast::Stmt::Let(
            "answer".to_owned(),
            Type::Number,
            Box::new(Expr::Factor(Box::new(Factor::Call(
                Box::new(Factor::Variable("add".to_owned())),
                vec![
                    Expr::Factor(Box::new(Factor::Variable("x".to_owned()))),
                    Expr::Factor(Box::new(Factor::Variable("y".to_owned()))),
                ],
            )))),
        ),
    ];

    let mut type_checker = TypeChecker::default();
    let result = type_checker.check_stmts(&stmts);
    println!("{:?}", result);
    println!("{:#?}", type_checker.bindings);

    // Refuse to evaluate if type checking failed.
    if result.is_err() {
        return;
    }

    let mut evaluator = Evaluator::default();
    evaluator.visitors.push(Box::new(log::Print::default()));
    println!();
    let result = evaluator.eval_stmts(&stmts).await;
    println!("{:?}", result);
    println!("{:#?}", evaluator.bindings);

    let mut unp = unparser::Unparser::default();
    unp.unparse_stmts(&stmts).unwrap();
    println!();
    println!("{}", unp.output());
}
