pub mod traits {
    pub fn should_implement_hash<T: std::hash::Hash>() {}
    pub fn should_implement_debug<T: std::fmt::Debug>() {}
    pub fn should_implement_from<T: std::convert::From<Inner>, Inner>() {}
    pub fn should_implement_try_from<T: std::convert::TryFrom<Inner>, Inner>() {}
    pub fn should_implement_from_str<T: std::str::FromStr>() {}
    pub fn should_implement_borrow<T: std::borrow::Borrow<Borrowed>, Borrowed: ?Sized>() {}
    pub fn should_implement_clone<T: Clone>() {}
    pub fn should_implement_copy<T: Copy>() {}
}
