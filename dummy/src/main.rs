use nutype::nutype;

#[nutype(
    derive(Debug, PartialEq, Deref, AsRef),
    sanitize(with = |mut guests: Vec<String>| { guests.sort(); guests }),
    validate(predicate = |guests| !guests.is_empty() ),
)]
pub struct GuestList(Vec<String>);

fn main() {
    // Empty list is not allowed
    assert_eq!(
        GuestList::new(vec![]),
        Err(GuestListError::PredicateViolated)
    );

    // Create the list of our guests
    let guest_list = GuestList::new(vec![
        "Seneca".to_string(),
        "Marcus Aurelius".to_string(),
        "Socrates".to_string(),
        "Epictetus".to_string(),
    ]).unwrap();

    // The list is sorted (thanks to sanitize)
    assert_eq!(
        guest_list.as_ref(),
        &[
            "Epictetus".to_string(),
            "Marcus Aurelius".to_string(),
            "Seneca".to_string(),
            "Socrates".to_string(),
        ]
    );

    // Since GuestList derives Deref, we can use methods from `Vec<T>`
    // due to deref coercion (if it's a good idea or not, it's left up to you to decide!).
    assert_eq!(guest_list.len(), 4);

    for guest in guest_list.iter() {
        println!("{guest}");
    }
}
