use interp::interp_sexpr::eval;
use lexpr::{Cons, Value, print::Options};

const SEXPRS: &str = r###"
(defun option/as-list (self)
  (if (option? self)
    (match self
      [(Some x) '(x)]
      [(None)   '()])
    (panic! "option/as-list: expected option, got...")))
"###
.trim_ascii();

fn main() {
    let v = lexpr::from_str(SEXPRS).unwrap();
    println!("{:#?}", v);
    println!(
        "{}",
        lexpr::print::to_string_custom(&v, Options::elisp()).unwrap()
    );
}
