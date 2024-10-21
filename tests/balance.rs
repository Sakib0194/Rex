extern crate rex_tui;
use chrono::{naive::NaiveDate, Duration};
use rex_tui::db::create_db;
use rex_tui::tx_handler::*;
use rex_tui::utility::*;
use rusqlite::{Connection, Result as sqlResult};
use std::collections::HashMap;
use std::fs;

fn create_test_db(file_name: &str) -> Connection {
    if let Ok(metadata) = fs::metadata(file_name) {
        if metadata.is_file() {
            fs::remove_file(file_name).expect("Failed to delete existing file");
        }
    }

    let mut conn = Connection::open(file_name).unwrap();
    create_db(&["test1".to_string(), "test 2".to_string()], &mut conn).unwrap();
    conn
}

#[test]
fn check_last_balances_1() {
    let file_name = "last_balances_1.sqlite";
    let conn = create_test_db(file_name);
    let data = get_last_balances(&conn);
    let expected_data = vec!["0".to_string(), "0".to_string()];
    conn.close().unwrap();

    fs::remove_file(file_name).unwrap();

    assert_eq!(data, expected_data);
}

#[test]
fn check_last_balances_2() {
    let file_name = "last_balances_2.sqlite";
    let mut conn = create_test_db(file_name);

    add_tx(
        "2022-07-19",
        "Testing transaction",
        "test1",
        "159.00",
        "Expense",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2022-07-19",
        "Testing transaction",
        "test 2",
        "159.19",
        "Income",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    let data = get_last_balances(&conn);
    let expected_data = vec!["-159".to_string(), "159.19".to_string()];

    delete_tx(1, &mut conn).unwrap();

    let data_2 = get_last_balances(&conn);
    let expected_data_2 = vec!["0".to_string(), "159.19".to_string()];

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, expected_data);
    assert_eq!(data_2, expected_data_2);
}

#[test]
fn check_last_balances_3() {
    let file_name = "last_balances_3.sqlite";
    let mut conn = create_test_db(file_name);

    add_tx(
        "2022-07-19",
        "Testing transaction",
        "test1 to test 2",
        "159.00",
        "Transfer",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2022-07-19",
        "Testing transaction",
        "test 2 to test1",
        "159.00",
        "Transfer",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    let data = get_last_balances(&conn);
    let expected_data = vec!["0".to_string(), "0".to_string()];

    delete_tx(1, &mut conn).unwrap();

    let data_2 = get_last_balances(&conn);
    let expected_data_2 = vec!["159".to_string(), "-159".to_string()];

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, expected_data);
    assert_eq!(data_2, expected_data_2);
}

#[test]
fn check_last_month_balance_1() {
    let file_name = "last_month_balance_1.sqlite";
    let conn = create_test_db(file_name);
    let tx_methods = get_all_tx_methods(&conn);

    let data = get_last_time_balance(6, 1, &tx_methods, &conn);
    let expected_data = HashMap::from([("test1".to_string(), 0.0), ("test 2".to_string(), 0.0)]);

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, expected_data);
}

#[test]
fn check_last_balance_id() {
    let file_name = "last_balance_id.sqlite";
    let conn = create_test_db(file_name);

    let data = get_last_balance_id(&conn);
    let expected_data: sqlResult<i32> = Ok(193);

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data, expected_data);
}

#[test]
fn check_last_month_balance_2() {
    let file_name = "last_month_balance_2.sqlite";
    let mut conn = create_test_db(file_name);
    let tx_methods = get_all_tx_methods(&conn);

    add_tx(
        "2022-07-19",
        "Testing transaction",
        "test1",
        "100.00",
        "Income",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2022-07-19",
        "Testing transaction",
        "test 2",
        "100.00",
        "Income",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2022-08-19",
        "Testing transaction",
        "test1",
        "100.00",
        "Income",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2022-09-19",
        "Testing transaction",
        "test1",
        "100.00",
        "Income",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2022-10-19",
        "Testing transaction",
        "test1",
        "100.00",
        "Income",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    let data_1 = get_last_time_balance(8, 0, &tx_methods, &conn);
    let expected_data_1 =
        HashMap::from([("test 2".to_string(), 100.0), ("test1".to_string(), 200.0)]);

    delete_tx(1, &mut conn).unwrap();
    delete_tx(2, &mut conn).unwrap();

    let data_2 = get_last_time_balance(10, 3, &tx_methods, &conn);
    let expected_data_2 =
        HashMap::from([("test 2".to_string(), 0.0), ("test1".to_string(), 300.0)]);

    add_tx(
        "2028-08-19",
        "Testing transaction",
        "test1",
        "100.00",
        "Income",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2025-09-19",
        "Testing transaction",
        "test1",
        "100.00",
        "Income",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    add_tx(
        "2025-10-19",
        "Testing transaction",
        "test1",
        "100.00",
        "Income",
        "Unknown",
        None,
        &mut conn,
    )
    .unwrap();

    let data_3 = get_last_time_balance(10, 4, &tx_methods, &conn);
    let expected_data_3 =
        HashMap::from([("test 2".to_string(), 0.0), ("test1".to_string(), 500.0)]);

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data_1, expected_data_1);
    assert_eq!(data_2, expected_data_2);
    assert_eq!(data_3, expected_data_3);
}

#[test]
#[ignore]
fn check_balance_all_day() {
    let file_name = "check_balance_all_day.sqlite";
    let mut conn = create_test_db(file_name);
    let tx_methods = get_all_tx_methods(&conn);

    let mut current_date = NaiveDate::parse_from_str("2022-01-01", "%Y-%m-%d").unwrap();
    let ending_date = NaiveDate::parse_from_str("2037-12-31", "%Y-%m-%d").unwrap();

    let mut total_days = 0;
    let mut total_amount = 0;

    let details = "Test Transaction";
    let amount = "1.0";
    let tx_method = "test1";
    let tx_type = "Income";

    loop {
        if current_date > ending_date {
            break;
        }
        add_tx(
            &current_date.to_string(),
            details,
            tx_method,
            amount,
            tx_type,
            "Unknown",
            None,
            &mut conn,
        )
        .unwrap();
        current_date += Duration::days(28);
        total_amount += 1;
        total_days += 1;
    }

    let data = get_last_balances(&conn);
    let expected = vec![total_amount.to_string(), "0".to_string()];
    assert_eq!(data, expected);

    let mut delete_id_num = total_days;

    loop {
        if delete_id_num == 0 {
            break;
        }
        delete_tx(delete_id_num, &mut conn).unwrap();
        delete_id_num -= 1;
    }

    let data_1 = get_last_balances(&conn);
    let data_2 = get_last_time_balance(12, 3, &tx_methods, &conn);

    let expected_data_1 = vec!["0".to_string(), "0".to_string()];
    let mut expected_data_2 = HashMap::new();
    for i in data_2.keys() {
        expected_data_2.insert(i.to_string(), 0.0);
    }

    conn.close().unwrap();
    fs::remove_file(file_name).unwrap();

    assert_eq!(data_1, expected_data_1);
    assert_eq!(data_2, expected_data_2);
}
