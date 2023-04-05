use crate::utility::{get_all_tx_methods, get_last_balance_id, get_last_balances};
use rusqlite::{Connection, Result as sqlResult};

/// Updates the absolute final balance, balance data and deletes the selected transaction.
/// Foreign key cascade takes care of the Changes data in the database.
pub fn delete_tx(id_num: usize, path: &str) -> sqlResult<()> {
    let mut conn = Connection::open(path)?;
    let sp = conn.savepoint()?;

    let tx_methods = get_all_tx_methods(&sp);
    let last_balance = get_last_balances(&sp, &tx_methods);
    let last_balance_id = get_last_balance_id(&sp)?;

    let mut final_last_balance = Vec::new();

    // get the deletion tx data
    let query = format!("SELECT * FROM tx_all Where id_num = {}", id_num);
    let data = sp.query_row(&query, [], |row| {
        let mut final_data: Vec<String> = Vec::new();
        final_data.push(row.get(0)?);
        final_data.push(row.get(2)?);
        final_data.push(row.get(3)?);
        final_data.push(row.get(4)?);
        Ok(final_data)
    })?;

    let splitted = data[0].split('-').collect::<Vec<&str>>();
    let (year, month) = (
        splitted[0].parse::<i32>().unwrap(),
        splitted[1].parse::<i32>().unwrap(),
    );

    let year = year - 2022;

    let mut target_id_num = month + (year * 12);

    //
    let mut from_method = "";
    let mut to_method = "";

    // the tx_method of the tx
    let source = &data[1];

    // execute this block to get block tx method if the tx type is a Transfer
    if source.contains(" to ") {
        let from_to = data[1].split(" to ").collect::<Vec<&str>>();

        from_method = from_to[0];
        to_method = from_to[1];
    }

    let amount = &data[2].parse::<f64>().unwrap();
    let tx_type: &str = &data[3];

    // loop through all rows in the balance_all table from the deletion point and update balance
    loop {
        let mut query = format!(
            "SELECT {:?} FROM balance_all WHERE id_num = {}",
            tx_methods, target_id_num
        );
        query = query.replace('[', "");
        query = query.replace(']', "");

        let cu_month_balance = sp.query_row(&query, [], |row| {
            let mut final_data: Vec<String> = Vec::new();
            for i in 0..tx_methods.len() {
                final_data.push(row.get(i)?)
            }
            Ok(final_data)
        })?;

        let mut updated_month_balance = vec![];

        // reverse that amount that was previously added and commit them to db
        // add or subtract based on the tx type to the relevant method

        // check the month balance as not zero because if it is 0, there was never any transaction
        // done on that month
        for i in 0..tx_methods.len() {
            if &tx_methods[i] == source && cu_month_balance[i] != "0.00" {
                let mut cu_int_amount = cu_month_balance[i].parse::<f64>().unwrap();
                if tx_type == "Expense" {
                    cu_int_amount += amount;
                } else if tx_type == "Income" {
                    cu_int_amount -= amount;
                }
                updated_month_balance.push(format!("{:.2}", cu_int_amount));
            } else if tx_methods[i] == from_method && cu_month_balance[i] != "0.00" {
                let mut cu_int_amount = cu_month_balance[i].parse::<f64>().unwrap();
                cu_int_amount += amount;
                updated_month_balance.push(format!("{:.2}", cu_int_amount));
            } else if tx_methods[i] == to_method && cu_month_balance[i] != "0.00" {
                let mut cu_int_amount = cu_month_balance[i].parse::<f64>().unwrap();
                cu_int_amount -= amount;
                updated_month_balance.push(format!("{:.2}", cu_int_amount));
            } else {
                updated_month_balance.push(format!(
                    "{:.2}",
                    cu_month_balance[i].parse::<f64>().unwrap()
                ));
            }
        }

        // the query kept on breaking for a single comma so had to follow this ugly way to do this.
        // loop and add a comma until the last index and ignore it in the last time
        let mut balance_query = "UPDATE balance_all SET ".to_string();
        for i in 0..updated_month_balance.len() {
            if i != updated_month_balance.len() - 1 {
                balance_query.push_str(&format!(
                    r#""{}" = "{}", "#,
                    tx_methods[i], updated_month_balance[i]
                ))
            } else {
                balance_query.push_str(&format!(
                    r#""{}" = "{}" "#,
                    tx_methods[i], updated_month_balance[i]
                ))
            }
        }
        balance_query.push_str(&format!("WHERE id_num = {target_id_num}"));
        sp.execute(&balance_query, [])?;

        // 49 is the absolute final balance which we don't need to modify
        target_id_num += 1;
        if target_id_num == 49 {
            break;
        }
    }

    // we are deleting 1 transaction, so loop through all tx methods, and whichever method matches
    // with the one we are deleting, add/subtract from the amount.
    // Calculate the balance/s for the absolute final balance and create the query
    for i in 0..tx_methods.len() {
        let mut cu_balance = last_balance[i].parse::<f64>().unwrap();
        if &tx_methods[i] == source && tx_type != "Transfer" {
            match tx_type {
                "Expense" => cu_balance += amount,
                "Income" => cu_balance -= amount,
                _ => {}
            }
        } else if tx_methods[i] == from_method && tx_type == "Transfer" {
            cu_balance += amount;
        } else if tx_methods[i] == to_method && tx_type == "Transfer" {
            cu_balance -= amount;
        }
        final_last_balance.push(format!("{:.2}", cu_balance));
    }

    let del_query = format!("DELETE FROM tx_all WHERE id_num = {id_num}");

    let mut last_balance_query = "UPDATE balance_all SET ".to_string();
    for i in 0..final_last_balance.len() {
        if i != final_last_balance.len() - 1 {
            last_balance_query.push_str(&format!(
                r#""{}" = "{}", "#,
                tx_methods[i], final_last_balance[i]
            ))
        } else {
            last_balance_query.push_str(&format!(
                r#""{}" = "{}" "#,
                tx_methods[i], final_last_balance[i]
            ))
        }
    }
    last_balance_query.push_str(&format!("WHERE id_num = {last_balance_id}"));
    sp.execute(&last_balance_query, [])?;
    sp.execute(&del_query, [])?;

    sp.commit()?;
    Ok(())
}