use std::collections::HashMap;

use crate::ast::{Expr, Factor, Function, Identifier, Stmt};

pub(crate) type Error = Box<dyn std::error::Error>;

pub(crate) type Env = HashMap<Identifier, Value>;

#[derive(Debug, Clone)]
pub(crate) enum Value {
    Number(f64),
    Closure(Box<Function>, Env),
}

pub(crate) trait Visit: std::fmt::Debug {
    fn pre_visit_stmt(&mut self, stmt: &Stmt) -> Result<(), Error>;
    fn post_visit_stmt(&mut self, stmt: &Stmt, result: &Result<Value, Error>) -> Result<(), Error>;

    fn pre_visit_expr(&mut self, expr: &Expr) -> Result<(), Error>;
    fn post_visit_expr(&mut self, expr: &Expr, result: &Result<Value, Error>) -> Result<(), Error>;

    fn pre_visit_factor(&mut self, factor: &Factor) -> Result<(), Error>;
    fn post_visit_factor(
        &mut self,
        factor: &Factor,
        result: &Result<Value, Error>,
    ) -> Result<(), Error>;
}
