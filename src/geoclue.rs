#![allow(clippy::future_not_send)]

use std::time::Duration;

use anyhow::{bail, Context};
use async_io::Timer;
use futures_lite::{stream::StreamExt, FutureExt};
use zbus::{proxy, zvariant::ObjectPath, Connection, Result};

use crate::geo::Coords;

#[proxy(
    default_service = "org.freedesktop.GeoClue2",
    interface = "org.freedesktop.GeoClue2.Manager",
    default_path = "/org/freedesktop/GeoClue2/Manager"
)]
trait GeoClueManager {
    #[zbus(object = "GeoClueClient")]
    fn get_client(&self);
}

#[proxy(
    default_service = "org.freedesktop.GeoClue2",
    interface = "org.freedesktop.GeoClue2.Client"
)]
trait GeoClueClient {
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;

    #[zbus(property)]
    fn set_desktop_id(&self, desktop_id: &str) -> Result<()>;

    #[zbus(signal)]
    fn location_updated(&self, old: ObjectPath<'_>, new: ObjectPath<'_>) -> Result<()>;
}

#[proxy(
    default_service = "org.freedesktop.GeoClue2",
    interface = "org.freedesktop.GeoClue2.Location"
)]
trait GeoClueLocation {
    #[zbus(property)]
    fn latitude(&self) -> Result<f64>;
    #[zbus(property)]
    fn longitude(&self) -> Result<f64>;
}

async fn get_location_async() -> anyhow::Result<Coords> {
    log::debug!("Trying to get location from GeoClue");

    let connection = Connection::system()
        .await
        .context("couldn't connect to dbus")?;
    let manager = GeoClueManagerProxy::new(&connection)
        .await
        .context("couldn't create GeoClue manager")?;
    let client = manager
        .get_client()
        .await
        .context("couldn't get CeoClue client")?;

    client.set_desktop_id("timewall").await?;
    let mut location_updated = client.receive_location_updated().await?;
    client.start().await?;

    let signal = location_updated.next().await.unwrap();
    let args = signal.args()?;
    let location = GeoClueLocationProxy::builder(&connection)
        .path(args.new())?
        .build()
        .await?;

    client.stop().await?;

    let coords = Coords {
        lat: location.latitude().await?,
        lon: location.longitude().await?,
    };

    log::debug!("Got location from GeoClue: {:?}", coords);

    Ok(coords)
}

pub fn get_location(timeout: Duration) -> anyhow::Result<Coords> {
    async_io::block_on(get_location_async().or(async {
        Timer::after(timeout).await;
        bail!("timed out while waiting for location from GeoClue")
    }))
}
