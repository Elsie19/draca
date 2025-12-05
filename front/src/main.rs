use interp::interp_sexpr::eval;
use lexpr::{Cons, Value, print::Options};

const SEXPRS: &str = r###"
(defenum! option
  (list (Some v) None)
  ; TODO: Add derives
  (:impl (list
     ('is-some
       (lambda (self)
         (match self
           ((Some _) true)
           (None     false))))
     ('is-none
       (lambda (self)
         (match self
           ((Some _) false)
           (None     true))))
     ('unwrap
       (lambda (self)
         (match self
           ((Some x) x)
           (None (panic! "Tried to unwrap None"))))))))
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
