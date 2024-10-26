pub mod traits {
    pub fn should_implement_hash<T: core::hash::Hash>() {}
    pub fn should_implement_debug<T: core::fmt::Debug>() {}
    pub fn should_implement_from<T: core::convert::From<Inner>, Inner>() {}
    pub fn should_implement_try_from<T: core::convert::TryFrom<Inner>, Inner>() {}
    pub fn should_implement_from_str<T: core::str::FromStr>() {}
    pub fn should_implement_borrow<T: core::borrow::Borrow<Borrowed>, Borrowed: ?Sized>() {}
    pub fn should_implement_clone<T: Clone>() {}
    pub fn should_implement_copy<T: Copy>() {}
    pub fn should_implement_eq<T: Eq>() {}
}
