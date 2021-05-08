

fn main() {
    let s = String::from("book");

    println!("Hello, world!ss {}", pluralize(s.clone()));
}

fn pluralize(s: String) -> String {
    s + "s"
}