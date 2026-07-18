use anyhow::Result;
use zbus::zvariant::Value;
use zbus::Connection;

pub async fn system_hostname() -> Result<String> {
    let connection = Connection::system().await?;

    let reply = connection
        .call_method(
            Some("org.freedesktop.hostname1"),
            "/org/freedesktop/hostname1",
            Some("org.freedesktop.DBus.Properties"),
            "Get",
            &("org.freedesktop.hostname1", "Hostname"),
        )
        .await?;

    let body = reply.body();
    let value: Value = body.deserialize()?;
    let hostname: String = value.try_into()?;
    Ok(hostname)
}
