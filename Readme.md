# Table Builder PoC #

_**Warning:** This repo exists for demo-purposes only, and should not be used for anything._

Most table accesses done from a Postgres extension are to either the builtin catalog tables, or to catalog-like tables that the extension defines itself. Since these tables have known, effectively non-changing layouts, we should be able to build a safe interface to access and manipulate the data in these tables from rust.

This repo contains a PoC for this feature using [`pgx`](https://github.com/zombodb/pgx) and the SPI. It provides macros for defining tables, and initializing tables during extension creation, and a safe interface to SELECT-from and INSERT-into tables that rust understands.

## Interface Walkthrough ##

### Basic Table Creation ###

```rust
table! {
    Example (
        foo: i32,
        bar: Option<String>,
    )
}
```

Tables are created using the `table!{}` macro. Which creates a struct representing the contents of row in the table, some trait glue-code to make the query side work, and generates the SQL to create the table.

### Table Initialization ###

```rust
table! {
    Example (
        foo: i32,
        bar: Option<String>,
    )
    insert: {
        (0..3).map(|i| (i, i.to_string()))
    }
}
```

Tables can be initialized with default data via an iterator that yields tuples. This will do the equivalent of

```SQL
INSERT INTO Example
    SELECT * FROM ((0..3).map(|i| (i, i.to_string())))
```

### Queries ###

```rust
Spi::connect(|client|
    // INSERT values into a known table
    query!(client
        insert into: Example
        values: (-10..0).map(|i| (i, "".to_string()))
    );

    // SELECT data from a table
    let positive_values = query!(client
        from: Example
        select: (bar, foo)
    ).filter(|(_, foo)| foo > 0);

    let mut foo = 0;
    let mut bar = String::new();
    // note that rust already knows what values to expect out
    for (b, f) in positive_values {
        foo += f;
        bar.push_str(&*b);
    }

    // single-value INSERTion is also supported
    query!(client
        insert into: Example
        value: Example {foo, bar}
    );
)
```

The `query!()` macro provides a safe interface to created tables.
`insert into` inserts value(s) of the struct type into the table, while `from` returns an iterator of tuples selected from the table.

Safety is provided at two levels:

1. The rust code knows what types to expect from the SQL, so there's no way to get the types wrong.
2. The generated SQL contains casts asserting that the SQL values are of the correct type; even if the table changes out from under the rust code, this will just result in an SQL error, not data corruption or a segfault.