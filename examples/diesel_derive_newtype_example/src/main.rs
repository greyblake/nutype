use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use nutype::nutype;

// Nutypes
#[nutype(derive(Debug, DieselNewType))]
pub struct FloatValue(f32);

#[nutype(derive(Debug, DieselNewType))]
pub struct IntegerValue(i32);

#[nutype(derive(Debug, DieselNewType))]
pub struct StringValue(String);

// Schema.rs
table! {
    some_object {
        id -> Integer,
        float_value -> Float,
        integer_value -> Integer,
        string_value -> Text,
    }
}

// Struct corresponding to schema
#[derive(Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = some_object)]
pub struct SomeObject {
    pub id: i32,
    pub float_value: FloatValue,
    pub integer_value: IntegerValue,
    pub string_value: StringValue,
}

fn prepare_table() -> SqliteConnection {
    let mut conn = SqliteConnection::establish(":memory:")
        .expect("Could not establish database connection in-memory");
    let setup = sql::<diesel::sql_types::Bool>(
        "CREATE TABLE IF NOT EXISTS some_object (
            id INTEGER PRIMARY KEY,
            float_value FLOAT,
            integer_value INTEGER,
            string_value TEXT
        )",
    );
    setup
        .execute(&mut conn)
        .expect("Could not create database table");
    conn
}

fn main() {
    let mut conn = prepare_table();
    let obj = SomeObject {
        id: 1,
        float_value: FloatValue::new(12.345),
        integer_value: IntegerValue::new(123),
        string_value: StringValue::new("some text ".to_string()),
    };

    diesel::insert_into(some_object::table)
        .values(&obj)
        .execute(&mut conn)
        .expect("Could not insert struct into database table");

    let _inserted_obj = some_object::table
        .select(SomeObject::as_select())
        .first(&mut conn)
        .expect("Could not get struct from database table");
}
