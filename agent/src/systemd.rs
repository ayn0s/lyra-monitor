use anyhow::{Context, Result};
use zbus::zvariant::OwnedObjectPath;
use zbus::Connection;

pub struct ServiceUnit {
    pub name: String,
    pub description: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
}

type UnitTuple = (
    String,
    String,
    String,
    String,
    String,
    String,
    OwnedObjectPath,
    u32,
    String,
    OwnedObjectPath,
);

pub async fn list_services() -> Result<Vec<ServiceUnit>> {
    let connection = Connection::system().await?;

    let reply = connection
        .call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            "ListUnits",
            &(),
        )
        .await
        .context("calling org.freedesktop.systemd1.Manager.ListUnits")?;

    let units: Vec<UnitTuple> = reply
        .body()
        .deserialize()
        .context("decoding ListUnits reply")?;

    let mut services: Vec<ServiceUnit> = units
        .into_iter()
        .filter(|unit| unit.0.ends_with(".service"))
        .map(|unit| ServiceUnit {
            name: unit.0,
            description: unit.1,
            load_state: unit.2,
            active_state: unit.3,
            sub_state: unit.4,
        })
        .collect();

    services.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(services)
}
