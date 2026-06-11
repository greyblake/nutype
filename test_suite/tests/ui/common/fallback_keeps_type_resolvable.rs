use nutype::nutype;

// The attribute is broken, so expansion fails. Thanks to the fallback skeleton,
// the type `Name`, its constructors and `into_inner` still exist, so the usages
// below do NOT produce spurious "cannot find type/function `Name`" errors --
// only the attribute error surfaces.
#[nutype(validte)]
pub struct Name(String);

fn use_it(n: Name) -> String {
    n.into_inner()
}

fn construct() {
    // Both constructors are stubbed because we cannot know whether the fixed
    // attribute will end up validated (`try_new`) or not (`new`).
    let _ = Name::new("hello");
    let _ = Name::try_new("hello");
}

fn main() {
    let _ = use_it;
    let _ = construct;
}
