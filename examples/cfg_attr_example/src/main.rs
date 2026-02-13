use nutype::nutype;

// 1. Conditional serde behind a feature flag
//    Serialize and Deserialize are only derived when the "serde" feature is enabled.
//    Note: nutype/serde must be enabled at compile time so the macro accepts these traits,
//    but the actual derive is gated by cfg_attr.
#[nutype(
    sanitize(trim, lowercase),
    validate(not_empty, len_char_max = 100),
    derive(Debug, Clone, PartialEq, AsRef),
    cfg_attr(feature = "serde", derive(Serialize, Deserialize))
)]
pub struct Email(String);

// 2. Conditional Default for tests
//    Default is only derived under `cfg(test)`, but the default value is always specified.
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 65535),
    default = 8080,
    derive(Debug, Clone, Copy, PartialEq, Into),
    cfg_attr(test, derive(Default))
)]
pub struct Port(u16);

// 3. Complex predicate
//    Clone and Display are only derived when both `test` and `debug_assertions` are active.
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 50),
    derive(Debug, PartialEq, AsRef),
    cfg_attr(all(test, debug_assertions), derive(Clone, Display))
)]
pub struct Label(String);

// 4. Multiple cfg_attr entries
//    Each cfg_attr line is independent and can gate different traits behind different predicates.
#[nutype(
    validate(not_empty),
    derive(Debug),
    cfg_attr(test, derive(Clone)),
    cfg_attr(feature = "serde", derive(Serialize, Deserialize))
)]
pub struct Tag(String);

fn main() {
    // Exercise Email
    let email = Email::try_new("  Alice@Example.COM  ").unwrap();
    assert_eq!(email.as_ref(), "alice@example.com");
    println!("Email: {email:?}");

    // Exercise Email with serde (only when the feature is enabled)
    #[cfg(feature = "serde")]
    {
        let json = serde_json::to_string(&email).unwrap();
        println!("Email as JSON: {json}");

        let parsed: Email = serde_json::from_str(&json).unwrap();
        assert_eq!(email, parsed);
        println!("Round-tripped email: {parsed:?}");
    }

    // Exercise Port
    let port = Port::try_new(3000).unwrap();
    let port_val: u16 = port.into();
    assert_eq!(port_val, 3000u16);
    println!("Port: {port:?}");

    // Exercise Label
    let label = Label::try_new("  Rust  ").unwrap();
    assert_eq!(label.as_ref(), "Rust");
    println!("Label: {label:?}");

    // Exercise Tag
    let tag = Tag::try_new("nutype").unwrap();
    println!("Tag: {tag:?}");

    // Exercise Tag with serde (only when the feature is enabled)
    #[cfg(feature = "serde")]
    {
        let json = serde_json::to_string(&tag).unwrap();
        println!("Tag as JSON: {json}");
    }

    println!("All cfg_attr examples passed!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_default() {
        // Default is conditionally derived under cfg(test), so this works in tests.
        let port = Port::default();
        let port_val: u16 = port.into();
        assert_eq!(port_val, 8080u16);
    }

    #[test]
    fn test_tag_clone() {
        // Clone is conditionally derived under cfg(test).
        let tag = Tag::try_new("example").unwrap();
        let tag2 = tag.clone();
        assert_eq!(format!("{tag:?}"), format!("{tag2:?}"));
    }
}
