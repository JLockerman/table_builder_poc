#![allow(dead_code)]
use pgx::*;

use table_builder_macro::*;

pg_module_magic!();

pub mod framework;

table!{
    ExampleTable (
        foo: i32,
        bar: i64,
        baz: String,
        avg: f64,
        optional: Option<f32>,
    )
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use std::collections::HashMap;

    use pgx::*;

    use table_builder_macro::*;

    use crate::framework;

    table!{
        KeyValueTable (
            key: String,
            value: Option<i32>,
        )
    }

    #[pg_test]
    fn test_query() {
        Spi::connect(|client| {
            let expected = HashMap::from([
                ("31", Some(31)),
                ("2", Some(2)),
                ("", Some(0)),
                ("foo", Some(111)),
            ]);

            for (key, value) in expected.iter() {
                client.select("INSERT INTO KeyValueTable VALUES ($1, $2)",
                    None,
                    Some(vec![
                        (PgBuiltInOids::TEXTOID.oid(), key.into_datum()),
                        (PgBuiltInOids::INT4OID.oid(), value.into_datum()),
                    ])
                );
            }

            let values = query!(client
                from: KeyValueTable
                select: (key, value)
                where: "key <> 'foo'"
            );
            let mut count = 0;
            for (key, val) in values {
                assert_eq!(val, expected[&*key]);
                count += 1;
            }
            assert_eq!(count, expected.len() - 1);
            Ok(Some(()))
        });
    }
}

#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
