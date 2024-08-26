use recursion::{Collapsible, MappableFrame, PartiallyApplied};

use crate::ast::{Expr, Factor, FactorBinaryOp, TermBinaryOp};

pub(crate) enum ExprFrame<A> {
    Factor(Box<FactorFrame<A>>),
    BinaryOp {
        op: TermBinaryOp,
        lhs: A,
        rhs: A,
    },
}

impl MappableFrame for ExprFrame<PartiallyApplied> {
    type Frame<X> = ExprFrame<X>;

    fn map_frame<A, B>(input: Self::Frame<A>, mut f: impl FnMut(A) -> B) -> Self::Frame<B> {
        match input {
            ExprFrame::Factor(a) => ExprFrame::Factor(Box::new(FactorFrame::map_frame(*a, &mut f))),
            ExprFrame::BinaryOp { op, lhs, rhs } => {
                ExprFrame::BinaryOp {
                    op,
                    lhs: f(lhs),
                    rhs: f(rhs),
                }
            }
        }
    }
}

impl<'a> Collapsible for &'a Expr {
    type FrameToken = ExprFrame<PartiallyApplied>;

    fn into_frame(self) -> <Self::FrameToken as MappableFrame>::Frame<Self> {
        match self {
            Expr::Factor(f) => ExprFrame::Factor(Box::new(f.into_frame())),
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
    Group(Box<ExprFrame<A>>),
    BinaryOp {
        op: FactorBinaryOp,
        lhs: A,
        rhs: A,
    },
}

impl MappableFrame for FactorFrame<PartiallyApplied> {
    type Frame<X> = FactorFrame<X>;

    fn map_frame<A, B>(input: Self::Frame<A>, mut f: impl FnMut(A) -> B) -> Self::Frame<B> {
        match input {
            FactorFrame::Literal(a) => FactorFrame::Literal(a),
            FactorFrame::Group(a) => FactorFrame::Group(Box::new(ExprFrame::map_frame(*a, &mut f))),
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
            Factor::Group(e) => FactorFrame::Group(Box::new(e.into_frame())),
            Factor::BinaryOp { op, lhs, rhs } => FactorFrame::BinaryOp {
                op: *op,
                lhs: lhs,
                rhs: rhs,
            },
        }
    }
}

// trait Collapse {
//     fn collapse_expr<A>(expr: Expr) -> ExprFrame<A>;
//     fn collapse_factor<A>(factor: Factor) -> FactorFrame<A>;
// }

// struct Evaluator;

// impl Collapse for Evaluator {
//     fn collapse_expr<A>(expr: Expr) -> ExprFrame<A> { todo!() }
//     fn collapse_factor<A>(factor: Factor) -> FactorFrame<A> { todo!() }
// }

// impl Evaluator {
//     fn eval(self, expr: &Expr) -> f64 {
//         expr.collapse_frames(self)
//     }
// }
