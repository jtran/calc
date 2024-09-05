//! An example of a static analysis.
use std::fmt::Write;

use crate::ast::{Expr, Factor, FactorBinaryOp, Stmt, TermBinaryOp};

#[derive(Debug, Default)]
pub(crate) struct Unparser {
    indent: usize,
    out: String,
}

const INDENT_WIDTH: usize = 2;

impl Unparser {
    pub(crate) fn output(&self) -> &str {
        &self.out
    }

    pub(crate) fn unparse_stmts(&mut self, stmts: &[Stmt]) -> Result<(), std::fmt::Error> {
        for stmt in stmts {
            self.unparse_stmt(stmt)?;
        }
        Ok(())
    }

    fn unparse_stmt(&mut self, stmt: &Stmt) -> Result<(), std::fmt::Error> {
        match stmt {
            Stmt::Let(name, expr) => {
                write!(
                    self.out,
                    "{:indent$}let {name} = ",
                    "",
                    indent = self.indent * INDENT_WIDTH,
                    name = name
                )?;
                self.unparse_expr(expr)?;
                writeln!(self.out)?;
            }
            Stmt::Fun(name, fun) => {
                write!(
                    self.out,
                    "{:indent$}fun {name}(",
                    "",
                    indent = self.indent * INDENT_WIDTH,
                    name = name
                )?;
                for (i, param) in fun.params.iter().enumerate() {
                    if i > 0 {
                        write!(self.out, ", ")?;
                    }
                    write!(self.out, "{}", param)?;
                }
                write!(self.out, ") = ")?;
                self.unparse_expr(&fun.body)?;
                writeln!(self.out)?;
            }
        }
        Ok(())
    }

    fn unparse_expr(&mut self, expr: &Expr) -> Result<(), std::fmt::Error> {
        match expr {
            Expr::Factor(factor) => self.unparse_factor(factor)?,
            Expr::BinaryOp { op, lhs, rhs } => {
                self.unparse_expr(lhs)?;
                write!(self.out, " ")?;
                self.unparse_term_binary_op(op)?;
                write!(self.out, " ")?;
                self.unparse_expr(rhs)?;
            }
        }
        Ok(())
    }

    fn unparse_term_binary_op(&mut self, op: &TermBinaryOp) -> Result<(), std::fmt::Error> {
        match op {
            TermBinaryOp::Add => write!(self.out, "+"),
            TermBinaryOp::Sub => write!(self.out, "-"),
        }
    }

    fn unparse_factor(&mut self, factor: &Factor) -> Result<(), std::fmt::Error> {
        match factor {
            Factor::Literal(value) => write!(self.out, "{}", value)?,
            Factor::Variable(name) => write!(self.out, "{}", name)?,
            Factor::Group(expr) => {
                write!(self.out, "(")?;
                self.unparse_expr(expr)?;
                write!(self.out, ")")?;
            }
            Factor::BinaryOp { op, lhs, rhs } => {
                self.unparse_factor(lhs)?;
                write!(self.out, " ")?;
                self.unparse_factor_binary_op(op)?;
                write!(self.out, " ")?;
                self.unparse_factor(rhs)?;
            }
            Factor::Call(fun, args) => {
                self.unparse_factor(fun)?;
                write!(self.out, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(self.out, ", ")?;
                    }
                    self.unparse_expr(arg)?;
                }
                write!(self.out, ")")?;
            }
        }
        Ok(())
    }

    fn unparse_factor_binary_op(&mut self, op: &FactorBinaryOp) -> Result<(), std::fmt::Error> {
        match op {
            FactorBinaryOp::Mul => write!(self.out, "*"),
            FactorBinaryOp::Div => write!(self.out, "/"),
        }
    }
}
