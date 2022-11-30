pub mod traits {
    pub fn should_implement_hash<T: std::hash::Hash>() {}
    pub fn should_implement_debug<T: std::fmt::Debug>() {}
    pub fn should_implement_from<T: std::convert::From<Inner>, Inner>() {}
    pub fn should_implement_try_from<T: std::convert::TryFrom<Inner>, Inner>() {}
    pub fn should_implement_from_str<T: std::str::FromStr>() {}
}
