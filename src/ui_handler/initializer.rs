use crate::initial_page::check_version;
use crate::ui_handler::start_app;
use crate::utility::{
    check_n_create_db, check_old_sql, enter_tui_interface, exit_tui_interface, get_user_tx_methods,
    start_terminal,
};
use crate::{db::add_new_tx_methods, outputs::HandlingOutput};
use atty::Stream;
use std::fs::File;
use std::io::prelude::*;
use std::{error::Error, process, thread, time::Duration};

pub fn initialize_app(verifying_path: &str, current_dir: &str) -> Result<(), Box<dyn Error>> {
    let new_version_available = check_version()?;
    if !atty::is(Stream::Stdout) {
        if let Err(err) = start_terminal(current_dir) {
            let mut open = File::create(format!("{current_dir}/Error.txt"))?;
            open.write_all(err.to_string().as_bytes())?;
            process::exit(1);
        }
    }

    // create a new db if not found. If there is an error, delete the failed data.sqlite file and exit
    check_n_create_db(verifying_path)?;

    // initiates migration if old database is detected.
    check_old_sql();

    loop {
        let mut terminal = enter_tui_interface()?;
        let result = start_app(&mut terminal, new_version_available);
        exit_tui_interface()?;

        match result {
            Ok(output) => match output {
                HandlingOutput::AddTxMethod => match get_user_tx_methods(true) {
                    Some(tx_methods) => {
                        let status = add_new_tx_methods("data.sqlite", tx_methods);
                        match status {
                            Ok(_) => {
                                println!("Added Transaction Methods Successfully. The app will restart in 5 seconds");
                                thread::sleep(Duration::from_millis(5000));
                            }
                            Err(e) => {
                                println!(
                                    "Error while adding new Transaction Methods. Error: {e:?}. Restarting in 5 seconds"
                                );
                                thread::sleep(Duration::from_millis(5000));
                            }
                        }
                    }
                    None => {
                        println!("Operation Cancelled. Restarting in 5 seconds");
                        thread::sleep(Duration::from_millis(5000));
                    }
                },
                HandlingOutput::QuitUi => break,
                HandlingOutput::PrintNewUpdate => println!("Could not open browser.\n\nLatest Version Link: https://github.com/TheRustyPickle/Rex/releases/latest")
            },
            Err(error) => {}
        }
    }

    Ok(())
}