use lexpr::Value;

use crate::env::Env;

pub fn eval(val: &Value) {
    match val {
        Value::Nil => todo!(),
        Value::Null => todo!(),
        Value::Bool(_) => todo!(),
        Value::Number(number) => todo!(),
        Value::Char(_) => todo!(),
        Value::String(_) => todo!(),
        Value::Symbol(_) => todo!(),
        Value::Keyword(_) => todo!(),
        Value::Bytes(items) => todo!(),
        Value::Cons(cons) => {
            if let Value::Symbol(r) = cons.car()
                && **r == *"require"
            {
                let components = cons.iter().skip(1).collect::<Vec<_>>();
                match components.len() {
                    1 => println!("Importing {:?}", components),
                    mult => {
                        println!("{mult}");
                        let mut first = true;
                        for part in components {
                            if first {
                                println!("Importing from {:?}", part.car());
                                first = false;
                            } else {
                                println!("This function: {:?}", part.car());
                            }
                        }
                    }
                }
            }
        }
        Value::Vector(values) => todo!(),
    }
}
