use recursion::{Collapsible, CollapsibleExt, MappableFrame, PartiallyApplied};

use crate::ast::{Expr, Factor, FactorBinaryOp, TermBinaryOp};

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
    Group(Box<Expr>),
    BinaryOp { op: FactorBinaryOp, lhs: A, rhs: A },
}

impl MappableFrame for FactorFrame<PartiallyApplied> {
    type Frame<X> = FactorFrame<X>;

    fn map_frame<A, B>(input: Self::Frame<A>, mut f: impl FnMut(A) -> B) -> Self::Frame<B> {
        match input {
            FactorFrame::Literal(a) => FactorFrame::Literal(a),
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
pub(crate) struct Evaluator;

impl Evaluator {
    pub(crate) fn eval(&mut self, expr: &Expr) -> f64 {
        expr.collapse_frames(|frame| match frame {
            ExprFrame::Factor(f) => self.eval_factor(f.as_ref()),
            ExprFrame::BinaryOp { op, lhs, rhs } => match op {
                TermBinaryOp::Add => lhs + rhs,
                TermBinaryOp::Sub => lhs - rhs,
            },
        })
    }

    fn eval_factor(&mut self, factor: &Factor) -> f64 {
        factor.collapse_frames(|frame| match frame {
            FactorFrame::Literal(a) => a,
            FactorFrame::Group(e) => self.eval(e.as_ref()),
            FactorFrame::BinaryOp { op, lhs, rhs } => match op {
                FactorBinaryOp::Mul => lhs * rhs,
                FactorBinaryOp::Div => lhs / rhs,
            },
        })
    }
}
