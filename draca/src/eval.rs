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
        | Expression::Quoted(_)
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
    let first = &list[0];

    if let Expression::Symbol(s) = first {
        match s.as_str() {
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
            _ => {
                if let Some(exp) = env.get(s) {
                    match exp {
                        Expression::Func(f) => {
                            let function = *f;
                            let args: Result<Vec<Expression>, String> = list[1..]
                                .iter()
                                .map(|x| eval_expr(x.clone(), env))
                                .collect();
                            function(&args?)
                        }
                        Expression::Function(proc) => {
                            let env_clone = &mut env.clone();

                            let args: Result<Vec<Expression>, String> = list[1..]
                                .iter()
                                .map(|x| eval_expr(x.clone(), env_clone))
                                .collect();

                            // Create a new execution environment for the function
                            let mut local_env = proc.env.clone();

                            // Insert the function name into the new environment
                            local_env.insert(NamespaceItem::from_str(s), exp.clone());

                            for (param, arg) in proc.params.iter().zip(args?) {
                                if let Expression::Symbol(param_name) = param {
                                    local_env.insert(NamespaceItem::from_str(param_name), arg);
                                } else {
                                    return Err("Invalid parameter name".into());
                                }
                            }

                            let mut result = Expression::Bool(false);

                            for exp in proc.body.clone() {
                                result = eval_expr(exp.clone(), &mut local_env)?;
                            }

                            Ok(result)
                        }
                        _ => Err(format!("Undefined function: {s}")),
                    }
                } else {
                    Err(format!("Undefined function: {s}"))
                }
            }
        }
    } else {
        eprintln!("{:?}", first);
        Err("Expected a symbol".into())
    }
}

fn eval_define(list: &[Expression], env: &mut Environment) -> Result<Expression, String> {
    if list.len() < 3 {
        return Err("`define` requires at least two arguments".into());
    }

    if let Expression::List(func) = list.get(1).unwrap() {
        if let Some(Expression::Symbol(func_name)) = func.first() {
            let params = func[1..].to_vec();
            let body = list.get(2..).ok_or("Invalid define syntax")?.to_vec();

            let proc = Procedure {
                params,
                body,
                env: env.clone(),
            };

            let function = Expression::Function(proc);

            env.insert(NamespaceItem::from_str(func_name), function);
            Ok(Expression::Symbol(func_name.clone()))
        } else {
            Err("Invalid define syntax".into())
        }
    } else if let Expression::Symbol(var_name) = list.get(1).unwrap() {
        let value = eval_expr(list[2].clone(), env)?;
        env.insert(NamespaceItem::from_str(var_name), value.clone());
        Ok(Expression::Symbol(var_name.clone()))
    } else {
        Err("Invalid define syntax".into())
    }
}

fn eval_define_namespace(list: &[Expression], env: &mut Environment) -> Result<Expression, String> {
    if list.len() < 3 {
        return Err("`define/in-namespace` requires at least two arguments".into());
    }

    if let Expression::List(func) = list.get(1).unwrap() {
        if let Some(Expression::Symbol(func_name)) = func.first() {
            let params = func[1..].to_vec();
            let body = list.get(2..).ok_or("Invalid define syntax")?.to_vec();

            let proc = Procedure {
                params,
                body,
                env: env.clone(),
            };

            let function = Expression::Function(proc);

            env.insert(NamespaceItem::from_str(func_name), function);
            Ok(Expression::Symbol(func_name.clone()))
        } else {
            Err("Invalid define syntax".into())
        }
    } else if let Expression::Symbol(var_name) = list.get(1).unwrap() {
        let namespace = NamespaceItem::from(var_name.as_str());
        let mut inner_env = env.clone().with_scope(namespace.frags());

        let rhs = list[2].clone();

        // If the right-hand side begins with `define`, treat it as a definition
        // TODO: Figure out wtf to do for things besides define.
        if let Expression::List(items) = &rhs
            && let Some(Expression::Symbol(s)) = items.first()
            && s == "define"
            && let Expression::List(func_head) = &items[1]
        {
            let mut new_head = func_head.clone();

            new_head[0] = Expression::Symbol(var_name.clone());

            let mut rewritten = vec![
                Expression::Symbol("define".into()),
                Expression::List(new_head),
            ];
            rewritten.extend_from_slice(&items[2..]);

            let _ = eval_define(&rewritten, &mut inner_env)?;

            if let Some(bound) = inner_env.get(var_name) {
                env.insert(namespace, bound.clone());
                return Ok(Expression::Symbol(var_name.clone()));
            }

            return Err(format!("Inner define made no binding for {var_name}"));
        }

        let value = eval_expr(rhs, &mut inner_env)?;
        env.insert(namespace, value.clone());
        Ok(Expression::Symbol(var_name.clone()))
    } else {
        Err("Invalid define syntax".into())
    }
}

fn eval_symbol_namespace(list: &[Expression], env: &mut Environment) -> Result<Expression, String> {
    let expr = &list[1];

    if let Expression::Symbol(sym) = expr {
        return Ok(env
            .get_namespace_str(sym)
            .map_or(Expression::Bool(false), Expression::Symbol));
    }

    let evaled = eval_expr(expr.clone(), env)?;

    match evaled {
        Expression::Symbol(sym) => Ok(env
            .get_namespace_str(&sym)
            .map_or(Expression::Bool(false), Expression::Symbol)),
        _ => Ok(Expression::Bool(false)),
    }
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
    if list.len() != 2 {
        return Err("`require` requires at least 1 argument".into());
    }

    match list {
        [_, path] => match path {
            Expression::Symbol(sym) => {
                env.add_scope(Namespace::from_str(sym));
                Ok(Expression::Bool(true))
            }
            _ => Err("Expected symbol".into()),
        },
        _ => unreachable!("We checked above"),
    }
}

fn eval_deconst_fn(list: &[Expression], env: &mut Environment) -> Result<Expression, String> {
    if list.len() != 2 {
        return Err("`deconst-fn` requires at least 1 argument".into());
    }

    match list {
        [_, fn_] => {
            if let Expression::Symbol(fn_) = fn_ {
                match env.get(fn_) {
                    Some(got) => {
                        println!("{}", got);
                        Ok(Expression::Bool(true))
                    }
                    None => Err("Could not find symbol".into()),
                }
            } else {
                Err("`deconst-fn` requires a symbol".into())
            }
        }
        _ => unreachable!("We checked above"),
    }
}

fn eval_file(list: &[Expression], env: &mut Environment) -> Result<Expression, String> {
    if list.len() != 2 {
        return Err("`eval-file` requires at least 1 argument".into());
    }

    match list {
        [_, path] => {
            if let Expression::Symbol(path) = path {
                let contents = fs::read_to_string(path).unwrap_or(String::from("()"));
                let parsed_list = match crate::parser::parse(&contents) {
                    Ok(val) => val,
                    Err(e) => {
                        eprintln!("{:?}", e);
                        std::process::exit(1);
                    }
                };

                let mut retxpr = Ok(Expression::Bool(true));

                for expr in parsed_list {
                    retxpr = eval(expr, env);
                }

                retxpr
            } else {
                Err("eval-files requires a symbol".into())
            }
        }
        _ => unreachable!("We checked above"),
    }
}

fn eval_if(list: &[Expression], env: &mut Environment) -> Result<Expression, String> {
    if list.len() < 4 {
        return Err("`if` requires at least three arguments".into());
    }

    match list {
        [_, condition, then, else_] => match eval_expr(condition.clone(), env)? {
            Expression::Bool(true) => eval_expr(then.clone(), env),
            Expression::Bool(false) => eval_expr(else_.clone(), env),
            _ => Err("Invalid condition in if expression".into()),
        },
        _ => unreachable!("Checked above"),
    }
}

fn eval_let(list: &[Expression], env: &mut Environment) -> Result<Expression, String> {
    if list.len() < 3 {
        return Err("`let` requires bindings and a body".into());
    }

    let bindings = match &list[1] {
        Expression::List(b) => b,
        _ => return Err("`let` bindings must be a list".into()),
    };

    let mut local_env = env.clone();

    for binding in bindings {
        match binding {
            Expression::List(pair) if pair.len() == 2 => {
                let name = match &pair[0] {
                    Expression::Symbol(s) => s,
                    _ => return Err("`let` binding name must be a symbol".into()),
                };

                let value = eval_expr(pair[1].clone(), env)?;
                local_env.insert(NamespaceItem::from_str(name), value);
            }
            _ => return Err("Invalid `let` binding".into()),
        }
    }

    let mut result = Expression::Bool(false);

    for expr in &list[2..] {
        result = eval_expr(expr.clone(), &mut local_env)?;
    }

    Ok(result)
}
