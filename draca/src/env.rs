use std::collections::BTreeMap;
use std::f64::consts::{E, PI};
use std::fmt::Display;

use crate::{lisp, parser::Expression, stdlib};

macro_rules! env_insert {
    ($env:expr => $($entry:tt),* $(,)?) => {
        $(
            env_insert!(@one $env, $entry);
        )*
    };
    (@one $env:expr, ($name:expr, fn => $run:path)) => {
        $env.insert(
            $name,
            Expression::Func(|args: &[Expression]| $run(args))
        );
    };
    (@one $env:expr, ($name:expr, const => $val:expr)) => {
        $env.insert($name, $val);
    };
}

// TODO: replace with [`jupiter`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

    pub fn from_str(value: &str) -> Self {
        let frags = value.split("::").map(ToString::to_string).collect();
        Self { frags }
    }

    pub fn as_str(&self, join: &str) -> String {
        self.frags.join(join)
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
        let mut iter = self.frags.iter().peekable();

        while let Some(frag) = iter.next() {
            if iter.peek().is_none() {
                write!(f, "{frag}")?;
            } else {
                write!(f, "{frag}::")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
                (*target).to_string(),
            ),
            _ => unreachable!("oops"),
        };

        Self { frags, target }
    }
}

impl NamespaceItem {
    pub fn from_str(value: &str) -> Self {
        Self::from(value)
    }

    pub fn in_namespace(frags: impl Into<Namespace>, target: impl Into<String>) -> Self {
        Self {
            frags: frags.into(),
            target: target.into(),
        }
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn frags(&self) -> Namespace {
        self.frags.clone()
    }
}

impl Display for NamespaceItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.frags, self.target)
    }
}

impl PartialEq<String> for NamespaceItem {
    fn eq(&self, other: &String) -> bool {
        self.to_string().eq(other)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Environment {
    contents: BTreeMap<NamespaceItem, Expression>,
    in_scope: Vec<Namespace>,
}

impl Environment {
    pub fn empty() -> Self {
        Self {
            contents: BTreeMap::new(),
            in_scope: vec![],
        }
    }

    pub fn scopes(&self) -> &[Namespace] {
        &self.in_scope
    }

    pub fn full_path_and_name(&self) -> Vec<(String, &str)> {
        self.contents
            .keys()
            .map(|key| (key.to_string(), key.target()))
            .collect()
    }

    pub fn values(&self) -> Vec<&str> {
        let keys = self.contents.keys();
        keys.into_iter().map(|key| key.target()).collect()
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

    pub fn rust_builtins(mut self) -> Self {
        // MACROS //

        self.add_scope(["std", "macros"].into());

        env_insert![self =>
            ("std::macros::panic",  fn => stdlib::macros::panic),
            ("std::macros::format",  fn => stdlib::macros::format),
            ("std::macros::println",  fn => stdlib::macros::println),
        ];

        // SYSTEM COMPONENTS //

        env_insert![self =>
            ("std::sys::exit",  fn => stdlib::sys::exit),
        ];

        // NUMERICAL COMPARISONS //

        self.add_scope(["std", "cmp"].into());

        env_insert![self =>
            ("std::cmp::=",   fn => stdlib::cmp::eq),
            ("std::cmp::/=",  fn => stdlib::cmp::ne),
            ("std::cmp::>",   fn => stdlib::cmp::gt),
            ("std::cmp::<",   fn => stdlib::cmp::lt),
            ("std::cmp::>=",  fn => stdlib::cmp::ge),
            ("std::cmp::<=",  fn => stdlib::cmp::le),
        ];

        // MATH //

        self.add_scope(["std", "math"].into());
        self.add_scope(["std", "math", "consts"].into());

        env_insert![self =>
            ("std::math::+",           fn => stdlib::math::add),
            ("std::math::-",           fn => stdlib::math::sub),
            ("std::math::*",           fn => stdlib::math::mul),
            ("std::math::/",           fn => stdlib::math::div),
            ("std::math::rem",         fn => stdlib::math::rem),
            ("std::math::pow",         fn => stdlib::math::pow),
            ("std::math::consts::pi",  const => Expression::Number(PI)),
            ("std::math::consts::e",   const => Expression::Number(E)),
        ];

        self
    }

    pub fn stdlib(mut self) -> Self {
        self.add_scope(["std", "math", "fns"].into());

        let square = lisp!("(define (square x) (* x x))", &mut self);
        let abs = lisp!("(define (abs x) (if (< x 0) (- x) x))", &mut self);

        env_insert![self =>
            ("std::math::fns::square", const => square),
            ("std::math::fns::abs",    const => abs),
        ];

        self
    }

    pub fn build(self) -> Self {
        self
    }
}
