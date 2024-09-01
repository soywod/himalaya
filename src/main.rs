use clap::Parser;
use color_eyre::Result;
use himalaya::{
    cli::Cli, config::Config, envelope::command::list::ListEnvelopesCommand,
    message::command::mailto::MessageMailtoCommand, printer::StdoutPrinter,
};
use pimalaya_tui::cli::tracing;

#[tokio::main]
async fn main() -> Result<()> {
    let tracing = tracing::install()?;

    // if the first argument starts by "mailto:", execute straight the
    // mailto message command
    let mailto = std::env::args()
        .nth(1)
        .filter(|arg| arg.starts_with("mailto:"));

    if let Some(ref url) = mailto {
        let mut printer = StdoutPrinter::default();
        let config = Config::from_default_paths().await?;

        return MessageMailtoCommand::new(url)?
            .execute(&mut printer, &config)
            .await;
    }

    let cli = Cli::parse();
    let mut printer = StdoutPrinter::new(cli.output);
    let res = match cli.command {
        Some(cmd) => cmd.execute(&mut printer, cli.config_paths.as_ref()).await,
        None => {
            let config = Config::from_paths_or_default(cli.config_paths.as_ref()).await?;
            ListEnvelopesCommand::default()
                .execute(&mut printer, &config)
                .await
        }
    };

    tracing.with_debug_and_trace_notes(res)
}
