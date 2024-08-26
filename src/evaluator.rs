use std::collections::HashMap;

use recursion::CollapsibleExt;

use crate::{
    ast::{Expr, Factor, FactorBinaryOp, Identifier, Stmt, TermBinaryOp},
    recursion::{ExprFrame, FactorFrame},
};

type Error = Box<dyn std::error::Error>;

#[derive(Debug, Default)]
pub(crate) struct Evaluator {
    pub bindings: HashMap<Identifier, f64>,
}

impl Evaluator {
    pub(crate) fn eval_stmts(&mut self, stmts: &[Stmt]) -> Result<f64, Error> {
        let mut last = 0.0;
        for stmt in stmts {
            last = self.eval_stmt(stmt)?;
        }
        Ok(last)
    }

    pub(crate) fn eval_stmt(&mut self, stmt: &Stmt) -> Result<f64, Error> {
        match stmt {
            Stmt::Let(ident, expr) => {
                let value = self.eval_expr(expr)?;
                self.bindings.insert(ident.clone(), value);
                Ok(value)
            }
        }
    }

    pub(crate) fn eval_expr(&mut self, expr: &Expr) -> Result<f64, Error> {
        expr.try_collapse_frames(|frame| match frame {
            ExprFrame::Factor(f) => self.eval_factor(f.as_ref()),
            ExprFrame::BinaryOp { op, lhs, rhs } => match op {
                TermBinaryOp::Add => Ok(lhs + rhs),
                TermBinaryOp::Sub => Ok(lhs - rhs),
            },
        })
    }

    fn eval_factor(&mut self, factor: &Factor) -> Result<f64, Error> {
        factor.try_collapse_frames(|frame| match frame {
            FactorFrame::Literal(a) => Ok(a),
            FactorFrame::Variable(ident) => self
                .bindings
                .get(ident.as_str())
                .copied()
                .ok_or_else(|| format!("Variable not found in bindings: {}", ident).into()),
            FactorFrame::Group(e) => self.eval_expr(e.as_ref()),
            FactorFrame::BinaryOp { op, lhs, rhs } => match op {
                FactorBinaryOp::Mul => Ok(lhs * rhs),
                FactorBinaryOp::Div => Ok(lhs / rhs),
            },
        })
    }
}
