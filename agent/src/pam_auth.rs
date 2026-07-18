use anyhow::{anyhow, Result};
use pam::Client;

#[allow(dead_code)]
pub fn authenticate(username: &str, password: &str) -> Result<()> {
    let mut client = Client::with_password("login").map_err(|e| anyhow!("PAM init failed: {e}"))?;

    client
        .conversation_mut()
        .set_credentials(username, password);

    client
        .authenticate()
        .map_err(|e| anyhow!("PAM authentication failed: {e}"))?;

    client
        .open_session()
        .map_err(|e| anyhow!("failed to open PAM session: {e}"))?;

    Ok(())
}
