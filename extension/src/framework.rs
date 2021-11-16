

// marker trait that lets us know a struct was created with the `table!` macro
// and therefore it is safe to use in queries.
pub unsafe trait PgTable {}

// trait that lets us know the equivalent SQL type for a rust type
// this would likely be part of pgx in a real versin
pub unsafe trait PgTyped {
    const SQL_TYPE: &'static str;
}

macro_rules! pg_typed {
    ($($t: ty => $sql: literal),* $(,)?) => {
        $(
            unsafe impl PgTyped for $t {
                const SQL_TYPE: &'static str = $sql;
            }
        )*
    };
}

pg_typed!(
    i16    => "smallint",
    i32    => "integer",
    i64    => "bigint",
    f32    => "real",
    f64    => "double precision",
    String => "text",
    bool   => "boolean",
);

unsafe impl<T: PgTyped> PgTyped for Option<T> {
    const SQL_TYPE: &'static str = <T as PgTyped>::SQL_TYPE;
}

pub trait UnwrapTo<T> {
    #[track_caller]
    fn unwrap_to(self) -> T;
}

impl<T> UnwrapTo<T> for Option<T> {
    #[track_caller]
    fn unwrap_to(self) -> T {
        self.expect("unexpected NULL value")
    }
}

impl<T> UnwrapTo<Option<T>> for Option<T> {
    #[track_caller]
    fn unwrap_to(self) -> Option<T> {
        self
    }
}