use std::fs;

use crate::{
    env::{Environment, Namespace, NamespaceItem},
    parser::{Expression, Procedure},
};

pub fn eval(expr: Expression, env: &mut Environment) -> Result<Expression, String> {
    let evaluated_expression = eval_expr(expr, env)?;

    Ok(evaluated_expression)
}

pub(crate) fn eval_expr(expr: Expression, env: &mut Environment) -> Result<Expression, String> {
    match expr {
        Expression::Bool(_)
        | Expression::Number(_)
        | Expression::Func(_)
        | Expression::Quoted(_) // Pass as is.
        | Expression::Nil
        | Expression::String(_) => Ok(expr),
        Expression::Symbol(s) => env
            .get(&s)
            .cloned()
            .ok_or_else(|| format!("Undefined symbol: {s}")),
        Expression::List(list) => eval_list(&list, env),
        Expression::Function(_) => Err("Unexpected function definition".into()),
    }
}

pub(crate) fn eval_list(list: &[Expression], env: &mut Environment) -> Result<Expression, String> {
    let Expression::Symbol(head) = &list[0] else {
        eprintln!("{:?}", list[0]);
        return Err("Expected a symbol".into());
    };

    match head.as_str() {
        "define" => eval_define(list, env),
        "define/in-namespace" => eval_define_namespace(list, env),
        "namespace/symbol" => eval_symbol_namespace(list, env),
        "namespace/as-list" => eval_symbol_namespace_as_list(list, env),
        "quote" => eval_quote(list, env),
        "eval-file" => eval_file(list, env),
        "require" => eval_require(list, env),
        "deconst-fn" => eval_deconst_fn(list, env),
        "if" => eval_if(list, env),
        "let" => eval_let(list, env),
        "lambda" => eval_lambda(list, env),
        _ => apply_function(head, &list[1..], env),
    }
}

fn apply_function(
    name: &str,
    args: &[Expression],
    env: &mut Environment,
) -> Result<Expression, String> {
    let Some(exp) = &env.get(name).cloned() else {
        return Err(format!("Undefined function: {name}"));
    };

    match exp {
        Expression::Func(func) => {
            let args = args
                .iter()
                .map(|e| eval_expr(e.clone(), env))
                .collect::<Result<Vec<_>, _>>()?;

            func(&args)
        }

        Expression::Function(proc) => {
            let args = args
                .iter()
                .map(|e| eval_expr(e.clone(), env))
                .collect::<Result<Vec<_>, _>>()?;

            let mut local_env = proc.env.clone();
            local_env.insert(NamespaceItem::from_str(name), exp.clone());

            for (param, arg) in proc.params.iter().zip(args) {
                let Expression::Symbol(p) = param else {
                    return Err("Invalid parameter name".into());
                };
                local_env.insert(NamespaceItem::from_str(p), arg);
            }

            let mut result = Expression::Bool(false);
            for expr in &proc.body {
                result = eval_expr(expr.clone(), &mut local_env)?;
            }

            Ok(result)
        }

        _ => Err(format!("Undefined function: {name}")),
    }
}

fn eval_define(list: &[Expression], env: &mut Environment) -> Result<Expression, String> {
    if list.len() < 3 {
        return Err("`define` requires at least two arguments".into());
    }

    match &list[1] {
        Expression::List(func) => {
            let Some(Expression::Symbol(name)) = func.first() else {
                return Err("Invalid define syntax".into());
            };

            let proc = Procedure {
                params: func[1..].to_vec(),
                body: list[2..].to_vec(),
                env: env.clone(),
            };

            env.insert(NamespaceItem::from_str(name), Expression::Function(proc));
            Ok(Expression::Symbol(name.clone()))
        }

        Expression::Symbol(name) => {
            let value = eval_expr(list[2].clone(), env)?;
            env.insert(NamespaceItem::from_str(name), value);
            Ok(Expression::Symbol(name.clone()))
        }

        _ => Err("Invalid define syntax".into()),
    }
}

fn eval_define_namespace(list: &[Expression], env: &mut Environment) -> Result<Expression, String> {
    if list.len() < 3 {
        return Err("`define/in-namespace` requires at least two arguments".into());
    }

    match &list[1] {
        Expression::List(_) => eval_define(list, env),

        // Namespace form
        Expression::Symbol(ns_name) => {
            let namespace = NamespaceItem::from(ns_name.as_str());
            let mut inner_env = env.clone().with_scope(namespace.frags());
            let rhs = &list[2];

            // Must be a define
            let Expression::List(items) = rhs else {
                return Err("Expected define form inside define/in-namespace".into());
            };

            if !matches!(items.first(), Some(Expression::Symbol(s)) if s == "define") {
                return Err("Expected define form inside define/in-namespace".into());
            }

            match items.get(1) {
                // (define (name args...) ...)
                Some(Expression::List(func_head)) => {
                    let Some(Expression::Symbol(inner_name)) = func_head.first() else {
                        return Err("Invalid inner define syntax".into());
                    };

                    let full_name = format!("{ns_name}::{inner_name}");
                    let full_sym = NamespaceItem::from(full_name.as_str());

                    // Rewrite define head with full name
                    let mut rewritten = items.clone();
                    if let Expression::List(head) = &mut rewritten[1] {
                        head[0] = Expression::Symbol(full_name.clone());
                    }

                    eval_define(&rewritten, &mut inner_env)?;

                    let Some(bound) = inner_env.get(&full_name) else {
                        return Err(format!("Inner define made no binding for {full_name}"));
                    };

                    env.insert(full_sym, bound.clone());
                    Ok(Expression::Symbol(full_name))
                }

                Some(Expression::Symbol(inner_name)) => {
                    let full_name = format!("{ns_name}::{inner_name}");
                    let full_sym = NamespaceItem::from(full_name.as_str());

                    let mut rewritten = items.clone();
                    rewritten[1] = Expression::Symbol(full_name.clone());

                    eval_define(&rewritten, &mut inner_env)?;

                    let Some(bound) = inner_env.get(&full_name) else {
                        return Err(format!("Inner define made no binding for {full_name}"));
                    };

                    env.insert(full_sym, bound.clone());
                    Ok(Expression::Symbol(full_name))
                }

                _ => Err("Invalid inner define syntax".into()),
            }
        }

        _ => Err("Invalid define/in-namespace syntax".into()),
    }
}

fn eval_symbol_namespace(list: &[Expression], env: &mut Environment) -> Result<Expression, String> {
    let expr = &list[1];

    let sym = match expr {
        Expression::Symbol(s) => Some(s.clone()),
        _ => match eval_expr(expr.clone(), env)? {
            Expression::Symbol(s) => Some(s),
            _ => None,
        },
    };

    Ok(sym
        .and_then(|s| env.get_namespace_str(&s))
        .map_or(Expression::Bool(false), Expression::Symbol))
}

fn eval_symbol_namespace_as_list(
    _list: &[Expression],
    env: &mut Environment,
) -> Result<Expression, String> {
    Ok(Expression::List(
        env.scopes()
            .iter()
            .map(|n| Expression::Symbol(n.to_string()))
            .collect(),
    ))
}

fn eval_quote(list: &[Expression], _env: &mut Environment) -> Result<Expression, String> {
    Ok(list[1].clone())
}

fn eval_require(list: &[Expression], env: &mut Environment) -> Result<Expression, String> {
    let [_, Expression::Symbol(sym)] = list else {
        return Err("`require` requires at least 1 symbol argument".into());
    };

    env.add_scope(Namespace::from_str(sym));
    Ok(Expression::Bool(true))
}

fn eval_deconst_fn(list: &[Expression], env: &mut Environment) -> Result<Expression, String> {
    let [_, Expression::Symbol(name)] = list else {
        return Err("`deconst-fn` requires a symbol".into());
    };

    match env.get(name) {
        Some(v) => {
            println!("{v}");
            Ok(Expression::Bool(true))
        }
        None => Err("Could not find symbol".into()),
    }
}

fn eval_file(list: &[Expression], env: &mut Environment) -> Result<Expression, String> {
    let [_, Expression::Symbol(path)] = list else {
        return Err("`eval-file` requires a symbol".into());
    };

    let contents = fs::read_to_string(path).unwrap_or_else(|_| "()".into());
    let parsed = crate::parser::parse(&contents).map_err(|e| {
        eprintln!("{e:?}");
        String::from("Parsing failed")
    })?;

    let mut result = Expression::Bool(true);
    for expr in parsed {
        result = eval(expr, env)?;
    }

    Ok(result)
}

fn eval_if(list: &[Expression], env: &mut Environment) -> Result<Expression, String> {
    let [_, cond, then_, else_] = list else {
        return Err("`if` requires three arguments".into());
    };

    match eval_expr(cond.clone(), env)? {
        Expression::Bool(true) => eval_expr(then_.clone(), env),
        Expression::Bool(false) => eval_expr(else_.clone(), env),
        _ => Err("Invalid condition in if expression".into()),
    }
}

fn eval_let(list: &[Expression], env: &mut Environment) -> Result<Expression, String> {
    let Expression::List(bindings) = &list[1] else {
        return Err("`let` bindings must be a list".into());
    };

    let mut local_env = env.clone();

    for binding in bindings {
        let Expression::List(pair) = binding else {
            return Err("Invalid `let` binding".into());
        };

        let [Expression::Symbol(name), value] = &pair[..] else {
            return Err("Invalid `let` binding".into());
        };

        let val = eval_expr(value.clone(), env)?;
        local_env.insert(NamespaceItem::from_str(name), val);
    }

    let mut result = Expression::Bool(false);
    for expr in &list[2..] {
        result = eval_expr(expr.clone(), &mut local_env)?;
    }

    Ok(result)
}

fn eval_lambda(list: &[Expression], env: &mut Environment) -> Result<Expression, String> {
    if list.len() < 3 {
        return Err("`lambda` requires parameters and a body".into());
    }

    let params = match &list[1] {
        Expression::List(p) => p.clone(),
        _ => return Err("`lambda` parameter list must be a list".into()),
    };

    for param in &params {
        if !matches!(param, Expression::Symbol(_)) {
            return Err("`lambda` parameters must be symbols".into());
        }
    }

    let body = list[2..].to_vec();

    Ok(Expression::Function(Procedure {
        params,
        body,
        env: env.clone(),
    }))
}
