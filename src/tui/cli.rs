use clap;

use log::debug;

use crate::config::model::Config;
use crate::config::tui::KeybindingError;

use super::model::{Tui, TuiError};

/// Here are all subcommands related to the tui.
pub fn tui_subcmds<'a>() -> Vec<clap::App<'a, 'a>> {
    vec![clap::SubCommand::with_name("tui").about("Opens himalaya with the TUI")]
}

/// This function will look which subcommands (which belong to the TUI) has
/// been added in the commandline arguments and execute the appropriate code.
pub fn tui_matches<'func>(
    arg_matches: &clap::ArgMatches<'func>,
    config: &Config,
) -> Result<(), ()> {
    if let Some(_) = arg_matches.subcommand_matches("tui") {
        debug!("TUI subcommand matched => Opening TUI");

        // Start the TUI
        let mut tui = match Tui::new(config) {
            Ok(tui) => tui,
            Err(error) => match error {
                KeybindingError::NodeConflict(keybinding) => {
                    println!("Keybinding conflict:");
                    println!(
                        "There's a similiar keybinding like: {}",
                        keybinding
                    );
                    println!("Please take a look into your config again!");
                    panic!("Keybding conflict!");
                },
                KeybindingError::TraverseError => {
                    println!("The binary search failed to find the next node");
                    println!(
                        "of the keybinding tree! Please create a new issue!"
                    );
                    panic!("Node-Traversing-Error");
                }
            },
        };

        if let Err(err) = tui.run() {
            match err {
                TuiError::TerminalPreparation(io_err) => {
                    println!("An IO Error Happended!");
                    println!("{}", io_err);
                    panic!("Couldn't prepare the terminal for TUI.");
                },
                TuiError::EventKey =>
                    panic!("Couldn't handle the pressed keys during TUI session."),
                TuiError::Draw =>
                    panic!("Couldn't draw the TUI."),
                TuiError::RawMode(err) => {
                    println!("A Terminal-Error happened!");
                    println!("{}", err);
                    panic!("Couldn't put terminal into raw mode.");
                },
                TuiError::UnknownAccount =>
                    panic!("Couldn't find given account."),
                TuiError::ConnectAccount =>
                    panic!("Couldn't connect to IMAP server with given account."),
                TuiError::Sidebar =>
                    panic!("Couldn't load the mailboxes into the sidebar of the given account."),
                TuiError::MailList =>
                    panic!("Couldn't load the mails into the mail_list frame of the given account."),
            }
        }

        return Ok(());
    }

    debug!("Nothing matched");
    Err(())
}
