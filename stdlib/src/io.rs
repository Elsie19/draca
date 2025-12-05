use lexpr::Value;

pub fn puts(ch: char) -> Value {
    print!("{ch}");
    Value::Char(ch)
}
