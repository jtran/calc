use crate::{
    ast::{Expr, Factor, Stmt},
    runtime::{self, Error, Value},
};

#[derive(Debug, Default)]
pub(crate) struct Print {
    pub indent: usize,
}

const INDENT_WIDTH: usize = 2;

impl runtime::Visit for Print {
    fn pre_visit_stmt(&mut self, stmt: &Stmt) -> Result<(), Error> {
        println!(
            "{:indent$}Eval stmt={stmt:?}",
            "",
            indent = self.indent * INDENT_WIDTH,
            stmt = stmt
        );
        self.indent += 1;
        Ok(())
    }

    fn post_visit_stmt(&mut self, stmt: &Stmt, result: &Result<Value, Error>) -> Result<(), Error> {
        self.indent -= 1;
        println!(
            "{:indent$}Eval stmt={stmt:?} result={result:?}",
            "",
            indent = self.indent * INDENT_WIDTH,
            stmt = stmt,
            result = result
        );
        Ok(())
    }

    fn pre_visit_expr(&mut self, expr: &Expr) -> Result<(), Error> {
        println!(
            "{:indent$}Eval expr={expr:?}",
            "",
            indent = self.indent * INDENT_WIDTH,
            expr = expr
        );
        self.indent += 1;
        Ok(())
    }

    fn post_visit_expr(&mut self, expr: &Expr, result: &Result<Value, Error>) -> Result<(), Error> {
        self.indent -= 1;
        println!(
            "{:indent$}Eval expr={expr:?} result={result:?}",
            "",
            indent = self.indent * INDENT_WIDTH,
            expr = expr,
            result = result
        );
        Ok(())
    }

    fn pre_visit_factor(&mut self, factor: &Factor) -> Result<(), Error> {
        println!(
            "{:indent$}Eval factor={factor:?}",
            "",
            indent = self.indent * INDENT_WIDTH,
            factor = factor
        );
        self.indent += 1;
        Ok(())
    }

    fn post_visit_factor(
        &mut self,
        factor: &Factor,
        result: &Result<Value, Error>,
    ) -> Result<(), Error> {
        self.indent -= 1;
        println!(
            "{:indent$}Eval factor={factor:?} result={result:?}",
            "",
            indent = self.indent * INDENT_WIDTH,
            factor = factor,
            result = result
        );
        Ok(())
    }
}
