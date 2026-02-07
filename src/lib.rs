/// Greet a person by name, or greet the world if no name is provided
pub fn greet(name: Option<&str>) -> String {
    match name {
        Some(n) => format!("Hello, {}!", n),
        None => String::from("Hello, world!"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet_with_name() {
        assert_eq!(greet(Some("Alice")), "Hello, Alice!");
    }

    #[test]
    fn test_greet_without_name() {
        assert_eq!(greet(None), "Hello, world!");
    }
}
