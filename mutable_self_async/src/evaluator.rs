use async_recursion::async_recursion;

use crate::{
    ast::{Expr, Factor, FactorBinaryOp, Stmt, TermBinaryOp},
    runtime::{Env, Error, Value, Visit},
};

#[derive(Debug, Default)]
pub(crate) struct Evaluator {
    pub bindings: Env,
    pub visitors: Vec<Box<dyn Visit + Send>>,
}

impl Evaluator {
    pub(crate) async fn eval_stmts(&mut self, stmts: &[Stmt]) -> Result<Value, Error> {
        let mut last = Value::Number(0.0);
        for stmt in stmts {
            last = self.eval_stmt(stmt).await?;
        }
        Ok(last)
    }

    pub(crate) async fn eval_stmt(&mut self, stmt: &Stmt) -> Result<Value, Error> {
        // Pre-order visitors.
        for visitor in self.visitors.iter_mut() {
            visitor.pre_visit_stmt(stmt)?;
        }
        // Evaluate.
        let mut result = self.inner_eval_stmt(stmt).await;
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

    async fn inner_eval_stmt(&mut self, stmt: &Stmt) -> Result<Value, Error> {
        match stmt {
            Stmt::Let(ident, expr) => {
                let value = self.eval_expr(expr).await?;
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

    #[async_recursion]
    pub(crate) async fn eval_expr(&mut self, expr: &Expr) -> Result<Value, Error> {
        // Pre-order visitors.
        for visitor in self.visitors.iter_mut() {
            visitor.pre_visit_expr(expr)?;
        }
        // Evaluate.
        let mut result = self.inner_eval_expr(expr).await;
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

    async fn inner_eval_expr(&mut self, expr: &Expr) -> Result<Value, Error> {
        match expr {
            Expr::Factor(f) => self.eval_factor(f).await,
            Expr::BinaryOp { op, lhs, rhs } => {
                let lhs = self.eval_expr(lhs).await?;
                let rhs = self.eval_expr(rhs).await?;

                // Sleep to reliably trigger a timeout.
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;

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

    #[async_recursion]
    async fn eval_factor(&mut self, factor: &Factor) -> Result<Value, Error> {
        // Pre-order visitors.
        for visitor in self.visitors.iter_mut() {
            visitor.pre_visit_factor(factor)?;
        }
        // Evaluate.
        let mut result = self.inner_eval_factor(factor).await;
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

    async fn inner_eval_factor(&mut self, factor: &Factor) -> Result<Value, Error> {
        match factor {
            Factor::Timeout { milliseconds, expr} => {
                let duration = std::time::Duration::from_millis(*milliseconds);
                tokio::time::timeout(duration, self.eval_expr(expr)).await?
            }
            Factor::Yield(expr) => {
                tokio::task::yield_now().await;
                self.eval_expr(expr).await
            }
            Factor::Literal(x) => Ok(Value::Number(*x)),
            Factor::Variable(ident) => self
                .bindings
                .get(ident)
                .cloned()
                .ok_or_else(|| format!("Variable not found in bindings: {}", ident).into()),
            Factor::Group(expr) => self.eval_expr(expr).await,
            Factor::BinaryOp { op, lhs, rhs } => {
                let lhs = self.eval_factor(lhs).await?;
                let rhs = self.eval_factor(rhs).await?;
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
                let fun = self.eval_factor(fun).await?;
                let Value::Closure(fun, env) = fun else {
                    return Err("Expected closure".into());
                };
                let mut body_env = env.clone();
                for (param, arg) in fun.params.iter().zip(args.iter()) {
                    body_env.insert(param.clone(), self.eval_expr(arg).await?);
                }
                let current_bindings = std::mem::replace(&mut self.bindings, body_env);
                let result = self.eval_expr(&fun.body).await;
                self.bindings = current_bindings;
                result
            }
        }
    }
}
