use crate::{
    ast::{Expr, Factor, FactorBinaryOp, Stmt, TermBinaryOp},
    runtime::{Env, Value},
};

pub(crate) type Error = Box<dyn std::error::Error>;

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

#[derive(Debug, Default)]
pub(crate) struct Evaluator {
    pub bindings: Env,
    pub visitors: Vec<Box<dyn Visit>>,
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
        // Pre-order visitors.
        for visitor in self.visitors.iter_mut() {
            visitor.pre_visit_stmt(stmt)?;
        }
        // Evaluate.
        let mut result = self.inner_eval_stmt(stmt);
        // Post-order visitors.
        for visitor in self.visitors.iter_mut().rev() {
            let visit_result = visitor.post_visit_stmt(stmt, &result);
            match (visit_result, &result) {
                (Ok(_), _) => {}
                (Err(err), Ok(_)) => {
                    result = Err(err);
                }
                (Err(visit_err), Err(eval_err)) => {
                    // TODO: Combine errors.
                    panic!("Double error: eval_err={eval_err:?}, visit_err={visit_err:?}");
                }
            }
        }

        result
    }

    fn inner_eval_stmt(&mut self, stmt: &Stmt) -> Result<Value, Error> {
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
        // Pre-order visitors.
        for visitor in self.visitors.iter_mut() {
            visitor.pre_visit_expr(expr)?;
        }
        // Evaluate.
        let mut result = self.inner_eval_expr(expr);
        // Post-order visitors.
        for visitor in self.visitors.iter_mut().rev() {
            let visit_result = visitor.post_visit_expr(expr, &result);
            match (visit_result, &result) {
                (Ok(_), _) => {}
                (Err(err), Ok(_)) => {
                    result = Err(err);
                }
                (Err(visit_err), Err(eval_err)) => {
                    // TODO: Combine errors.
                    panic!("Double error: eval_err={eval_err:?}, visit_err={visit_err:?}");
                }
            }
        }

        result
    }

    fn inner_eval_expr(&mut self, expr: &Expr) -> Result<Value, Error> {
        match expr {
            Expr::Factor(f) => self.eval_factor(f),
            Expr::BinaryOp { op, lhs, rhs } => {
                let lhs = self.eval_expr(lhs)?;
                let rhs = self.eval_expr(rhs)?;
                match op {
                    TermBinaryOp::Add => match (lhs, rhs) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                        _ => Err("Addition only supported for numbers".into()),
                    },
                    TermBinaryOp::Sub => match (lhs, rhs) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                        _ => Err("Subtraction only supported for numbers".into()),
                    },
                }
            }
        }
    }

    fn eval_factor(&mut self, factor: &Factor) -> Result<Value, Error> {
        // Pre-order visitors.
        for visitor in self.visitors.iter_mut() {
            visitor.pre_visit_factor(factor)?;
        }
        // Evaluate.
        let mut result = self.inner_eval_factor(factor);
        // Post-order visitors.
        for visitor in self.visitors.iter_mut().rev() {
            let visit_result = visitor.post_visit_factor(factor, &result);
            match (visit_result, &result) {
                (Ok(_), _) => {}
                (Err(err), Ok(_)) => {
                    result = Err(err);
                }
                (Err(visit_err), Err(eval_err)) => {
                    // TODO: Combine errors.
                    panic!("Double error: eval_err={eval_err:?}, visit_err={visit_err:?}");
                }
            }
        }

        result
    }

    fn inner_eval_factor(&mut self, factor: &Factor) -> Result<Value, Error> {
        match factor {
            Factor::Literal(x) => Ok(Value::Number(*x)),
            Factor::Variable(ident) => self
                .bindings
                .get(ident)
                .cloned()
                .ok_or_else(|| format!("Variable not found in bindings: {}", ident).into()),
            Factor::Group(expr) => self.eval_expr(expr),
            Factor::BinaryOp { op, lhs, rhs } => {
                let lhs = self.eval_factor(lhs)?;
                let rhs = self.eval_factor(rhs)?;
                match op {
                    FactorBinaryOp::Mul => match (lhs, rhs) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                        _ => Err("Multiplication only supported for numbers".into()),
                    },
                    FactorBinaryOp::Div => match (lhs, rhs) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
                        _ => Err("Division only supported for numbers".into()),
                    },
                }
            }
            Factor::Call(fun, args) => {
                let fun = self.eval_factor(fun)?;
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
        }
    }
}
