use core::fmt::Display;
use nutype::nutype;

#[nutype(derive(IntoIterator))]
struct GenericNames<'a, T: Display>(Vec<&'a T>);

#[nutype(derive(IntoIterator))]
struct StringNames(Vec<String>);

fn main() {
    let alice = "Alice".to_string();
    let bob = "Bob".to_string();

    let string_names = StringNames::new(vec![alice, bob]);

    // Test iterator over references
    {
        let mut ref_iter = (&string_names).into_iter();
        assert_eq!(ref_iter.next(), Some(&"Alice".to_string()));
        assert_eq!(ref_iter.next(), Some(&"Bob".to_string()));
        assert_eq!(ref_iter.next(), None);
    }

    // Test iterator over owned values
    {
        let mut owned_iter = string_names.into_iter();
        assert_eq!(owned_iter.next(), Some("Alice".to_string()));
        assert_eq!(owned_iter.next(), Some("Bob".to_string()));
        assert_eq!(owned_iter.next(), None);
    }
}
