use interp::interp_sexpr::eval;
use lexpr::{Cons, Value, print::Options};

const SEXPRS: &str = r###"
(let (persons-name (std::io::read-line))
  (if-let (val persons-name)
    ;; Some case
    (let (person (make-name val))
      (println "Hello {}!" (name->name person))
    )

    ;; None case
    (println "No name given.")
  )
)
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
