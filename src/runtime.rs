use std::collections::HashMap;

use crate::ast::{Function, Identifier};

pub(crate) type Env = HashMap<Identifier, Value>;

#[derive(Debug, Clone)]
pub(crate) enum Value {
    Number(f64),
    Closure(Box<Function>, Env),
}
