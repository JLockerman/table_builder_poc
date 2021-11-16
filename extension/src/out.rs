// demonstrates the desugaring provided by these macros
#![allow(unused)]
use crate::framework;
use pgx::*;

struct ExampleTable {
    foo: i32,
    bar: i64,
    baz: String,
    avg: f64,
    optional: Option<f32>,
}
unsafe impl framework::PgTable for ExampleTable {}
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
mod _ExampleTable_table_mod {
    pub type foo = i32;
    pub type _optional_foo = Option<i32>;
    pub type bar = i64;
    pub type _optional_bar = Option<i64>;
    pub type baz = String;
    pub type _optional_baz = Option<String>;
    pub type avg = f64;
    pub type _optional_avg = Option<f64>;
    pub type optional = Option<f32>;
    pub type _optional_optional = Option<f32>;
}
pgx::extension_sql! {
    "CREATE TABLE ExampleTable (\n    foo integer NOT NULL,\n    bar bigint NOT NULL,\n    baz text NOT NULL,\n    avg double precision NOT NULL,\n    optional real\n);\n",
    name = "__CREATE_TABLE_ExampleTable_2",
}

fn test() {
    Spi::connect(|client| {
        client
            .select(
                &format!(
                    "SELECT key::{key}, value::{value} FROM KeyValueTable",
                    key = <_KeyValueTable_table_mod::key as framework::PgTyped>::SQL_TYPE,
                    value = <_KeyValueTable_table_mod::value as framework::PgTyped>::SQL_TYPE,
                ),
                None,
                None,
            )
            .map(|__tuple| {
                let key: _KeyValueTable_table_mod::_optional_key =
                    __tuple.by_ordinal(1usize).unwrap().value();
                let key: _KeyValueTable_table_mod::key =
                    <_ as framework::UnwrapTo<_>>::unwrap_to(key);
                let value: _KeyValueTable_table_mod::_optional_value =
                    __tuple.by_ordinal(2usize).unwrap().value();
                let value: _KeyValueTable_table_mod::value =
                    <_ as framework::UnwrapTo<_>>::unwrap_to(value);
                (key, value)
            });
        Ok(Some(()))
    });
}
struct KeyValueTable {
    key: String,
    value: Option<i32>,
}
unsafe impl framework::PgTable for KeyValueTable {}
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
mod _KeyValueTable_table_mod {
    pub type key = String;
    pub type _optional_key = Option<String>;
    pub type value = Option<i32>;
    pub type _optional_value = Option<i32>;
}
pgx::extension_sql! {
    "CREATE TABLE KeyValueTable (\n    key text NOT NULL,\n    value integer\n);\n",
    name = "__CREATE_TABLE_KeyValueTable_2",
}
