use std::process;

use clap::Parser;
use color_eyre::Result;
use email::{backend::feature::BackendFeatureSource, folder::purge::PurgeFolder};
use pimalaya_tui::prompt;
use tracing::info;

use crate::{
    account::arg::name::AccountNameFlag, backend::Backend, config::Config,
    folder::arg::name::FolderNameArg, printer::Printer,
};

/// Purge a folder.
///
/// All emails from the given folder are definitely deleted. The
/// purged folder will remain empty after execution of the command.
#[derive(Debug, Parser)]
pub struct FolderPurgeCommand {
    #[command(flatten)]
    pub folder: FolderNameArg,

    #[command(flatten)]
    pub account: AccountNameFlag,
}

impl FolderPurgeCommand {
    pub async fn execute(self, printer: &mut impl Printer, config: &Config) -> Result<()> {
        info!("executing purge folder command");

        let folder = &self.folder.name;

        let confirm = format!("Do you really want to purge the folder {folder}? All emails will be definitely deleted.");

        if !prompt::bool(confirm, false)? {
            process::exit(0);
        };

        let (toml_account_config, account_config) = config
            .clone()
            .into_account_configs(self.account.name.as_deref())?;

        let purge_folder_kind = toml_account_config.purge_folder_kind();

        let backend = Backend::new(
            toml_account_config.clone(),
            account_config,
            purge_folder_kind,
            |builder| builder.set_purge_folder(BackendFeatureSource::Context),
        )
        .await?;

        backend.purge_folder(folder).await?;

        printer.log(format!("Folder {folder} successfully purged!"))
    }
}
