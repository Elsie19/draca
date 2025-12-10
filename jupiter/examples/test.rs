use jupiter::{Namespace, PathRules};

fn hello(s: &str) -> String {
    format!("Hello {s}")
}

fn bye(s: &str) -> String {
    format!("Bye {s}")
}

fn format_add(s: &str) -> String {
    format!("{s} + {s}")
}

fn main() {
    let mut namespace = Namespace::<&str, fn(&str) -> String>::new();

    namespace.insert_at_module(["std", "fns", "hello"], hello);
    namespace.insert_with_name(["std", "fns"], "adios", bye);
    namespace.insert_with_name(["std", "math", "fns"], "add", format_add);

    println!(
        "There are {} instances of `fns` here",
        namespace.find(&"fns").len(),
    );

    println!(
        "{}",
        match namespace.get_item(["std", "fns", "hello"]) {
            Some(v) => v("human"),
            None => String::from("nope"),
        }
    );

    println!(
        "{}",
        match namespace.get_item(["std", "fns", "adios"]) {
            Some(v) => v("human"),
            None => String::from("nope"),
        }
    );

    for comp in namespace.find(&"fns") {
        if let Some(path) = comp.path_from_root(&namespace) {
            println!(
                "{}",
                path.as_absolute_path("::", PathRules::SepPreceedsRoot)
            );
        }
    }
}
