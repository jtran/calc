//! Type checking.

use std::collections::HashMap;

use crate::{
    ast::{Expr, Factor, Identifier, Stmt, Type},
    runtime::Error,
};

pub(crate) type Env = HashMap<Identifier, Type>;

#[derive(Debug, Default)]
pub(crate) struct TypeChecker {
    pub bindings: Env,
}

impl TypeChecker {
    pub(crate) fn check_stmts(&mut self, stmts: &[Stmt]) -> Result<(), Error> {
        for stmt in stmts {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }

    pub(crate) fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), Error> {
        match stmt {
            Stmt::Let(ident, ty, expr) => {
                let expr_ty = self.check_expr(expr)?;
                if ty != &expr_ty {
                    return Err(format!("Type mismatch: expected {}, found {}", ty, expr_ty).into());
                }
                self.bindings.insert(ident.clone(), ty.clone());
            }
            Stmt::Fun(ident, function) => {
                let fun_ty = Type::Arrow(
                    function
                        .params
                        .iter()
                        .map(|param| param.ty.clone())
                        .collect(),
                    Box::new(function.return_ty.clone()),
                );
                let body_ty = {
                    for param in &function.params {
                        self.bindings.insert(param.name.clone(), param.ty.clone());
                    }
                    // Check the function body with parameters in scope.
                    let result = self.check_expr(&function.body);
                    // Remove parameters.
                    for param in function.params.iter().rev() {
                        self.bindings.remove(&param.name);
                    }
                    result
                }?;
                if body_ty != function.return_ty {
                    return Err(format!(
                        "Function body type doesn't match annotated return type: annotated {}, found {}",
                        function.return_ty, body_ty
                    )
                    .into());
                }
                self.bindings.insert(ident.clone(), fun_ty);
            }
        }
        Ok(())
    }

    pub(crate) fn check_expr(&mut self, expr: &Expr) -> Result<Type, Error> {
        match expr {
            Expr::Factor(factor) => self.check_factor(factor),
            Expr::BinaryOp { op: _op, lhs, rhs } => {
                let t1 = self.check_expr(lhs)?;
                let t2 = self.check_expr(rhs)?;
                // All ops currently expect the same types.
                let expected_ty = Type::Number;
                if t1 != expected_ty {
                    return Err(
                        format!("Type mismatch: expected {}, found {}", expected_ty, t1).into(),
                    );
                }
                if t2 != expected_ty {
                    return Err(
                        format!("Type mismatch: expected {}, found {}", expected_ty, t2).into(),
                    );
                }
                Ok(Type::Number)
            }
        }
    }

    pub(crate) fn check_factor(&mut self, factor: &Factor) -> Result<Type, Error> {
        match factor {
            Factor::Literal(_) => Ok(Type::Number),
            Factor::Variable(ident) => self
                .bindings
                .get(ident.as_str())
                .cloned()
                .ok_or_else(|| format!("Undefined variable: {ident}",).into()),
            Factor::Group(expr) => self.check_expr(expr),
            Factor::BinaryOp { op: _op, lhs, rhs } => {
                let t1 = self.check_factor(lhs)?;
                let t2 = self.check_factor(rhs)?;
                // All ops currently expect the same types.
                let expected_ty = Type::Number;
                if t1 != expected_ty {
                    return Err(
                        format!("Type mismatch: expected {}, found {}", expected_ty, t1).into(),
                    );
                }
                if t2 != expected_ty {
                    return Err(
                        format!("Type mismatch: expected {}, found {}", expected_ty, t2).into(),
                    );
                }
                Ok(Type::Number)
            }
            Factor::Call(fun, args) => {
                let fun_ty = self.check_factor(fun)?;
                let (param_tys, return_ty) = match fun_ty {
                    Type::Arrow(param_tys, return_ty) => (param_tys, *return_ty),
                    _ => return Err(format!("Expected function, found {fun_ty}").into()),
                };
                if param_tys.len() != args.len() {
                    return Err(format!(
                        "Number of function parameters differs from arguments: expected {}, found {}",
                        param_tys.len(),
                        args.len()
                    )
                    .into());
                }
                for (param_ty, arg) in param_tys.iter().zip(args.iter()) {
                    let arg_ty = self.check_expr(arg)?;
                    if param_ty != &arg_ty {
                        return Err(format!(
                            "Type mismatch: expected {}, found {}",
                            param_ty, arg_ty
                        )
                        .into());
                    }
                }
                Ok(return_ty)
            }
        }
    }
}
