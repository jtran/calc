use std::collections::HashMap;

use recursion::{Collapsible, CollapsibleExt, MappableFrame, PartiallyApplied};

use crate::ast::{Expr, Factor, FactorBinaryOp, Identifier, Stmt, TermBinaryOp};

type Error = Box<dyn std::error::Error>;

pub(crate) enum ExprFrame<A> {
    Factor(Box<Factor>),
    BinaryOp { op: TermBinaryOp, lhs: A, rhs: A },
}

impl MappableFrame for ExprFrame<PartiallyApplied> {
    type Frame<X> = ExprFrame<X>;

    fn map_frame<A, B>(input: Self::Frame<A>, mut f: impl FnMut(A) -> B) -> Self::Frame<B> {
        match input {
            ExprFrame::Factor(a) => ExprFrame::Factor(a),
            ExprFrame::BinaryOp { op, lhs, rhs } => ExprFrame::BinaryOp {
                op,
                lhs: f(lhs),
                rhs: f(rhs),
            },
        }
    }
}

impl<'a> Collapsible for &'a Expr {
    type FrameToken = ExprFrame<PartiallyApplied>;

    fn into_frame(self) -> <Self::FrameToken as MappableFrame>::Frame<Self> {
        match self {
            Expr::Factor(f) => ExprFrame::Factor(f.clone()),
            Expr::BinaryOp { op, lhs, rhs } => ExprFrame::BinaryOp {
                op: *op,
                lhs: lhs,
                rhs: rhs,
            },
        }
    }
}

pub(crate) enum FactorFrame<A> {
    Literal(f64),
    Variable(Identifier),
    Group(Box<Expr>),
    BinaryOp { op: FactorBinaryOp, lhs: A, rhs: A },
}

impl MappableFrame for FactorFrame<PartiallyApplied> {
    type Frame<X> = FactorFrame<X>;

    fn map_frame<A, B>(input: Self::Frame<A>, mut f: impl FnMut(A) -> B) -> Self::Frame<B> {
        match input {
            FactorFrame::Literal(a) => FactorFrame::Literal(a),
            FactorFrame::Variable(ident) => FactorFrame::Variable(ident),
            FactorFrame::Group(a) => FactorFrame::Group(a),
            FactorFrame::BinaryOp { op, lhs, rhs } => FactorFrame::BinaryOp {
                op,
                lhs: f(lhs),
                rhs: f(rhs),
            },
        }
    }
}

impl<'a> Collapsible for &'a Factor {
    type FrameToken = FactorFrame<PartiallyApplied>;

    fn into_frame(self) -> <Self::FrameToken as MappableFrame>::Frame<Self> {
        match self {
            Factor::Literal(a) => FactorFrame::Literal(*a),
            Factor::Variable(ident) => FactorFrame::Variable(ident.clone()),
            Factor::Group(e) => FactorFrame::Group(e.clone()),
            Factor::BinaryOp { op, lhs, rhs } => FactorFrame::BinaryOp {
                op: *op,
                lhs: lhs,
                rhs: rhs,
            },
        }
    }
}

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
