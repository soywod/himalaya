use anyhow::{anyhow, Result};
use clap::Parser;
use log::info;

use crate::{
    account::arg::name::AccountNameFlag,
    backend::Backend,
    cache::arg::disable::DisableCacheFlag,
    config::TomlConfig,
    envelope::arg::ids::EnvelopeIdArg,
    folder::arg::name::FolderNameArg,
    message::arg::{body::BodyRawArg, header::HeaderRawArgs, reply::MessageReplyAllArg},
    printer::Printer,
};

/// Generate a reply message template
#[derive(Debug, Parser)]
pub struct TemplateReplyCommand {
    #[command(flatten)]
    pub folder: FolderNameArg,

    #[command(flatten)]
    pub envelope: EnvelopeIdArg,

    #[command(flatten)]
    pub reply: MessageReplyAllArg,

    #[command(flatten)]
    pub headers: HeaderRawArgs,

    #[command(flatten)]
    pub body: BodyRawArg,

    #[command(flatten)]
    pub cache: DisableCacheFlag,

    #[command(flatten)]
    pub account: AccountNameFlag,
}

impl TemplateReplyCommand {
    pub async fn execute(self, printer: &mut impl Printer, config: &TomlConfig) -> Result<()> {
        info!("executing template reply command");

        let folder = &self.folder.name;
        let id = self.envelope.id;
        let account = self.account.name.as_ref().map(String::as_str);
        let cache = self.cache.disable;

        let (toml_account_config, account_config) =
            config.clone().into_account_configs(account, cache)?;
        let backend = Backend::new(toml_account_config, account_config.clone(), false).await?;

        let tpl: String = backend
            .get_messages(folder, &[id])
            .await?
            .first()
            .ok_or(anyhow!("cannot find message {id}"))?
            .to_reply_tpl_builder(&account_config)
            .with_headers(self.headers.raw)
            .with_body(self.body.raw())
            .with_reply_all(self.reply.all)
            .build()
            .await?
            .into();

        printer.print(tpl)
    }
}
