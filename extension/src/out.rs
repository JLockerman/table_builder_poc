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
impl ExampleTable {
    pub fn to_values_vec(self) -> Vec<(pgx::PgOid, Option<pgx::pg_sys::Datum>)> {
        use pgx::IntoDatum;
        let Self {
            foo,
            bar,
            baz,
            avg,
            optional,
        } = self;
        vec![
            (
                pgx::PgOid::from(<i32 as pgx::IntoDatum>::type_oid()),
                foo.into_datum(),
            ),
            (
                pgx::PgOid::from(<i64 as pgx::IntoDatum>::type_oid()),
                bar.into_datum(),
            ),
            (
                pgx::PgOid::from(<String as pgx::IntoDatum>::type_oid()),
                baz.into_datum(),
            ),
            (
                pgx::PgOid::from(<f64 as pgx::IntoDatum>::type_oid()),
                avg.into_datum(),
            ),
            (
                pgx::PgOid::from(<Option<f32> as pgx::IntoDatum>::type_oid()),
                optional.into_datum(),
            ),
        ]
    }
}
pgx::extension_sql! {
    "CREATE TABLE ExampleTable (\n    foo integer NOT NULL,\n    bar bigint NOT NULL,\n    baz text NOT NULL,\n    avg double precision NOT NULL,\n    optional real\n);\n",
    name = "__CREATE_TABLE_ExampleTable",
}

fn test() {
    {
        use pgx::IntoDatum;
        let value: KeyValueTable = KeyValueTable {
            key: "111".to_string(),
            value: Some(6),
        };
        let args = value.to_values_vec();
        client.update(
            "INSERT INTO KeyValueTable VALUES ($1, $2)",
            None,
            Some(args),
        )
    }
    {
        use pgx::IntoDatum;
        let vals = expected.iter().map(|(key, value)| KeyValueTable {
            key: key.to_string(),
            value: value.clone(),
        });
        for value in vals {
            let value: KeyValueTable = value;
            let args = value.to_values_vec();
            client.update(
                "INSERT INTO KeyValueTable VALUES ($1, $2)",
                None,
                Some(args),
            );
        }
    }
    client
        .select(
            &format!(
                "SELECT key::{key}, value::{value} FROM KeyValueTable WHERE key <> \'foo\'",
                key = <_KeyValueTable_table_mod::key as framework::PgTyped>::SQL_TYPE,
                value = <_KeyValueTable_table_mod::value as framework::PgTyped>::SQL_TYPE,
            ),
            None,
            None,
        )
        .map(|__tuple| {
            let key: _KeyValueTable_table_mod::_optional_key =
                __tuple.by_ordinal(1usize).unwrap().value();
            let key: _KeyValueTable_table_mod::key = <_ as framework::UnwrapTo<_>>::unwrap_to(key);
            let value: _KeyValueTable_table_mod::_optional_value =
                __tuple.by_ordinal(2usize).unwrap().value();
            let value: _KeyValueTable_table_mod::value =
                <_ as framework::UnwrapTo<_>>::unwrap_to(value);
            (key, value)
        });
    client
        .select(
            &format!(
                "SELECT key::{key}, value::{value} FROM KeyValueTable WHERE key <> \'foo\'",
                key = <_KeyValueTable_table_mod::key as framework::PgTyped>::SQL_TYPE,
                value = <_KeyValueTable_table_mod::value as framework::PgTyped>::SQL_TYPE,
            ),
            None,
            None,
        )
        .map(|__tuple| {
            let key: _KeyValueTable_table_mod::_optional_key =
                __tuple.by_ordinal(1usize).unwrap().value();
            let key: _KeyValueTable_table_mod::key = <_ as framework::UnwrapTo<_>>::unwrap_to(key);
            let value: _KeyValueTable_table_mod::_optional_value =
                __tuple.by_ordinal(2usize).unwrap().value();
            let value: _KeyValueTable_table_mod::value =
                <_ as framework::UnwrapTo<_>>::unwrap_to(value);
            (key, value)
        });
}
