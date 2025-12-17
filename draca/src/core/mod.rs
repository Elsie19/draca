pub mod cmp;
pub mod macros;
pub mod math;
pub mod sys;

#[macro_export]
macro_rules! lisp {
    ($code:expr, $env:expr) => {{
        let ast = $crate::parser::parse($code.trim()).expect("Could not parse text!!!");
        $crate::eval::eval_expr(ast.first().unwrap().clone(), $env).expect("Could not eval!!!")
    }};
}
