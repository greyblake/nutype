pub mod traits {
    pub fn should_implement_hash<T: std::hash::Hash>() {}
    pub fn should_implement_debug<T: std::fmt::Debug>() {}
    pub fn should_implement_from<T: std::convert::From<Inner>, Inner>() {}
    pub fn should_implement_try_from<T: std::convert::TryFrom<Inner>, Inner>() {}
    pub fn should_implement_from_str<T: std::str::FromStr>() {}
    pub fn should_implement_borrow<T: std::borrow::Borrow<Borrowed>, Borrowed: ?Sized>() {}
    pub fn should_implement_clone<T: Clone>() {}
    pub fn should_implement_copy<T: Copy>() {}
    pub fn should_implement_eq<T: Eq>() {}
}

#[macro_export]
macro_rules! prepare_database_table_for_type {
    ($table:ident, $value_type:expr) => {{
        let mut conn = SqliteConnection::establish(":memory:")
            .expect("Could not establish database connection in-memory");
        let setup = sql::<diesel::sql_types::Bool>(
            format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    id INTEGER PRIMARY KEY,
                    value {}
                )",
                stringify!($table),
                $value_type,
            )
            .as_str(),
        );
        setup
            .execute(&mut conn)
            .expect("Could not create database table");
        conn
    }};
}
