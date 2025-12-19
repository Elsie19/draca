use std::fmt::Display;

use pest_consume::{Parser, match_nodes};

use crate::env::Environment;

#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Expression {
    Bool(bool),
    Number(f64),
    Symbol(String),
    String(String),
    List(Vec<Expression>),
    Func(fn(&[Expression]) -> std::result::Result<Expression, String>),
    Function(Procedure),
    Nil,
    Quoted(Box<Expression>),
}

impl Expression {
    pub fn symbol<T: Into<String>>(symbol: T) -> Self {
        Self::Symbol(symbol.into())
    }

    pub fn string<T: Into<String>>(string: T) -> Self {
        Self::String(string.into())
    }

    pub fn list<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Expression>,
    {
        Self::List(iter.into_iter().collect())
    }

    pub fn fmt_string(&self) -> String {
        match self {
            Self::Bool(b) => {
                if *b {
                    String::from("#t")
                } else {
                    String::from("#f")
                }
            }
            Self::Number(n) => n.to_string(),
            Self::String(s) => s.clone(),
            Self::Nil => String::from("nil"),
            Self::Quoted(fmt) => format!("'{}", fmt.fmt_string()),
            Self::List(lst) => format!(
                "({})",
                lst.iter()
                    .map(Self::fmt_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            Self::Function(_) => String::from("<function>"),
            Self::Func(_) => String::from("<fn>"),
            Self::Symbol(s) => s.clone(),
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(b) => write!(f, "{}", if *b { "#t" } else { "#f" }),
            Self::Number(n) => write!(f, "{n}"),
            Self::Symbol(s) => write!(f, "{s}"),
            Self::Nil => write!(f, "nil"),
            Self::String(s) => write!(f, "\"{s}\""),
            Self::List(list) => {
                let formatted_list: Vec<_> = list.iter().map(ToString::to_string).collect();
                write!(f, "({})", formatted_list.join(" "))
            }
            Self::Func(func) => write!(f, "<{:p}>", *func as *const ()),
            Self::Function(func) => {
                write!(
                    f,
                    "<fn>({}): {}",
                    func.params
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", "),
                    func.body
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
            Self::Quoted(qt) => write!(f, "'{}", qt),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Procedure {
    pub params: Vec<Expression>,
    pub body: Vec<Expression>,
    pub env: Environment,
}

type Result<T> = std::result::Result<T, pest_consume::Error<Rule>>;
type Node<'i> = pest_consume::Node<'i, Rule, ()>;

#[derive(Parser)]
#[grammar = "src/grammar.pest"]
struct Parse;

#[pest_consume::parser]
impl Parse {
    fn EOI(_input: Node) -> Result<()> {
        Ok(())
    }

    fn number(input: Node) -> Result<f64> {
        input.as_str().parse().map_err(|e| input.error(e))
    }

    fn strinner(input: Node<'_>) -> Result<&str> {
        Ok(input.as_str())
    }

    fn string(input: Node) -> Result<String> {
        Ok(match_nodes!(input.into_children();
            [strinner(st)] => st.to_string(),
        ))
    }

    fn symbol(input: Node) -> Result<String> {
        Ok(input.as_str().to_string())
    }

    fn nil(_input: Node) -> Result<Expression> {
        Ok(Expression::Nil)
    }

    fn bool(input: Node) -> Result<Expression> {
        Ok(Expression::Bool(match input.as_str() {
            "#t" => true,
            "#f" | _ => false,
        }))
    }

    fn quoted(input: Node) -> Result<Expression> {
        Ok(match_nodes!(input.into_children();
            [form(fm)] => Expression::Quoted(Box::new(fm)),
        ))
    }

    fn list(input: Node) -> Result<Expression> {
        Ok(match_nodes!(input.into_children();
            [form(fms)..] => Expression::List(fms.collect()),
        ))
    }

    fn form(input: Node) -> Result<Expression> {
        Ok(match_nodes!(input.into_children();
            [number(n)] => Expression::Number(n),
            [quoted(q)] => q,
            [nil(n)] => n,
            [bool(b)] => b,
            [string(s)] => Expression::String(s),
            [symbol(s)] => Expression::Symbol(s),
            [list(l)] => l,
        ))
    }

    fn program(input: Node) -> Result<Vec<Expression>> {
        Ok(match_nodes!(input.into_children();
            [list(lst).., _] => lst.collect(),
        ))
    }
}

pub fn parse(input: &str) -> Result<Vec<Expression>> {
    let inputs = Parse::parse(Rule::program, input)?;

    let input = inputs.single()?;

    Parse::program(input)
}
