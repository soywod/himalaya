//! Module related to IMAP handling.
//!
//! This module gathers all IMAP handlers triggered by the CLI.

use anyhow::Result;

use crate::domain::{config::entity::Config, imap::service::ImapServiceInterface};

/// Notify handler.
pub fn notify<ImapService: ImapServiceInterface>(
    keepalive: u64,
    config: &Config,
    imap: &mut ImapService,
) -> Result<()> {
    imap.notify(&config, keepalive)?;
    imap.logout()?;
    Ok(())
}

/// Watch handler.
pub fn watch<ImapService: ImapServiceInterface>(
    keepalive: u64,
    imap: &mut ImapService,
) -> Result<()> {
    imap.watch(keepalive)?;
    imap.logout()?;
    Ok(())
}