use pgx::*;

use table_builder_macro::*;

pg_module_magic!();

table!{
    TestTable (
        foo: i32,
        bar: i64,
        baz: String,
        avg: f64,
    )
}

#[pg_extern]
fn hello_table_builder() -> &'static str {
    "Hello, table_builder"
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgx::*;

    #[pg_test]
    fn test_hello_table_builder() {
        assert_eq!("Hello, table_builder", crate::hello_table_builder());
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
