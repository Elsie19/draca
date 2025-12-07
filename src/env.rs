use std::collections::HashMap;
use std::f64::consts::PI;
use std::fmt::Display;

use crate::parser::Expression;
use crate::stdlib;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Namespace {
    frags: Vec<String>,
}

impl Namespace {
    pub fn new<I, T>(it: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        Self::from(it)
    }

    pub fn push<T: Into<String>>(&mut self, t: T) {
        self.frags.push(t.into());
    }

    pub fn join<T: Into<String>>(&self, item: T) -> NamespaceItem {
        NamespaceItem {
            frags: self.clone(),
            target: item.into(),
        }
    }

    pub fn from_string(value: String) -> Self {
        let frags = value.split("::").map(ToString::to_string).collect();
        Self { frags }
    }
}

impl<I, T> From<I> for Namespace
where
    I: IntoIterator<Item = T>,
    T: Into<String>,
{
    fn from(value: I) -> Self {
        Self {
            frags: value.into_iter().map(Into::into).collect(),
        }
    }
}

impl Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for frag in &self.frags {
            write!(f, "{frag}::")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamespaceItem {
    frags: Namespace,
    target: String,
}

impl From<&str> for NamespaceItem {
    fn from(value: &str) -> Self {
        let split = value.split("::");
        let split = split.collect::<Vec<_>>();
        let (frags, target) = match split.as_slice() {
            [frags @ .., target] => (
                frags
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .into(),
                target.to_string(),
            ),
            _ => unreachable!("oops"),
        };

        Self { frags, target }
    }
}

impl NamespaceItem {
    pub fn from_string(value: String) -> Self {
        Self::from(value.as_str())
    }

    pub fn in_namespace(frags: impl Into<Namespace>, target: impl Into<String>) -> Self {
        Self {
            frags: frags.into(),
            target: target.into(),
        }
    }
}

impl Display for NamespaceItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.frags, self.target)
    }
}

impl PartialEq<String> for NamespaceItem {
    fn eq(&self, other: &String) -> bool {
        self.to_string().eq(other)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    contents: HashMap<NamespaceItem, Expression>,
    in_scope: Vec<Namespace>,
}

impl Environment {
    fn empty() -> Self {
        Self {
            contents: HashMap::new(),
            in_scope: vec![],
        }
    }

    pub fn with_scope(mut self, ns: Namespace) -> Self {
        self.in_scope.push(ns);
        self
    }

    pub fn add_scope(&mut self, ns: Namespace) {
        self.in_scope.push(ns);
    }

    pub fn insert_item(&mut self, item: NamespaceItem, val: Expression) {
        self.contents.insert(item, val);
    }

    pub fn insert(&mut self, key: impl Into<NamespaceItem>, val: Expression) {
        self.contents.insert(key.into(), val);
    }

    pub fn get_namespace_str(&self, target: &str) -> Option<String> {
        if self.contents.contains_key(&NamespaceItem::from(target)) {
            return Some(target.to_string());
        }

        for ns in &self.in_scope {
            let try_item = NamespaceItem {
                frags: ns.clone(),
                target: target.to_string(),
            };

            if self.contents.contains_key(&try_item) {
                return Some(try_item.to_string());
            }
        }

        for item in self.contents.keys() {
            if item.target == target {
                return Some(item.to_string());
            }
        }

        None
    }

    pub fn get(&self, key: &str) -> Option<&Expression> {
        if let Some(found) = self.contents.get(&NamespaceItem::from(key)) {
            return Some(found);
        }

        for ns in &self.in_scope {
            let try_item = NamespaceItem {
                frags: ns.clone(),
                target: key.to_string(),
            };
            if let Some(found) = self.contents.get(&try_item) {
                return Some(found);
            }
        }

        None
    }

    pub fn math_std_env() -> Self {
        let mut env = Self::empty();

        env.in_scope = vec![
            ["std", "math"].into(),
            ["std", "cmp"].into(),
            ["std", "math", "consts"].into(),
            ["local"].into(),
        ];

        env.insert(
            "std::math::+",
            Expression::Func(|args: &[Expression]| match stdlib::math::add(args) {
                Ok(expr) => expr,
                Err(e) => panic!("{e}"),
            }),
        );

        env.insert(
            "std::math::-",
            Expression::Func(|args: &[Expression]| match stdlib::math::sub(args) {
                Ok(expr) => expr,
                Err(e) => panic!("{e}"),
            }),
        );

        env.insert(
            "std::math::*",
            Expression::Func(|args: &[Expression]| match stdlib::math::mul(args) {
                Ok(expr) => expr,
                Err(e) => panic!("{e}"),
            }),
        );

        env.insert(
            "std::math::/",
            Expression::Func(|args: &[Expression]| match stdlib::math::div(args) {
                Ok(expr) => expr,
                Err(e) => panic!("{e}"),
            }),
        );

        env.insert(
            "std::cmp::=",
            Expression::Func(|args: &[Expression]| match stdlib::cmp::eq(args) {
                Ok(expr) => expr,
                Err(e) => panic!("{e}"),
            }),
        );

        env.insert("std::math::consts::pi", Expression::Number(PI));

        env
    }
}
