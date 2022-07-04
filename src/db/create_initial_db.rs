use rusqlite::{Connection, Result};

/// If the local database is not found, this is executed to create the default
/// database with a set of provided Transaction Methods.
pub fn create_db(tx_methods: Vec<String>) -> Result<()> {
    let months = vec![
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let years = vec!["2022", "2023", "2024", "2025"];

    let path = "data.sqlite";
    let conn = Connection::open(path)?;

    conn.execute(
        "CREATE TABLE tx_all (
        date TEXT,
        details TEXT,
        tx_method TEXT,
        amount TEXT,
        tx_type TEXT,
        id_num INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT
    );",
        [],
    )?;

    let mut query = format!("CREATE TABLE changes_all (
        date TEXT,
        id_num INTEGER NOT NULL PRIMARY KEY,");
    for i in &tx_methods {
        query.push_str(&format!("\n{i} TEXT DEFAULT 0.00,"))
    }
    query.push_str("\nCONSTRAINT changes_all_FK FOREIGN KEY (id_num) REFERENCES tx_all(id_num) ON DELETE CASCADE
);");

    conn.execute(
        &query,
        [],
    )?;

    let mut query = format!("CREATE TABLE balance_all (
        id_num INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT");
    for i in &tx_methods {
        query.push_str(&format!(",\n{i} TEXT DEFAULT 0.00"))
    }
    query.push_str(");");

    conn.execute(
        &query,
        [],
    )?;

    conn.execute(
        "CREATE UNIQUE INDEX all_tx_date_IDX ON tx_all (id_num);",
        [],
    )
    .unwrap();

    conn.execute(
        "CREATE UNIQUE INDEX changes_all_date_IDX ON changes_all (id_num);",
        [],
    )
    .unwrap();

    conn.execute(
        "CREATE UNIQUE INDEX balance_all_id_num_IDX ON balance_all (id_num);",
        [],
    )
    .unwrap();

    let mut q_marks = vec![];
    for _i in &tx_methods {
        q_marks.push("0.0")
    }

    let mut query = format!("INSERT INTO balance_all ({:?}) VALUES ({:?})", tx_methods, q_marks);
    query = query.replace("[", "").replace("]", "").replace(r#"""#, "");

    for _i in years {
        for _a in 0..months.len() {
            conn.execute(&query,
        [])?;
        }
    }
    conn.execute(
        &query,
        [],
    )?;

    Ok(())
}
