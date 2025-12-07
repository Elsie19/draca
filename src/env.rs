use std::collections::HashMap;

use crate::parser::Expression;
use crate::stdlib;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    contents: HashMap<String, Expression>,
}

impl Environment {
    fn new() -> Self {
        Self {
            contents: HashMap::new(),
        }
    }

    pub fn math_std_env() -> Self {
        let mut env = Self::new();

        env.insert(
            "+".to_string(),
            Expression::Func(|args: &[Expression]| match stdlib::math::add(args) {
                Ok(expr) => expr,
                Err(e) => panic!("{e}"),
            }),
        );

        env.insert(
            "-".to_string(),
            Expression::Func(|args: &[Expression]| match stdlib::math::sub(args) {
                Ok(expr) => expr,
                Err(e) => panic!("{e}"),
            }),
        );

        env.insert(
            "*".to_string(),
            Expression::Func(|args: &[Expression]| match stdlib::math::mul(args) {
                Ok(expr) => expr,
                Err(e) => panic!("{e}"),
            }),
        );

        env.insert(
            "/".to_string(),
            Expression::Func(|args: &[Expression]| match stdlib::math::div(args) {
                Ok(expr) => expr,
                Err(e) => panic!("{e}"),
            }),
        );

        env.insert(
            "=".to_string(),
            Expression::Func(|args: &[Expression]| match stdlib::cmp::eq(args) {
                Ok(expr) => expr,
                Err(e) => panic!("{e}"),
            }),
        );

        env
    }

    pub fn insert(&mut self, k: String, v: Expression) {
        self.contents.insert(k, v);
    }

    pub fn get(&self, k: &str) -> Option<&Expression> {
        self.contents.get(k)
    }
}
