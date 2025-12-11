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
    let mut namespace = Namespace::<&str, fn(&str) -> String>::new("::");

    namespace.insert_at_module(["std", "fns", "hello"], hello);
    namespace.insert_in_module(["std", "fns"], "adios", bye);
    namespace.insert_in_module(["std", "math", "fns"], "add", format_add);

    let fns = namespace.find("fns");

    println!("There are {} instances of `fns` here", fns.len());

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

    for comp in fns {
        if let Some(path) = comp.path_from_root(&namespace) {
            println!("{}", path.as_absolute_path(PathRules::SepPreceedsRoot));
        }
    }

    for item in namespace.all_items() {
        println!("{:#?}", item.name().expect("Checked above"));
    }
}
