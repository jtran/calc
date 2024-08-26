pub(crate) type Identifier = String;

#[derive(Debug, Clone)]
pub(crate) enum Stmt {
    Let(Identifier, Box<Expr>),
}

#[derive(Debug, Clone)]
pub(crate) enum Expr {
    Factor(Box<Factor>),
    BinaryOp {
        op: TermBinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum TermBinaryOp {
    Add,
    Sub,
}

#[derive(Debug, Clone)]
pub(crate) enum Factor {
    Literal(f64),
    Variable(Identifier),
    Group(Box<Expr>),
    BinaryOp {
        op: FactorBinaryOp,
        lhs: Box<Factor>,
        rhs: Box<Factor>,
    },
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum FactorBinaryOp {
    Mul,
    Div,
}
