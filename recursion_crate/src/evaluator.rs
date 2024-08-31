use recursion::CollapsibleExt;

use crate::{
    ast::{Expr, Factor, FactorBinaryOp, Stmt, TermBinaryOp},
    recursion::{ExprFrame, FactorFrame},
    runtime::{Env, Value},
};

type Error = Box<dyn std::error::Error>;

#[derive(Debug, Default)]
pub(crate) struct Evaluator {
    pub bindings: Env,
}

impl Evaluator {
    pub(crate) fn eval_stmts(&mut self, stmts: &[Stmt]) -> Result<Value, Error> {
        let mut last = Value::Number(0.0);
        for stmt in stmts {
            last = self.eval_stmt(stmt)?;
        }
        Ok(last)
    }

    pub(crate) fn eval_stmt(&mut self, stmt: &Stmt) -> Result<Value, Error> {
        match stmt {
            Stmt::Let(ident, expr) => {
                let value = self.eval_expr(expr)?;
                self.bindings.insert(ident.clone(), value.clone());
                Ok(value)
            }
            Stmt::Fun(ident, function) => {
                let closure =
                    Value::Closure(Box::new(function.as_ref().clone()), self.bindings.clone());
                self.bindings.insert(ident.to_owned(), closure.clone());
                Ok(closure)
            }
        }
    }

    pub(crate) fn eval_expr(&mut self, expr: &Expr) -> Result<Value, Error> {
        expr.try_collapse_frames(|frame| match frame {
            ExprFrame::Factor(f) => self.eval_factor(f.as_ref()),
            ExprFrame::BinaryOp { op, lhs, rhs } => match op {
                TermBinaryOp::Add => match (lhs, rhs) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                    _ => Err("Addition only supported for numbers".into()),
                },
                TermBinaryOp::Sub => match (lhs, rhs) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                    _ => Err("Subtraction only supported for numbers".into()),
                },
            },
        })
    }

    fn eval_factor(&mut self, factor: &Factor) -> Result<Value, Error> {
        factor.try_collapse_frames(|frame| match frame {
            FactorFrame::Literal(a) => Ok(Value::Number(a)),
            FactorFrame::Variable(ident) => self
                .bindings
                .get(ident.as_str())
                .cloned()
                .ok_or_else(|| format!("Variable not found in bindings: {}", ident).into()),
            FactorFrame::Group(e) => self.eval_expr(e.as_ref()),
            FactorFrame::BinaryOp { op, lhs, rhs } => match op {
                FactorBinaryOp::Mul => match (lhs, rhs) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                    _ => Err("Multiplication only supported for numbers".into()),
                },
                FactorBinaryOp::Div => match (lhs, rhs) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
                    _ => Err("Division only supported for numbers".into()),
                },
            },
            FactorFrame::Call(fun, args) => {
                let Value::Closure(fun, env) = fun else {
                    return Err("Expected closure".into());
                };
                let mut body_env = env.clone();
                for (param, arg) in fun.params.iter().zip(args.iter()) {
                    body_env.insert(param.clone(), self.eval_expr(arg)?);
                }
                let current_bindings = std::mem::replace(&mut self.bindings, body_env);
                let result = self.eval_expr(&fun.body);
                self.bindings = current_bindings;
                result
            }
        })
    }
}
