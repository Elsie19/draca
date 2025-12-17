pub mod cmp;
pub mod list;
pub mod macros;
pub mod math;
pub mod string;
pub mod sys;

#[macro_export]
macro_rules! lisp {
    ($code:expr, $env:expr) => {{
        let ast = $crate::parser::parse($code.trim()).expect("Could not parse text!!!");
        $crate::eval::eval_expr(ast.first().unwrap().clone(), $env).expect("Could not eval!!!")
    }};
}

#[macro_export]
macro_rules! empty_quoted_list {
    () => {{ $crate::parser::Expression::Quoted(Box::new($crate::parser::Expression::List(vec![]))) }};
}

#[macro_export]
macro_rules! num {
    ($num:expr) => {{ $crate::parser::Expression::Number($num) }};
}
