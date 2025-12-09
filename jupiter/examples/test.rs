use jupiter::Namespace;

fn hello(s: &str) -> &str {
    s
}

fn bye(s: &str) -> &str {
    s
}

fn main() {
    let mut namespace = Namespace::<&str, fn(&str) -> &str>::new();

    namespace.insert_at_module(["std", "fns", "hello"], hello);
    namespace.insert_with_name(["std", "fns"], "adios", bye);
}
