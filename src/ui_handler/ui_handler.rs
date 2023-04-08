use crate::add_tx_page::add_tx_ui;
use crate::chart_page::{chart_ui, ChartData};
use crate::home_page::home_ui;
use crate::home_page::TransactionData;
use crate::initial_page::initial_ui;
use crate::key_checker::{
    add_tx_keys, chart_keys, home_keys, initial_keys, summary_keys, transfer_keys, InputKeyHandler,
};
use crate::outputs::{HandlingOutput, UiHandlingError};
use crate::popup_page::add_popup;
use crate::summary_page::{summary_ui, SummaryData};
use crate::transfer_page::transfer_ui;
use crate::tx_handler::TxData;
use crate::ui_handler::{
    AddTxTab, CurrentUi, HomeTab, PopupState, TableData, TimeData, TransferTab,
};
use crate::utility::{get_all_tx_methods, get_empty_changes};
use crossterm::event::poll;
use crossterm::event::{self, Event};
use rusqlite::Connection;
use std::time::Duration;
use tui::backend::Backend;
use tui::layout::Constraint;
use tui::Terminal;

/// The core part that makes the entire program run. It loops
/// incredibly fast to refresh the terminal and passes the provided data to ui modules to draw them.
pub fn start_app<B: Backend>(
    terminal: &mut Terminal<B>,
    new_version_available: bool,
) -> Result<HandlingOutput, UiHandlingError> {
    // Setting up some default values. Let's go through all of them

    // contains the home page month list that can be indexed
    let mut months = TimeData::new_monthly();

    // contains the home page year list that can be indexed
    let mut years = TimeData::new_yearly();

    // We only want to open the version checker popup once, turns true once the checking is done
    let mut version_checked = false;

    // the selected widget on the Home Page. Default set to the month selection
    let mut selected_tab = HomeTab::Months;

    // Helps keep track if the Home Page's month or the year index was changed
    let mut last_month_index = 99;
    let mut last_year_index = 99;

    // the database connection and the path of the db
    let path = "data.sqlite";
    let conn = Connection::open(path).expect("Could not connect to database");
    conn.execute("PRAGMA foreign_keys = ON", [])
        .expect("Could not enable foreign keys");

    // Stores all data relevant for home page such as balance, changes and txs
    let mut all_data = TransactionData::new(&conn, 0, 0);
    // data for the Home Page's tx table
    let mut table = TableData::new(all_data.get_txs());

    // The page which is currently selected. Default is the initial page
    let mut cu_page = CurrentUi::Initial;
    // stores current popup status
    let mut cu_popup = PopupState::Nothing;
    // Stores the current selected widget on Add Transaction page
    let mut cu_tx_page = AddTxTab::Nothing;
    // Store the current selected widget on Add Transfer page
    let mut cu_transfer_page = TransferTab::Nothing;

    // Holds the data that will be/are inserted into the Add Tx page's input fields
    let mut data_for_tx = TxData::new();
    // Holds the data that will be/are inserted into the Transfer Tx page's input fields
    let mut data_for_transfer = TxData::new_transfer();
    // Holds the data that will be/are inserted into the Summary Page
    let mut summary_data = SummaryData::new(&conn);

    // Stores how many unique tags that stores. Used as an index for Summary Page table
    let mut total_tags = summary_data.get_table_data().len();
    // data for the Summary Page's table
    let mut summary_table = TableData::new(summary_data.get_table_data());
    // Texts and numbers of the details shown at top of summary page
    let mut summary_texts = summary_data.get_tx_data();
    // summary data generation is a heavy task. As it's a loop, the data is loaded once the page is selected.
    // If true, the data is no longer generated even if the summary page is on
    let mut summary_reloaded = false;

    // the initial page REX loading index
    let mut starter_index = 0;

    // The loop begins at this point and before the loop starts, multiple variables are initiated
    // with the default values which will quickly be changing once the loop starts.
    loop {
        // after each refresh this will check the current selected month, year and if a table/spreadsheet row is selected in the ui.
        let cu_month_index = months.index;
        let cu_year_index = years.index;
        let cu_table_index = table.state.selected();

        // reload the data saved in memory each time the month or the year changes
        if cu_month_index != last_month_index || cu_year_index != last_year_index {
            all_data = TransactionData::new(&conn, cu_month_index, cu_year_index);
            table = TableData::new(all_data.get_txs());
            last_month_index = cu_month_index;
            last_year_index = cu_year_index;
        };

        // total income and expense value for the home page
        let total_income = all_data.get_total_income(&conn, cu_table_index);
        let total_expense = all_data.get_total_expense(&conn, cu_table_index);

        // balance variable contains all the 'rows' of the Balance widget in the home page.
        // So each line is inside a vector. "" represents empty placeholder.
        let mut balance: Vec<Vec<String>> = vec![vec!["".to_string()]];
        balance[0].extend(get_all_tx_methods(&conn));
        balance[0].extend(vec!["Total".to_string()]);

        // save the % of space each column should take in the Balance section based on the total
        // transaction methods/columns available
        let width_percent = 100 / balance[0].len() as u16;
        let mut width_data = vec![];
        for _i in 0..balance[0].len() {
            width_data.push(Constraint::Percentage(width_percent));
        }

        // cu_table_index is the Home Page table widget index. If a row is selected,
        // get the balance there was once that transaction happened + the changes it did
        // otherwise, get the absolute final balance after all transaction happened + no changes.

        match cu_table_index {
            // pass out the current index to get the necessary balance & changes data
            Some(a) => {
                balance.push(all_data.get_balance(a));
                balance.push(all_data.get_changes(a));
            }
            // if none selected, get empty changes + the absolute final balance
            None => {
                balance.push(all_data.get_last_balance(&conn));
                balance.push(get_empty_changes(&conn));
            }
        }

        // total_income & total_expense data changes on each month/year index change. So push it now
        // to the balance vector to align with the rows.
        balance.push(total_income.clone());
        balance.push(total_expense.clone());

        // check the version of current TUI and based on that, turn on the popup
        if !version_checked {
            if new_version_available {
                cu_popup = PopupState::NewUpdate
            }
            version_checked = true;
        }

        // passing out relevant data to the ui function

        terminal
            .draw(|f| {
                match cu_page {
                    CurrentUi::Home => {
                        home_ui(
                            f,
                            &months,
                            &years,
                            &mut table,
                            &mut balance,
                            &selected_tab,
                            &mut width_data,
                        );
                        summary_reloaded = false;
                    }
                    CurrentUi::AddTx => {
                        add_tx_ui(
                            f,
                            data_for_tx.get_all_texts(),
                            &cu_tx_page,
                            &data_for_tx.tx_status,
                        );
                        summary_reloaded = false;
                    }
                    CurrentUi::Initial => {
                        initial_ui(f, starter_index);
                        starter_index += 1;
                        if starter_index > 27 {
                            starter_index = 0;
                        }
                        summary_reloaded = false;
                    }
                    CurrentUi::Transfer => {
                        transfer_ui(
                            f,
                            data_for_transfer.get_all_texts(),
                            &cu_transfer_page,
                            &data_for_transfer.tx_status,
                        );
                        summary_reloaded = false;
                    }
                    CurrentUi::Chart => {
                        let data_for_chart = ChartData::set(cu_year_index);
                        chart_ui(f, data_for_chart);
                        summary_reloaded = false;
                    }
                    CurrentUi::Summary => {
                        if !summary_reloaded {
                            summary_data = SummaryData::new(&conn);
                            total_tags = summary_data.get_table_data().len();
                            summary_table = TableData::new(summary_data.get_table_data());
                            summary_texts = summary_data.get_tx_data();
                            if total_tags > 0 {
                                summary_table.state.select(Some(0));
                            }
                        }
                        summary_reloaded = true;
                        summary_ui(f, &mut summary_table, &summary_texts);
                    }
                }
                add_popup(f, &cu_popup);
            })
            .map_err(|err| UiHandlingError::DrawingError(err))?;

        if let CurrentUi::Initial = cu_page {
            if !poll(Duration::from_millis(40)).map_err(|err| UiHandlingError::PollingError(err))? {
                continue;
            }
        }

        if let Event::Key(key) = event::read().map_err(|err| UiHandlingError::PollingError(err))? {
            let mut handler = InputKeyHandler::new(
                key,
                &mut cu_page,
                &mut cu_popup,
                &mut cu_tx_page,
                &mut cu_transfer_page,
                &mut data_for_tx,
                &mut data_for_transfer,
                &mut all_data,
                &mut table,
                &mut summary_table,
                &mut selected_tab,
                &mut months,
                &mut years,
                cu_month_index,
                cu_year_index,
                cu_table_index,
                total_tags,
                &conn,
            );

            let status;

            match handler.current_page {
                CurrentUi::Initial => {
                    status = initial_keys(&mut handler);
                }
                CurrentUi::Home => {
                    status = home_keys(&mut handler);
                }
                CurrentUi::AddTx => {
                    status = add_tx_keys(&mut handler);
                }
                CurrentUi::Transfer => {
                    status = transfer_keys(&mut handler);
                }
                CurrentUi::Chart => {
                    status = chart_keys(&mut handler);
                }
                CurrentUi::Summary => {
                    status = summary_keys(&mut handler);
                }
            }
            if let Some(output) = status {
                return Ok(output);
            }
        }
    }
}