pub(crate) type Identifier = String;

#[derive(Debug, Clone)]
pub(crate) enum Stmt {
    Let(Identifier, Box<Expr>),
    Fun(Identifier, Box<Function>),
}

#[derive(Debug, Clone)]
pub(crate) struct Function {
    pub params: Params,
    pub body: Expr,
}

pub(crate) type Params = Vec<Identifier>;

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
    #[allow(dead_code)]
    Sub,
}

#[derive(Debug, Clone)]
pub(crate) enum Factor {
    Timeout { milliseconds: u64, expr: Box<Expr> },
    Yield(Box<Expr>),
    Literal(f64),
    Variable(Identifier),
    #[allow(dead_code)]
    Group(Box<Expr>),
    #[allow(dead_code)]
    BinaryOp {
        op: FactorBinaryOp,
        lhs: Box<Factor>,
        rhs: Box<Factor>,
    },
    Call(Box<Factor>, Vec<Expr>),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum FactorBinaryOp {
    #[allow(dead_code)]
    Mul,
    #[allow(dead_code)]
    Div,
}
